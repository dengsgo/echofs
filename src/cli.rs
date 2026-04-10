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
        format!("{}:{}", self.bind, self.port)
    }

    pub fn speed_limit_bytes(&self) -> Option<u64> {
        self.speed_limit.as_ref().map(|s| {
            throttle::parse_speed(s).unwrap_or_else(|| {
                eprintln!("Error: invalid speed limit '{}' (examples: 500k, 1m, 10m)", s);
                std::process::exit(1);
            })
        })
    }
}
