use std::path::PathBuf;

use crate::cli::Args;

/// Resolved server startup configuration.
///
/// This is the single source of truth for everything `server::run` needs to
/// stand up a listener. It is built either from parsed CLI [`Args`] (via
/// `From<&Args>`) or directly from the GUI form. Keeping it separate from
/// [`crate::handlers::AppState`] lets the config carry network details
/// (`bind`/`port`) that the request handlers don't care about.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub root: PathBuf,
    pub bind: String,
    pub port: u16,
    pub show_hidden: bool,
    pub max_depth: i32,
    pub speed_limit: Option<u64>,
    pub webdav: bool,
    pub webdav_user: Option<String>,
    pub webdav_pass: Option<String>,
    pub webui_auth: bool,
}

impl ServerConfig {
    /// `bind:port` string suitable for `TcpListener::bind`.
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }

    /// Whether this config binds to a wildcard address (all interfaces),
    /// in which case callers may want to enumerate LAN addresses.
    pub fn is_wildcard_bind(&self) -> bool {
        self.bind == "0.0.0.0" || self.bind == "::"
    }
}

impl From<&Args> for ServerConfig {
    /// Build a [`ServerConfig`] from parsed CLI arguments.
    ///
    /// Note: this delegates to `Args::root_path()` and
    /// `Args::speed_limit_bytes()`, which print an error and exit the process
    /// on invalid input. That behavior is intended for the CLI entry point;
    /// the GUI constructs `ServerConfig` directly and never goes through here.
    fn from(args: &Args) -> Self {
        ServerConfig {
            root: args.root_path(),
            bind: args.bind.clone(),
            port: args.port,
            show_hidden: args.show_hidden,
            max_depth: args.max_depth,
            speed_limit: args.speed_limit_bytes(),
            webdav: !args.no_webdav,
            webdav_user: args.webdav_user.clone(),
            webdav_pass: args.webdav_pass.clone(),
            webui_auth: args.webui_auth,
        }
    }
}
