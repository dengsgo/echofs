use clap::Parser;
use std::path::PathBuf;
use crate::throttle;

#[derive(Parser, Debug)]
#[command(name = "echofs", about = "A Rust file server with directory browsing and media preview")]
pub struct Args {
    /// Root directory to serve
    #[arg(short, long, default_value = ".")]
    pub root: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0")]
    pub bind: String,

    /// Open browser automatically
    #[arg(short, long, default_value_t = false)]
    pub open: bool,

    /// Show hidden files and directories (names starting with '.')
    #[arg(short = 'H', long, default_value_t = false)]
    pub show_hidden: bool,

    /// Maximum directory depth for browsing (-1 for unlimited)
    #[arg(short = 'd', long, default_value_t = -1)]
    pub max_depth: i32,

    /// Access log output: "stdout" (default), "off" to disable, or a file path
    #[arg(short, long, default_value = "stdout")]
    pub log: String,

    /// Speed limit per request, e.g. 500k, 1m, 10m (default: unlimited)
    #[arg(short = 's', long)]
    pub speed_limit: Option<String>,

    /// Disable WebDAV access (PROPFIND support for file managers)
    #[arg(long, default_value_t = false)]
    pub no_webdav: bool,

    /// WebDAV username (enables Basic Auth for WebDAV access when set; does not affect web UI)
    #[arg(long)]
    pub webdav_user: Option<String>,

    /// WebDAV password (used with --webdav-user)
    #[arg(long)]
    pub webdav_pass: Option<String>,

    /// Share WebDAV credentials with the web UI. When set, browser access to
    /// the directory listing / file downloads requires the same Basic Auth
    /// credentials as WebDAV. Requires --webdav-user to be configured.
    /// Default: off (web UI is open even when --webdav-user is set).
    #[arg(long, default_value_t = false)]
    pub webui_auth: bool,

    /// Allow cross-origin (CORS) requests with permissive Access-Control-* headers
    ///
    /// Off by default, and deliberately so: with CORS on, any web page the
    /// user visits can script destructive WebDAV calls (PUT/DELETE/MOVE) at
    /// a running server — and without --webdav-user nothing stands in the
    /// way. The bundled web UI is served from the same origin, so it never
    /// needs this. Only enable it for a genuinely cross-origin API client.
    #[arg(long, default_value_t = false)]
    pub cors: bool,

    /// Maximum size of a single upload (PUT), e.g. 100m, 2g (default: unlimited)
    // `<SIZE>` instead of the derived `<MAX_UPLOAD_SIZE>`: this is the longest
    // option name, and once the option column passes 40% of the terminal width
    // clap switches *every* option to next-line help. See clap's
    // `help_template::arg_next_line_help`.
    #[arg(long, value_name = "SIZE")]
    pub max_upload_size: Option<String>,

    /// Launch the desktop GUI control panel instead of running headless.
    /// (Also opened automatically when echofs is started with no arguments.)
    #[cfg(feature = "gui")]
    #[arg(long, default_value_t = false)]
    pub gui: bool,
}

impl Args {
    pub fn root_path(&self) -> PathBuf {
        let p = PathBuf::from(&self.root);
        std::fs::canonicalize(&p).unwrap_or_else(|_| {
            eprintln!("Error: root directory '{}' does not exist", self.root);
            std::process::exit(1);
        })
    }

    pub fn bind_addr(&self) -> String {
        crate::config::join_host_port(&self.bind, self.port)
    }

    pub fn speed_limit_bytes(&self) -> Option<u64> {
        self.speed_limit.as_ref().map(|s| {
            throttle::parse_speed(s).unwrap_or_else(|| {
                eprintln!("Error: invalid speed limit '{}' (examples: 500k, 1m, 10m)", s);
                std::process::exit(1);
            })
        })
    }

    pub fn max_upload_bytes(&self) -> Option<u64> {
        self.max_upload_size.as_ref().map(|s| {
            throttle::parse_speed(s).unwrap_or_else(|| {
                eprintln!(
                    "Error: invalid upload size limit '{}' (examples: 100m, 2g)",
                    s
                );
                std::process::exit(1);
            })
        })
    }

    /// Validate cross-option dependencies. Returns an error message on failure.
    pub fn validate(&self) -> Result<(), String> {
        if self.webui_auth && self.webdav_user.is_none() {
            return Err(
                "--webui-auth requires --webdav-user to be set (web UI shares WebDAV credentials)"
                    .to_string(),
            );
        }
        Ok(())
    }
}
