use clap::Parser;
use std::path::PathBuf;

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

    /// Access log output: "stdout" (default), "off" to disable, or a file path
    #[arg(short, long, default_value = "stdout")]
    pub log: String,
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
}
