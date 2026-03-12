use axum::Router;
use axum::routing::get;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::handlers::{self, AppState};
use crate::logging::{self, LogTarget};

pub async fn run(root: PathBuf, addr: &str, log_target: LogTarget) {
    let state = Arc::new(AppState { root });

    let app = Router::new()
        .route("/api/ls", get(handlers::api_ls_root))
        .route("/api/ls/{*path}", get(handlers::api_ls_path))
        .route("/", get(handlers::serve_index))
        .route("/{*path}", get(handlers::serve_path))
        .layer(axum::middleware::from_fn_with_state(
            log_target,
            logging::access_log,
        ))
        .layer(CorsLayer::permissive())
        .with_state(state);

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
