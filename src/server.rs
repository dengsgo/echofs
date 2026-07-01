use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response};
use axum::middleware::Next;
use axum::routing::{any, get};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;

use crate::config::ServerConfig;
use crate::handlers::{self, AppState};
use crate::logging::{self, LogTarget};
use crate::webdav;

/// Error returned when the server fails to start.
#[derive(Debug)]
pub enum StartError {
    /// Failed to bind the TCP listener (e.g. port in use, permission denied).
    Bind(std::io::Error),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::Bind(e) => write!(f, "failed to bind listener: {}", e),
        }
    }
}

impl std::error::Error for StartError {}

/// Handle to a running server. Dropping it does **not** stop the server; call
/// [`ServerHandle::stop`] for a graceful shutdown, or [`ServerHandle::wait`] to
/// block until the server task finishes on its own.
pub struct ServerHandle {
    shutdown: Option<oneshot::Sender<()>>,
    join: tokio::task::JoinHandle<()>,
    /// The address the listener actually bound to. When the requested port was
    /// `0`, this carries the OS-assigned port.
    pub local_addr: SocketAddr,
}

impl ServerHandle {
    /// Signal graceful shutdown and wait for the server task to finish.
    pub async fn stop(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = self.join.await;
    }

    /// Block until the server task finishes (e.g. via an external signal or a
    /// fatal serve error). Used by the CLI entry point to run indefinitely.
    pub async fn wait(self) {
        let _ = self.join.await;
    }
}

/// Middleware that injects DAV headers into every response.
/// Applied as the outermost layer so it can modify CORS-preflight OPTIONS responses too.
async fn dav_headers_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("DAV", "1, 2".parse().expect("valid header"));
    if !headers.contains_key("Allow") {
        headers.insert("Allow", "OPTIONS, GET, HEAD, PUT, DELETE, MKCOL, COPY, MOVE, PROPFIND, PROPPATCH, LOCK, UNLOCK".parse().expect("valid header"));
    }
    response
}

/// Assemble the Axum router with all middleware layers applied.
///
/// Layer order: last `.layer()` = outermost (processes request first, response
/// last). We want: request → dav_headers → cors → logging → handler. Auth is
/// handled inside the WebDAV handlers (does not affect browser/web page access).
pub fn build_router(state: Arc<AppState>, log_target: LogTarget) -> Router {
    let webdav = state.webdav;

    let mut app = Router::new()
        .route("/", get(handlers::serve_index))
        .route("/{*path}", get(handlers::serve_path));

    if webdav {
        app = app
            .route("/", any(webdav::handle_webdav_root))
            .route("/{*path}", any(webdav::handle_webdav_path));
    }

    if webdav {
        app.layer(axum::middleware::from_fn_with_state(
            log_target,
            logging::access_log,
        ))
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(dav_headers_middleware))
        .with_state(state)
    } else {
        app.layer(axum::middleware::from_fn_with_state(
            log_target,
            logging::access_log,
        ))
        .layer(CorsLayer::permissive())
        .with_state(state)
    }
}

/// Bind the listener and spawn the server task.
///
/// Returns a [`ServerHandle`] for lifecycle control. Unlike a blocking server
/// loop, this returns as soon as the listener is bound, so callers (CLI or GUI)
/// can read the real bound address and decide how to wait or stop. Binding
/// failures are returned as [`StartError`] rather than terminating the process.
pub async fn run(config: ServerConfig, log_target: LogTarget) -> Result<ServerHandle, StartError> {
    let state = Arc::new(AppState {
        root: config.root,
        show_hidden: config.show_hidden,
        max_depth: config.max_depth,
        speed_limit: config.speed_limit,
        webdav: config.webdav,
        webdav_user: config.webdav_user,
        webdav_pass: config.webdav_pass,
    });

    let app = build_router(state, log_target);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.bind, config.port))
        .await
        .map_err(StartError::Bind)?;

    let local_addr = listener.local_addr().map_err(StartError::Bind)?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let join = tokio::spawn(async move {
        let serve = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });

        if let Err(e) = serve.await {
            eprintln!("Server error: {}", e);
        }
    });

    Ok(ServerHandle {
        shutdown: Some(shutdown_tx),
        join,
        local_addr,
    })
}
