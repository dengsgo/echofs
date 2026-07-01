use clap::Parser;
use echofs::config::ServerConfig;
use echofs::{cli, logging, netinfo, server};
use cli::Args;
use std::net::IpAddr;

fn main() {
    let args = Args::parse();

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

/// Run the server headlessly (classic CLI behavior). Builds a Tokio runtime
/// manually rather than via `#[tokio::main]` so that the GUI path is free to
/// own the main thread (required by native windowing on macOS).
fn run_headless(args: Args) {
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

        if args.open {
            let url = format!("http://127.0.0.1:{}", port);
            let _ = open::that(&url);
        }

        println!("Listening on {}", handle.local_addr);

        handle.wait().await;
    });
}
