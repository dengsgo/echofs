//! Desktop GUI control panel (egui/eframe), compiled only with `--features gui`.
//!
//! The GUI owns the main thread (required by native windowing on macOS) and
//! drives a background Tokio runtime to start/stop the server. All server
//! lifecycle goes through [`crate::server::run`] and [`crate::server::ServerHandle`],
//! exactly like the CLI path — the GUI just builds a [`ServerConfig`] from form
//! fields instead of from parsed arguments.

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;

use eframe::egui;

use crate::cli::Args;
use crate::config::ServerConfig;
use crate::logging::LogTarget;
use crate::netinfo;
use crate::server::{self, ServerHandle};
use crate::throttle;

const MAX_LOG_LINES: usize = 1000;
const LOG_CHANNEL_CAP: usize = 4096;

/// Launch the GUI. Builds a multi-threaded Tokio runtime for server tasks and
/// runs the egui event loop on the calling (main) thread.
pub fn launch(args: Args) {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(e) => {
            eprintln!("Failed to start async runtime: {}", e);
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 680.0])
            .with_min_inner_size([460.0, 520.0])
            .with_title("EchoFS"),
        ..Default::default()
    };

    let app = EchoApp::new(args, runtime);

    if let Err(e) = eframe::run_native("EchoFS", native_options, Box::new(|_cc| Ok(Box::new(app)))) {
        eprintln!("GUI error: {}", e);
        std::process::exit(1);
    }
}

/// Editable form state plus runtime handles.
struct EchoApp {
    rt: Arc<tokio::runtime::Runtime>,

    // --- form fields (strings where the input is free-form / numeric) ---
    root: String,
    bind: String,
    port: String,
    show_hidden: bool,
    max_depth: String,
    speed_limit: String,
    webdav: bool,
    webdav_user: String,
    webdav_pass: String,
    open_browser: bool,

    // --- running state ---
    handle: Option<ServerHandle>,
    local_addr: Option<SocketAddr>,
    log_rx: Option<tokio::sync::broadcast::Receiver<String>>,
    logs: VecDeque<String>,
    status: Status,

    // --- QR popup ---
    qr_for: Option<String>,
    qr_texture: Option<(String, egui::TextureHandle)>,
}

enum Status {
    Idle,
    Running,
    Error(String),
}

impl EchoApp {
    fn new(args: Args, rt: Arc<tokio::runtime::Runtime>) -> Self {
        // Pre-fill the form from CLI args. Resolve root for display but fall
        // back to the raw string if it doesn't canonicalize yet.
        let root = std::fs::canonicalize(&args.root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| args.root.clone());

        let max_depth = if args.max_depth < 0 {
            String::new() // empty = unlimited
        } else {
            args.max_depth.to_string()
        };

        EchoApp {
            rt,
            root,
            bind: args.bind.clone(),
            port: args.port.to_string(),
            show_hidden: args.show_hidden,
            max_depth,
            speed_limit: args.speed_limit.clone().unwrap_or_default(),
            webdav: !args.no_webdav,
            webdav_user: args.webdav_user.clone().unwrap_or_default(),
            webdav_pass: args.webdav_pass.clone().unwrap_or_default(),
            open_browser: args.open,
            handle: None,
            local_addr: None,
            log_rx: None,
            logs: VecDeque::new(),
            status: Status::Idle,
            qr_for: None,
            qr_texture: None,
        }
    }

    fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    /// Validate the form into a [`ServerConfig`], or return an error message.
    fn build_config(&self) -> Result<ServerConfig, String> {
        let root_path = std::fs::canonicalize(&self.root)
            .map_err(|_| format!("Root directory does not exist: {}", self.root))?;
        if !root_path.is_dir() {
            return Err(format!("Root is not a directory: {}", self.root));
        }

        let bind = self.bind.trim().to_string();
        if bind.is_empty() {
            return Err("Bind address must not be empty".to_string());
        }

        let port: u16 = self
            .port
            .trim()
            .parse()
            .map_err(|_| format!("Invalid port: {}", self.port))?;

        let max_depth: i32 = if self.max_depth.trim().is_empty() {
            -1
        } else {
            self.max_depth
                .trim()
                .parse()
                .map_err(|_| format!("Invalid max depth: {}", self.max_depth))?
        };

        let speed_limit = if self.speed_limit.trim().is_empty() {
            None
        } else {
            Some(
                throttle::parse_speed(self.speed_limit.trim())
                    .ok_or_else(|| format!("Invalid speed limit: {} (examples: 500k, 1m, 10m)", self.speed_limit))?,
            )
        };

        let webdav_user = if self.webdav_user.trim().is_empty() {
            None
        } else {
            Some(self.webdav_user.trim().to_string())
        };
        let webdav_pass = if self.webdav_pass.is_empty() {
            None
        } else {
            Some(self.webdav_pass.clone())
        };

        Ok(ServerConfig {
            root: root_path,
            bind,
            port,
            show_hidden: self.show_hidden,
            max_depth,
            speed_limit,
            webdav: self.webdav,
            webdav_user,
            webdav_pass,
        })
    }

