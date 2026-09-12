use std::net::{IpAddr, SocketAddr};
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
    /// Reply with permissive CORS headers. Off by default: see the `--cors`
    /// help text for why.
    pub cors: bool,
    /// Reject uploads larger than this many bytes. `None` = unlimited.
    pub max_upload: Option<u64>,
}

/// Render `host:port` for `TcpListener::bind` and for display.
///
/// A literal IPv6 address must be bracketed (`[fe80::1]:8080`); naively
/// formatting it as `fe80::1:8080` parses as a *different address* and fails
/// to bind. Anything that isn't a bare IP (a hostname, `localhost`) is
/// concatenated as-is.
pub fn join_host_port(host: &str, port: u16) -> String {
    match host.parse::<IpAddr>() {
        Ok(ip) => SocketAddr::new(ip, port).to_string(),
        Err(_) => format!("{}:{}", host, port),
    }
}

impl ServerConfig {
    /// `bind:port` string suitable for `TcpListener::bind`.
    pub fn bind_addr(&self) -> String {
        join_host_port(&self.bind, self.port)
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
            cors: args.cors,
            max_upload: args.max_upload_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a config the way the GUI does — directly, without going through
    /// `Args` (which would drag in its `process::exit` error handling).
    fn config(bind: &str, port: u16) -> ServerConfig {
        ServerConfig {
            root: PathBuf::from("."),
            bind: bind.to_string(),
            port,
            show_hidden: false,
            max_depth: -1,
            speed_limit: None,
            webdav: true,
            webdav_user: None,
            webdav_pass: None,
            webui_auth: false,
            cors: false,
            max_upload: None,
        }
    }

    #[test]
    fn ipv4_bind_addr_is_unbracketed() {
        assert_eq!(config("127.0.0.1", 8080).bind_addr(), "127.0.0.1:8080");
        assert_eq!(config("0.0.0.0", 80).bind_addr(), "0.0.0.0:80");
    }

    /// Regression: a literal IPv6 address used to be rendered as
    /// `fe80::1:8080` — a *different* address, which fails to bind (and which
    /// `::`/`::1` only escaped by luck). It must come out bracketed.
    #[test]
    fn ipv6_bind_addr_is_bracketed() {
        assert_eq!(config("::1", 8080).bind_addr(), "[::1]:8080");
        assert_eq!(config("::", 8080).bind_addr(), "[::]:8080");
        assert_eq!(config("fe80::1", 8080).bind_addr(), "[fe80::1]:8080");
        assert_eq!(
            config("2001:db8::abcd:1", 443).bind_addr(),
            "[2001:db8::abcd:1]:443"
        );
    }

    #[test]
    fn hostname_bind_addr_is_passed_through() {
        assert_eq!(config("localhost", 8080).bind_addr(), "localhost:8080");
    }

    #[test]
    fn wildcard_detection() {
        assert!(config("0.0.0.0", 8080).is_wildcard_bind());
        assert!(config("::", 8080).is_wildcard_bind());
        assert!(!config("127.0.0.1", 8080).is_wildcard_bind());
    }
}
