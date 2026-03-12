use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Response};
use axum::response::{Html, IntoResponse};
use std::path::PathBuf;
use std::sync::Arc;

use crate::directory;
use crate::error::AppError;
use crate::mime_utils;
use crate::range;
use crate::template;

pub struct AppState {
    pub root: PathBuf,
}

pub async fn serve_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let full_path = &state.root;
    if full_path.is_dir() {
        Html(template::index_html()).into_response()
    } else {
        AppError::NotFound("Root is not a directory".into()).into_response()
    }
}

pub async fn serve_path(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    let rel_path = percent_encoding::percent_decode_str(&path)
        .decode_utf8_lossy()
        .to_string();

    let resolved = match directory::safe_resolve(&state.root, &rel_path) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    if resolved.is_dir() {
        Html(template::index_html()).into_response()
    } else if resolved.is_file() {
        let mime = mime_utils::detect_mime(&resolved);
        let content_type = if mime_utils::is_text(&mime) {
            format!("{}; charset=utf-8", mime)
        } else {
            mime.to_string()
        };
        match range::build_range_response(&resolved, &headers, &content_type).await {
            Ok(resp) => resp,
            Err(e) => AppError::from(e).into_response(),
        }
    } else {
        AppError::NotFound("Path not found".into()).into_response()
    }
}

pub async fn api_ls_root(
    State(state): State<Arc<AppState>>,
) -> Response<Body> {
    match directory::list_directory(&state.root, "") {
        Ok(listing) => axum::Json(listing).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn api_ls_path(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Response<Body> {
    let rel_path = percent_encoding::percent_decode_str(&path)
        .decode_utf8_lossy()
        .to_string();

    match directory::list_directory(&state.root, &rel_path) {
        Ok(listing) => axum::Json(listing).into_response(),
        Err(e) => e.into_response(),
    }
}
