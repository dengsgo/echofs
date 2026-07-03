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

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([560.0, 680.0])
        .with_min_inner_size([460.0, 520.0])
        .with_title("EchoFS");

    // Window icon (Windows taskbar / Linux window). macOS uses the .app
    // bundle's .icns instead, so this is mainly for the other platforms; it is
    // harmless there. The PNG is generated from assets/icon.svg by
    // scripts/make-icon.sh and embedded so no runtime file is needed. A decode
    // failure just leaves the default icon — never fatal.
    match eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        Ok(icon) => viewport = viewport.with_icon(icon),
        Err(e) => eprintln!("warning: failed to load window icon: {}", e),
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let app = EchoApp::new(args, runtime);

    if let Err(e) = eframe::run_native(
        "EchoFS",
        native_options,
        Box::new(|cc| {
            // egui's bundled fonts cover only Latin/Cyrillic; without a CJK
            // fallback, Chinese/Japanese/Korean text renders as tofu boxes (□).
            install_cjk_font(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    ) {
        eprintln!("GUI error: {}", e);
        std::process::exit(1);
    }
}

/// System fonts that cover CJK (Chinese/Japanese/Korean) glyphs, in preference
/// order for the host platform. egui bundles only Latin/Cyrillic faces, so one
/// of these must back them up or CJK text shows as tofu boxes. We load from the
/// OS rather than embedding a font because CJK faces are 15–50 MB and embedding
/// would bloat the binary (see the "lean single-binary" convention); the shipped
/// GUI is the macOS .app, where a system CJK font is always present.
#[cfg(target_os = "macos")]
const CJK_FONT_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/PingFang.ttc",
    "/System/Library/Fonts/STHeiti Light.ttc",
    "/System/Library/Fonts/STHeiti Medium.ttc",
    "/System/Library/Fonts/Hiragino Sans GB.ttc",
    "/System/Library/Fonts/Supplemental/Songti.ttc",
    "/Library/Fonts/Arial Unicode.ttf",
];

#[cfg(target_os = "windows")]
const CJK_FONT_CANDIDATES: &[&str] = &[
    r"C:\Windows\Fonts\msyh.ttc",   // Microsoft YaHei (SC)
    r"C:\Windows\Fonts\simhei.ttf", // SimHei
    r"C:\Windows\Fonts\simsun.ttc", // SimSun
    r"C:\Windows\Fonts\msjh.ttc",   // Microsoft JhengHei (TC)
    r"C:\Windows\Fonts\malgun.ttf", // Malgun Gothic (KR)
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const CJK_FONT_CANDIDATES: &[&str] = &[
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJKsc-Regular.otf",
    "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
    "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
    "/usr/share/fonts/truetype/arphic/uming.ttc",
];

/// Install the first available system CJK font as a low-priority fallback for
/// both the proportional and monospace families, so Chinese/Japanese/Korean
/// text renders. The default Latin fonts stay primary — the CJK face is only
/// consulted for glyphs they lack. A no-op if no candidate is found (text then
/// degrades to the prior tofu-box behavior rather than failing).
fn install_cjk_font(ctx: &egui::Context) {
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};

    for path in CJK_FONT_CANDIDATES {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        ctx.add_font(FontInsert::new(
            "system-cjk",
            egui::FontData::from_owned(bytes),
            vec![
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: FontPriority::Lowest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: FontPriority::Lowest,
                },
            ],
        ));
        return;
    }

    eprintln!(
        "EchoFS: no system CJK font found; non-Latin text may not render. \
         Looked in: {}",
        CJK_FONT_CANDIDATES.join(", ")
    );
}

/// UI language. Auto-detected from the system locale on startup, switchable at
/// runtime via the header selector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lang {
    En,
    Zh,
}

impl Lang {
    /// Languages offered in the selector, in display order.
    const ALL: [Lang; 2] = [Lang::En, Lang::Zh];

    /// Endonym shown in the selector (always in the language's own script, the
    /// usual convention for language pickers).
    fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Zh => "中文",
        }
    }

    /// Pick the initial UI language from the OS locale, defaulting to English
    /// when it is unknown or not Chinese.
    fn detect() -> Lang {
        Lang::from_locale(system_locale().as_deref())
    }

    /// Classify a raw locale string (e.g. `"zh_CN.UTF-8"`, `"en_US"`) into a UI
    /// language. Chinese tags (`zh*`) select [`Lang::Zh`]; everything else,
    /// including `None`, falls back to [`Lang::En`]. Split out from [`detect`]
    /// so the matching rules are unit-testable without touching the environment.
    fn from_locale(locale: Option<&str>) -> Lang {
        match locale {
            Some(loc) if loc.trim_start().to_ascii_lowercase().starts_with("zh") => Lang::Zh,
            _ => Lang::En,
        }
    }
}

