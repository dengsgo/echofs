use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response};
use axum::middleware::Next;
use axum::routing::{any, get};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::handlers::{self, AppState};
use crate::logging::{self, LogTarget};
use crate::webdav;

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

#[allow(clippy::too_many_arguments)]
pub async fn run(root: PathBuf, addr: &str, log_target: LogTarget, show_hidden: bool, max_depth: i32, speed_limit: Option<u64>, webdav: bool, webdav_user: Option<String>, webdav_pass: Option<String>) {
    let state = Arc::new(AppState { root, show_hidden, max_depth, speed_limit, webdav, webdav_user, webdav_pass });

    let mut app = Router::new()
        .route("/", get(handlers::serve_index))
        .route("/{*path}", get(handlers::serve_path));

    if webdav {
        app = app
            .route("/", any(webdav::handle_webdav_root))
            .route("/{*path}", any(webdav::handle_webdav_path));
    }

    // Layer order: last .layer() = outermost (processes request first, response last).
    // We want: request → dav_headers → cors → logging → handler
    // Auth is handled inside WebDAV handlers (does not affect browser/web page access).
    let app = if webdav {
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
    };

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });

    println!("Listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap_or_else(|e| {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    });
}
