use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

use crate::directory;
use crate::error::AppError;
use crate::mime_utils;
use crate::range;
use crate::template;
use crate::zip_stream;

pub struct AppState {
    pub root: PathBuf,
    pub show_hidden: bool,
    pub max_depth: i32,
    pub speed_limit: Option<u64>,
    pub webdav: bool,
    pub webdav_user: Option<String>,
    pub webdav_pass: Option<String>,
    pub webui_auth: bool,
}

/// JSON response wrapper that includes directory listing + server capabilities.
#[derive(Serialize)]
struct DirResponse {
    #[serde(flatten)]
    listing: directory::DirListing,
    webdav: bool,
    webdav_auth: bool,
}

fn is_ajax(headers: &HeaderMap) -> bool {
    headers
        .get("X-Requested-With")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "XMLHttpRequest")
        .unwrap_or(false)
}

/// `?download=zip` in the query string marks a folder ZIP-download request.
fn wants_zip_download(uri: &Uri) -> bool {
    uri.query()
        .map(|q| q.split('&').any(|pair| pair == "download=zip"))
        .unwrap_or(false)
}

/// ASCII chars that must be percent-encoded in an RFC 5987 `filename*` value.
/// Non-ASCII bytes are always encoded by `utf8_percent_encode`.
const FILENAME_STAR_ENCODE: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'%')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'{')
    .add(b'}');

/// `Content-Disposition: attachment` with an ASCII-only fallback filename plus
/// an RFC 5987 UTF-8 form so CJK folder names survive the download dialog.
fn zip_content_disposition(top_name: &str) -> String {
    let fallback: String = top_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let fallback = if fallback.is_empty() {
        "download"
    } else {
        fallback.as_str()
    };
    let encoded = percent_encoding::utf8_percent_encode(top_name, FILENAME_STAR_ENCODE);
    format!(
        "attachment; filename=\"{}.zip\"; filename*=UTF-8''{}.zip",
        fallback, encoded
    )
}

/// Shared handler for `GET <dir>?download=zip`: stream the folder as a ZIP.
/// The body is produced incrementally (no Content-Length, chunked), so the
/// download starts while compression is still running. HEAD short-circuits:
/// same headers, but no walk/compression is started at all.
async fn serve_dir_zip(
    state: &AppState,
    dir: PathBuf,
    rel_path: &str,
    headers: &HeaderMap,
    method: &axum::http::Method,
) -> Response<Body> {
    match zip_stream::zip_dir_body(
        dir,
        &state.root,
        rel_path,
        state.show_hidden,
        state.max_depth,
        state.speed_limit,
        *method == axum::http::Method::HEAD,
    )
    .await
    {
        Ok((body, top_name)) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/zip")
            .header(
                header::CONTENT_DISPOSITION,
                zip_content_disposition(&top_name),
            )
            .body(body)
            .expect("valid zip response with known headers"),
        Err(e) => e.into_response_for(headers),
    }
}

fn dir_response(listing: directory::DirListing, state: &AppState) -> DirResponse {
    DirResponse {
        listing,
        webdav: state.webdav,
        webdav_auth: state.webdav_user.is_some(),
    }
}

pub async fn serve_index(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    method: axum::http::Method,
) -> impl IntoResponse {
    let full_path = &state.root;
    if full_path.is_dir() {
        if wants_zip_download(&uri) {
            return serve_dir_zip(&state, state.root.clone(), "", &headers, &method).await;
        }
        let mut resp = if is_ajax(&headers) {
            match directory::list_directory(&state.root, "", state.show_hidden, state.max_depth).await {
                Ok(listing) => axum::Json(dir_response(listing, &state)).into_response(),
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
    uri: Uri,
    method: axum::http::Method,
) -> Response<Body> {
    let rel_path = percent_encoding::percent_decode_str(&path)
        .decode_utf8_lossy()
        .to_string();

    let resolved = match directory::safe_resolve(&state.root, &rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return e.into_response_for(&headers),
    };

    if resolved.is_dir() {
        if wants_zip_download(&uri) {
            return serve_dir_zip(&state, resolved, &rel_path, &headers, &method).await;
        }
        let mut resp = if is_ajax(&headers) {
            match directory::list_directory(&state.root, &rel_path, state.show_hidden, state.max_depth).await {
                Ok(listing) => axum::Json(dir_response(listing, &state)).into_response(),
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
