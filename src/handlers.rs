use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, Response, header};
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
    pub show_hidden: bool,
    pub max_depth: i32,
    pub speed_limit: Option<u64>,
    pub webdav: bool,
}

fn is_ajax(headers: &HeaderMap) -> bool {
    headers
        .get("X-Requested-With")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "XMLHttpRequest")
        .unwrap_or(false)
}

pub async fn serve_index(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let full_path = &state.root;
    if full_path.is_dir() {
        let mut resp = if is_ajax(&headers) {
            match directory::list_directory(&state.root, "", state.show_hidden, state.max_depth).await {
                Ok(listing) => axum::Json(listing).into_response(),
                Err(e) => e.into_response_for(&headers),
            }
        } else {
            Html(template::index_html()).into_response()
        };
        resp.headers_mut()
            .insert(header::VARY, HeaderValue::from_static("X-Requested-With"));
        resp
    } else {
        AppError::NotFound("Root is not a directory".into()).into_response_for(&headers)
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

    let resolved = match directory::safe_resolve(&state.root, &rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return e.into_response_for(&headers),
    };

    if resolved.is_dir() {
        let mut resp = if is_ajax(&headers) {
            match directory::list_directory(&state.root, &rel_path, state.show_hidden, state.max_depth).await {
                Ok(listing) => axum::Json(listing).into_response(),
                Err(e) => e.into_response_for(&headers),
            }
        } else {
            Html(template::index_html()).into_response()
        };
        resp.headers_mut()
            .insert(header::VARY, HeaderValue::from_static("X-Requested-With"));
        resp
    } else if resolved.is_file() {
        let mime = mime_utils::detect_mime(&resolved);
        let content_type = if mime_utils::is_text(&mime) {
            format!("{}; charset=utf-8", mime)
        } else {
            mime.to_string()
        };
        match range::build_range_response(&resolved, &headers, &content_type, state.speed_limit).await {
            Ok(resp) => resp,
            Err(e) => AppError::from(e).into_response_for(&headers),
        }
    } else {
        AppError::NotFound("Path not found".into()).into_response_for(&headers)
    }
}