    fn start(&mut self) {
        let config = match self.build_config() {
            Ok(c) => c,
            Err(e) => {
                self.status = Status::Error(e);
                return;
            }
        };

        // A broadcast channel feeds the live log panel.
        let (tx, rx) = tokio::sync::broadcast::channel::<String>(LOG_CHANNEL_CAP);
        let log_target = LogTarget::Channel(tx);

        let open_url = if self.open_browser {
            Some(format!("http://127.0.0.1:{}", config.port))
        } else {
            None
        };

        match self.rt.block_on(server::run(config, log_target)) {
            Ok(handle) => {
                self.local_addr = Some(handle.local_addr);
                self.handle = Some(handle);
                self.log_rx = Some(rx);
                self.logs.clear();
                self.status = Status::Running;
                if let Some(url) = open_url {
                    let _ = open::that(url);
                }
            }
            Err(e) => {
                self.status = Status::Error(format!("Failed to start: {}", e));
            }
        }
    }

    fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.rt.block_on(handle.stop());
        }
        self.local_addr = None;
        self.log_rx = None;
        self.status = Status::Idle;
    }

    /// Drain any pending log lines from the broadcast receiver into the ring buffer.
    fn drain_logs(&mut self) {
        use tokio::sync::broadcast::error::TryRecvError;
        if let Some(rx) = self.log_rx.as_mut() {
            loop {
                match rx.try_recv() {
                    Ok(line) => {
                        self.logs.push_back(line);
                        while self.logs.len() > MAX_LOG_LINES {
                            self.logs.pop_front();
                        }
                    }
                    Err(TryRecvError::Empty) | Err(TryRecvError::Closed) => break,
                    Err(TryRecvError::Lagged(_)) => continue,
                }
            }
        }
    }

    /// The list of URLs the running server is reachable on.
    fn addresses(&self) -> Vec<String> {
        let Some(addr) = self.local_addr else {
            return Vec::new();
        };
        let port = addr.port();
        let mut out = Vec::new();
        if self.bind == "0.0.0.0" || self.bind == "::" {
            out.push(format!("http://127.0.0.1:{}", port));
            for ip in netinfo::local_ips() {
                match ip {
                    std::net::IpAddr::V6(v6) => out.push(format!("http://[{}]:{}", v6, port)),
                    std::net::IpAddr::V4(v4) => out.push(format!("http://{}:{}", v4, port)),
                }
            }
        } else {
            out.push(format!("http://{}:{}", self.bind, port));
        }
        out
    }
}

