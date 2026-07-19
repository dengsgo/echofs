// On Windows, a GUI-enabled build targets the "windows" subsystem so that
// double-clicking the .exe (the desktop-app launch path) does not spawn a
// console window alongside the egui window. When such a build is instead run
// from a terminal with CLI args (the headless path), `attach_parent_console`
// below re-attaches to the calling console so server output stays visible.
// The default (no-gui) build keeps the console subsystem and is unaffected.
#![cfg_attr(all(target_os = "windows", feature = "gui"), windows_subsystem = "windows")]

use clap::Parser;
use echofs::config::ServerConfig;
use echofs::{cli, logging, netinfo, server};
use cli::Args;
use std::net::IpAddr;

fn main() {
    let args = Args::parse();

    if let Err(msg) = args.validate() {
        eprintln!("Error: {}", msg);
        std::process::exit(2);
    }

    // In a GUI-enabled build, launch the desktop control panel when explicitly
    // requested with --gui, or when echofs is started with no arguments at all
    // (e.g. double-clicked). Any CLI arguments → headless server, as before.
    #[cfg(feature = "gui")]
    {
        let no_args = std::env::args_os().count() <= 1;
        if args.gui || no_args {
            echofs::gui::launch(args);
            return;
        }
    }

    run_headless(args);
}

/// When a `windows_subsystem = "windows"` build is launched from a terminal
/// (the headless path), Windows gives it no console, so `println!`/`eprintln!`
/// output would be lost. Re-attach to the parent process's console if it has
/// one; this is a no-op failure when there is none (e.g. a double-click launch),
/// which is exactly when we want no console. Only compiled for the gui build on
/// Windows — the case where `windows_subsystem = "windows"` is actually set.
#[cfg(all(target_os = "windows", feature = "gui"))]
fn attach_parent_console() {
    // ATTACH_PARENT_PROCESS (DWORD)-1: attach to the console of the parent.
    const ATTACH_PARENT_PROCESS: u32 = 0xFFFF_FFFF;
    unsafe extern "system" {
        fn AttachConsole(dwProcessId: u32) -> i32;
    }
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Run the server headlessly (classic CLI behavior). Builds a Tokio runtime
/// manually rather than via `#[tokio::main]` so that the GUI path is free to
/// own the main thread (required by native windowing on macOS).
fn run_headless(args: Args) {
    // In a windows-subsystem build (gui feature on Windows) the process starts
    // without a console; reattach the parent terminal's so CLI output shows.
    #[cfg(all(target_os = "windows", feature = "gui"))]
    attach_parent_console();

    let runtime = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
        eprintln!("Failed to start async runtime: {}", e);
        std::process::exit(1);
    });

    runtime.block_on(async move {
        let config = ServerConfig::from(&args);
        let log_target = logging::LogTarget::from_arg(&args.log);

        println!(r#"
  ______
 | ____  \
 |      \_\________      ___      _          ___  ___
 |          \ \ \  |    | __|__ | |_   ___  | __|| __|
 |          | | |  |    | _|/ _|| ' \ / _ \ | _| |__ \
 |          / / /  |    |___\__||_||_|\___/ |_|  |___/
 |_________________|    v{}
"#, env!("CARGO_PKG_VERSION"));

        // Start the server (binds the listener). On failure, report and exit.
        let handle = match server::run(config.clone(), log_target).await {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Failed to start server on {}: {}", config.bind_addr(), e);
                std::process::exit(1);
            }
        };

        let port = handle.local_addr.port();
        println!("Serving {} on http://{}:{}", config.root.display(), config.bind, port);

        if config.is_wildcard_bind() {
            println!("Available on:");
            println!("  http://127.0.0.1:{}", port);
            for ip in netinfo::local_ips() {
                match ip {
                    IpAddr::V6(v6) => println!("  http://[{}]:{}", v6, port),
                    _ => println!("  http://{}:{}", ip, port),
                }
            }
        }

        if let Some(limit) = config.speed_limit {
            let display = if limit >= 1024 * 1024 * 1024 {
                format!("{:.1} GB/s", limit as f64 / (1024.0 * 1024.0 * 1024.0))
            } else if limit >= 1024 * 1024 {
                format!("{:.1} MB/s", limit as f64 / (1024.0 * 1024.0))
            } else if limit >= 1024 {
                format!("{:.1} KB/s", limit as f64 / 1024.0)
            } else {
                format!("{} B/s", limit)
            };
            println!("Speed limit: {} per request", display);
        }

        if !config.webdav {
            println!("WebDAV: disabled");
        } else if let Some(user) = config.webdav_user.as_deref() {
            println!("WebDAV: enabled (auth required, user: {})", user);
        }

        if config.webui_auth {
            println!("Web UI auth: enabled (shares WebDAV credentials)");
        }

        if args.open {
            let url = format!("http://127.0.0.1:{}", port);
            let _ = open::that(&url);
        }

        println!("Listening on {}", handle.local_addr);

        handle.wait().await;
    });
}