/// Best-effort read of the OS UI locale (e.g. `"zh_CN"`, `"en_US"`). Checks the
/// standard locale env vars first (set in most shells and Linux desktop
/// sessions), then falls back to a per-OS system query for GUI launches that
/// inherit no such vars: `defaults` on macOS (a Finder-launched `.app`) and the
/// Win32 user locale on Windows (which never sets POSIX locale vars at all).
/// Returns `None` when nothing usable is found (caller then defaults to English).
fn system_locale() -> Option<String> {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
        if let Ok(val) = std::env::var(key) {
            let val = val.trim();
            // The `C`/`POSIX` locales carry no real language preference.
            if !val.is_empty() && val != "C" && val != "POSIX" {
                return Some(val.to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleLocale"])
            .output()
            && out.status.success()
        {
            let loc = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !loc.is_empty() {
                return Some(loc);
            }
        }
    }

    // Windows sets no POSIX locale env vars, so an Explorer-launched `.exe`
    // reaches here with nothing from the loop above. Query the Win32 user
    // locale directly — `GetUserDefaultLocaleName` yields a BCP-47 name such as
    // `"zh-CN"` / `"en-US"`, which `Lang::from_locale` classifies. Hand-rolled
    // FFI (no extra crate), matching the `getifaddrs` approach in `netinfo`.
    #[cfg(target_os = "windows")]
    {
        // Max locale-name length incl. the null terminator (Win32 constant).
        const LOCALE_NAME_MAX_LENGTH: usize = 85;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            /// Writes a null-terminated UTF-16 locale name into `name` and
            /// returns its length in `u16`s (including the null), or 0 on error.
            fn GetUserDefaultLocaleName(name: *mut u16, size: i32) -> i32;
        }
        let mut buf = [0u16; LOCALE_NAME_MAX_LENGTH];
        let len = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
        // len includes the trailing null, so a real name needs len > 1.
        if len > 1 {
            let name = String::from_utf16_lossy(&buf[..(len as usize - 1)]);
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}

/// Pick the string for the active language. Translations live inline at each
/// call site — with only two languages and a small UI this is far easier to
/// review and keep in sync than a separate key table, and needs no dependency.
fn tr(lang: Lang, en: &'static str, zh: &'static str) -> &'static str {
    match lang {
        Lang::En => en,
        Lang::Zh => zh,
    }
}

/// Editable form state plus runtime handles.
struct EchoApp {
    rt: Arc<tokio::runtime::Runtime>,

    /// Currently selected UI language.
    lang: Lang,

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
            lang: Lang::detect(),
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
        let lang = self.lang;
        let root_path = std::fs::canonicalize(&self.root).map_err(|_| {
            format!(
                "{}: {}",
                tr(lang, "Root directory does not exist", "根目录不存在"),
                self.root
            )
        })?;
        if !root_path.is_dir() {
            return Err(format!(
                "{}: {}",
                tr(lang, "Root is not a directory", "根路径不是目录"),
                self.root
            ));
        }

        let bind = self.bind.trim().to_string();
        if bind.is_empty() {
            return Err(tr(lang, "Bind address must not be empty", "绑定地址不能为空").to_string());
        }

        let port: u16 = self
            .port
            .trim()
            .parse()
            .map_err(|_| format!("{}: {}", tr(lang, "Invalid port", "端口无效"), self.port))?;

        let max_depth: i32 = if self.max_depth.trim().is_empty() {
            -1
        } else {
            self.max_depth
                .trim()
                .parse()
                .map_err(|_| format!("{}: {}", tr(lang, "Invalid max depth", "最大深度无效"), self.max_depth))?
        };

        let speed_limit = if self.speed_limit.trim().is_empty() {
            None
        } else {
            Some(throttle::parse_speed(self.speed_limit.trim()).ok_or_else(|| {
                format!(
                    "{}: {} ({})",
                    tr(lang, "Invalid speed limit", "限速无效"),
                    self.speed_limit,
                    tr(lang, "examples: 500k, 1m, 10m", "示例：500k、1m、10m")
                )
            })?)
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
                self.status = Status::Error(format!(
                    "{}: {}",
                    tr(self.lang, "Failed to start", "启动失败"),
                    e
                ));
            }
        }
    }

    fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // Abort synchronously rather than `block_on`-ing a graceful drain:
            // a heavily throttled in-flight download (e.g. --speed-limit 1k)
            // can take hours to finish, and blocking the UI thread on it would
            // freeze the window. Aborting drops the listener (freeing the port)
            // immediately; any slow connection task dies in the background.
            handle.abort();
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
            let lang = self.lang;

            // --- Header ---
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("EchoFS");
                ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (dot, text) = match &self.status {
                        Status::Running => (egui::Color32::from_rgb(46, 204, 113), tr(lang, "Running", "运行中")),
                        Status::Idle => (egui::Color32::GRAY, tr(lang, "Stopped", "已停止")),
                        Status::Error(_) => (egui::Color32::from_rgb(231, 76, 60), tr(lang, "Error", "错误")),
                    };
                    ui.label(text);
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 6.0, dot);

                    // Language selector — always enabled, even while running.
                    ui.add_space(8.0);
                    egui::ComboBox::from_id_salt("lang_select")
                        .selected_text(self.lang.native_name())
                        .show_ui(ui, |ui| {
                            for opt in Lang::ALL {
                                ui.selectable_value(&mut self.lang, opt, opt.native_name());
                            }
                        })
                        .response
                        .on_hover_text(tr(lang, "Language", "语言"));
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
                        ui.label(tr(lang, "Root directory", "根目录"));
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.root).desired_width(280.0));
                            if ui.button(tr(lang, "Browse…", "浏览…")).clicked()
                                && let Some(dir) = rfd::FileDialog::new().pick_folder()
                            {
                                self.root = dir.display().to_string();
                            }
                        });
                        ui.end_row();

                        ui.label(tr(lang, "Bind address", "绑定地址"));
                        ui.add(egui::TextEdit::singleline(&mut self.bind).desired_width(160.0));
                        ui.end_row();

                        ui.label(tr(lang, "Port", "端口"));
                        ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(100.0));
                        ui.end_row();

                        ui.label(tr(lang, "Max depth", "最大深度"));
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.max_depth).desired_width(100.0));
                            ui.label(egui::RichText::new(tr(lang, "(empty = unlimited)", "（留空 = 不限制）")).weak());
                        });
                        ui.end_row();

                        ui.label(tr(lang, "Speed limit", "限速"));
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.speed_limit).desired_width(100.0));
                            ui.label(egui::RichText::new(tr(lang, "e.g. 500k, 1m, 10m", "如 500k、1m、10m")).weak());
                        });
                        ui.end_row();

                        ui.label(tr(lang, "Options", "选项"));
                        ui.vertical(|ui| {
                            ui.checkbox(&mut self.show_hidden, tr(lang, "Show hidden files", "显示隐藏文件"));
                            ui.checkbox(&mut self.webdav, tr(lang, "Enable WebDAV", "启用 WebDAV"));
                            ui.checkbox(&mut self.open_browser, tr(lang, "Open browser on start", "启动时打开浏览器"));
                        });
                        ui.end_row();

                        ui.label(tr(lang, "WebDAV auth", "WebDAV 认证"));
                        ui.add_enabled_ui(self.webdav, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.webdav_user)
                                        .hint_text(tr(lang, "user", "用户名"))
                                        .desired_width(120.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.webdav_pass)
                                        .hint_text(tr(lang, "password", "密码"))
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
                    if ui.add(egui::Button::new(tr(lang, "⏹  Stop", "⏹  停止")).min_size(egui::vec2(100.0, 30.0))).clicked() {
                        self.stop();
                    }
                } else if ui.add(egui::Button::new(tr(lang, "▶  Start", "▶  启动")).min_size(egui::vec2(100.0, 30.0))).clicked() {
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
                ui.label(egui::RichText::new(tr(lang, "Available at", "访问地址")).strong());
                let addresses = self.addresses();
                for url in &addresses {
                    ui.horizontal(|ui| {
                        ui.monospace(url);
                        if ui.small_button(tr(lang, "Copy", "复制")).clicked() {
                            ctx.copy_text(url.clone());
                        }
                        if ui.small_button(tr(lang, "Open", "打开")).clicked() {
                            let _ = open::that(url);
                        }
                        if ui.small_button(tr(lang, "QR", "二维码")).clicked() {
                            self.qr_for = Some(url.clone());
                        }
                    });
                }
            }

            // --- Live log ---
            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tr(lang, "Access log", "访问日志")).strong());
                if ui.small_button(tr(lang, "Clear", "清空")).clicked() {
                    self.logs.clear();
                }
            });
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if self.logs.is_empty() {
                        ui.label(egui::RichText::new(tr(lang, "No requests yet.", "暂无请求。")).weak());
                    } else {
                        for line in &self.logs {
                            ui.monospace(line);
                        }
                    }
                });

            // --- QR popup window ---
            if let Some(url) = self.qr_for.clone() {
                let mut open = true;
                egui::Window::new(tr(lang, "QR Code", "二维码"))
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

    /// Stop the server when the window closes. Uses the same non-blocking
    /// abort as the Stop button so closing the window never hangs, even with a
    /// throttled transfer still in flight.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_chinese_variants_select_zh() {
        for loc in ["zh", "zh_CN", "zh_CN.UTF-8", "zh-Hans", "zh_TW", "ZH_cn", " zh_CN"] {
            assert_eq!(Lang::from_locale(Some(loc)), Lang::Zh, "locale {loc:?}");
        }
    }

    #[test]
    fn locale_non_chinese_selects_en() {
        for loc in ["en_US.UTF-8", "en", "fr_FR", "ja_JP", "C", "POSIX", ""] {
            assert_eq!(Lang::from_locale(Some(loc)), Lang::En, "locale {loc:?}");
        }
    }

    #[test]
    fn locale_missing_defaults_to_en() {
        assert_eq!(Lang::from_locale(None), Lang::En);
    }

    #[test]
    fn tr_picks_the_active_language() {
        assert_eq!(tr(Lang::En, "Start", "启动"), "Start");
        assert_eq!(tr(Lang::Zh, "Start", "启动"), "启动");
    }

    #[test]
    fn every_language_has_a_native_name() {
        for lang in Lang::ALL {
            assert!(!lang.native_name().is_empty());
        }
    }
}