impl eframe::App for EchoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain_logs();

        // The `Ui` handed to `App::ui` has no background or margin, so wrap the
        // whole UI in a CentralPanel — otherwise empty regions show the near
        // black window clear-color instead of the themed panel fill.
        egui::CentralPanel::default().show(ui, |ui| {
            let ctx = ui.ctx().clone();

            // --- Header ---
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("EchoFS");
                ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (dot, text) = match &self.status {
                        Status::Running => (egui::Color32::from_rgb(46, 204, 113), "Running"),
                        Status::Idle => (egui::Color32::GRAY, "Stopped"),
                        Status::Error(_) => (egui::Color32::from_rgb(231, 76, 60), "Error"),
                    };
                    ui.label(text);
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 6.0, dot);
                });
            });
            ui.separator();

            let running = self.is_running();

            // --- Configuration form (disabled while running) ---
            ui.add_enabled_ui(!running, |ui| {
                egui::Grid::new("config_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Root directory");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.root).desired_width(280.0));
                            if ui.button("Browse…").clicked()
                                && let Some(dir) = rfd::FileDialog::new().pick_folder()
                            {
                                self.root = dir.display().to_string();
                            }
                        });
                        ui.end_row();

                        ui.label("Bind address");
                        ui.add(egui::TextEdit::singleline(&mut self.bind).desired_width(160.0));
                        ui.end_row();

                        ui.label("Port");
                        ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(100.0));
                        ui.end_row();

                        ui.label("Max depth");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.max_depth).desired_width(100.0));
                            ui.label(egui::RichText::new("(empty = unlimited)").weak());
                        });
                        ui.end_row();

                        ui.label("Speed limit");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.speed_limit).desired_width(100.0));
                            ui.label(egui::RichText::new("e.g. 500k, 1m, 10m").weak());
                        });
                        ui.end_row();

                        ui.label("Options");
                        ui.vertical(|ui| {
                            ui.checkbox(&mut self.show_hidden, "Show hidden files");
                            ui.checkbox(&mut self.webdav, "Enable WebDAV");
                            ui.checkbox(&mut self.open_browser, "Open browser on start");
                        });
                        ui.end_row();

                        ui.label("WebDAV auth");
                        ui.add_enabled_ui(self.webdav, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.webdav_user)
                                        .hint_text("user")
                                        .desired_width(120.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.webdav_pass)
                                        .hint_text("password")
                                        .password(true)
                                        .desired_width(120.0),
                                );
                            });
                        });
                        ui.end_row();
                    });
            });

            ui.add_space(8.0);

            // --- Start / Stop ---
            ui.horizontal(|ui| {
                if running {
                    if ui.add(egui::Button::new("⏹  Stop").min_size(egui::vec2(100.0, 30.0))).clicked() {
                        self.stop();
                    }
                } else if ui.add(egui::Button::new("▶  Start").min_size(egui::vec2(100.0, 30.0))).clicked() {
                    self.start();
                }
            });

            if let Status::Error(msg) = &self.status {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::from_rgb(231, 76, 60), msg);
            }

            // --- Addresses ---
            if running {
                ui.add_space(10.0);
                ui.separator();
                ui.label(egui::RichText::new("Available at").strong());
                let addresses = self.addresses();
                for url in &addresses {
                    ui.horizontal(|ui| {
                        ui.monospace(url);
                        if ui.small_button("Copy").clicked() {
                            ctx.copy_text(url.clone());
                        }
                        if ui.small_button("Open").clicked() {
                            let _ = open::that(url);
                        }
                        if ui.small_button("QR").clicked() {
                            self.qr_for = Some(url.clone());
                        }
                    });
                }
            }

            // --- Live log ---
            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Access log").strong());
                if ui.small_button("Clear").clicked() {
                    self.logs.clear();
                }
            });
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if self.logs.is_empty() {
                        ui.label(egui::RichText::new("No requests yet.").weak());
                    } else {
                        for line in &self.logs {
                            ui.monospace(line);
                        }
                    }
                });

            // --- QR popup window ---
            if let Some(url) = self.qr_for.clone() {
                let mut open = true;
                egui::Window::new("QR Code")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(&ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            // Regenerate the texture only when the target URL changes.
                            let needs_build = self
                                .qr_texture
                                .as_ref()
                                .map(|(u, _)| u != &url)
                                .unwrap_or(true);
                            if needs_build
                                && let Some(img) = qr_color_image(&url)
                            {
                                let tex = ctx.load_texture("qr", img, egui::TextureOptions::NEAREST);
                                self.qr_texture = Some((url.clone(), tex));
                            }
                            if let Some((_, tex)) = &self.qr_texture {
                                ui.image((tex.id(), tex.size_vec2()));
                            }
                            ui.add_space(6.0);
                            ui.monospace(&url);
                        });
                    });
                if !open {
                    self.qr_for = None;
                }
            }

            // Keep the log panel live while the server runs, even without input.
            if self.is_running() {
                ctx.request_repaint_after(std::time::Duration::from_millis(300));
            }
        });
    }

    /// Gracefully stop the server when the window closes, so the async runtime
    /// isn't torn down with a live serving task still running.
    fn on_exit(&mut self) {
        self.stop();
    }
}

/// Render a QR code for `data` into an egui image (black on white, with a
/// quiet zone). Returns `None` if the data is too large to encode.
fn qr_color_image(data: &str) -> Option<egui::ColorImage> {
    let code = qrcode::QrCode::new(data.as_bytes()).ok()?;
    let width = code.width();
    let colors = code.to_colors();
    let quiet = 4usize;
    let scale = 6usize;
    let side = (width + quiet * 2) * scale;
    let mut rgba = vec![255u8; side * side * 4];
    for y in 0..width {
        for x in 0..width {
            if colors[y * width + x] == qrcode::Color::Dark {
                let px0 = (x + quiet) * scale;
                let py0 = (y + quiet) * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        let idx = ((py0 + dy) * side + (px0 + dx)) * 4;
                        rgba[idx] = 0;
                        rgba[idx + 1] = 0;
                        rgba[idx + 2] = 0;
                        rgba[idx + 3] = 255;
                    }
                }
            }
        }
    }
    Some(egui::ColorImage::from_rgba_unmultiplied([side, side], &rgba))
}
