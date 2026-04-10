use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, Response, StatusCode, Uri, header};
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::directory;
use crate::error::AppError;
use crate::handlers::AppState;
use crate::mime_utils;

const DAV_HEADER: &str = "1, 2";
const ALLOW_METHODS: &str = "OPTIONS, GET, HEAD, PUT, DELETE, MKCOL, COPY, MOVE, PROPFIND, PROPPATCH, LOCK, UNLOCK";

const SUPPORTED_LOCK_XML: &str = "\
<D:supportedlock>\n\
<D:lockentry>\n\
<D:lockscope><D:exclusive/></D:lockscope>\n\
<D:locktype><D:write/></D:locktype>\n\
</D:lockentry>\n\
</D:supportedlock>\n";

// ---------------------------------------------------------------------------
// XmlWriter — lightweight builder for XML output
// ---------------------------------------------------------------------------

struct XmlWriter(String);

impl XmlWriter {
    fn new() -> Self {
        Self(String::with_capacity(512))
    }

    fn declaration(&mut self) -> &mut Self {
        self.0.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        self
    }

    fn open(&mut self, tag: &str) -> &mut Self {
        self.0.push('<');
        self.0.push_str(tag);
        self.0.push_str(">\n");
        self
    }

    fn open_attr(&mut self, tag: &str, attrs: &str) -> &mut Self {
        self.0.push('<');
        self.0.push_str(tag);
        self.0.push(' ');
        self.0.push_str(attrs);
        self.0.push_str(">\n");
        self
    }

    fn close(&mut self, tag: &str) -> &mut Self {
        self.0.push_str("</");
        self.0.push_str(tag);
        self.0.push_str(">\n");
        self
    }

    fn tag(&mut self, tag: &str, text: &str) -> &mut Self {
        self.0.push('<');
        self.0.push_str(tag);
        self.0.push('>');
        self.0.push_str(&xml_escape(text));
        self.0.push_str("</");
        self.0.push_str(tag);
        self.0.push_str(">\n");
        self
    }

    fn tag_if(&mut self, tag: &str, text: &str) -> &mut Self {
        if !text.is_empty() {
            self.tag(tag, text);
        }
        self
    }

    fn empty(&mut self, tag: &str) -> &mut Self {
        self.0.push('<');
        self.0.push_str(tag);
        self.0.push_str("/>\n");
        self
    }

    fn raw(&mut self, s: &str) -> &mut Self {
        self.0.push_str(s);
        self
    }

    fn finish(self) -> String {
        self.0
    }
}

// ---------------------------------------------------------------------------
// DavResource — data carrier for a single WebDAV resource
// ---------------------------------------------------------------------------

struct DavResource {
    href: String,
    display_name: String,
    is_dir: bool,
    size: u64,
    content_type: String,
    creation_date: String,
    last_modified: String,
    etag: String,
}

impl DavResource {
    fn to_xml(&self) -> String {
        let mut w = XmlWriter::new();
        w.open("D:response")
            .tag("D:href", &self.href)
            .open("D:propstat")
            .open("D:prop")
            .tag("D:displayname", &self.display_name);
        if self.is_dir {
            w.raw("<D:resourcetype><D:collection/></D:resourcetype>\n");
        } else {
            w.empty("D:resourcetype")
                .tag("D:getcontentlength", &self.size.to_string());
        }
        w.tag_if("D:getcontenttype", &self.content_type)
            .tag_if("D:creationdate", &self.creation_date)
            .tag_if("D:getlastmodified", &self.last_modified)
            .tag_if("D:getetag", &self.etag)
            .raw(SUPPORTED_LOCK_XML)
            .close("D:prop")
            .tag("D:status", "HTTP/1.1 200 OK")
            .close("D:propstat")
            .close("D:response");
        w.finish()
    }
}

// ---------------------------------------------------------------------------
// Public handlers
// ---------------------------------------------------------------------------

/// Unified WebDAV handler for the root path `/`.
/// All WebDAV methods require auth when `--webdav-user` is configured.
pub async fn handle_webdav_root(
    method: Method,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    if let Err(resp) = check_auth(&state, &headers) {
        return resp;
    }
    match method.as_str() {
        "PROPFIND" => handle_propfind_inner(&state, "", &headers).await,
        "OPTIONS" => handle_options(),
        "LOCK" => handle_lock(&headers),
        "UNLOCK" => handle_unlock(),
        "PUT" => handle_put(&state, "", &headers, body).await,
        "DELETE" => handle_delete(&state, "", &headers).await,
        "MKCOL" => handle_mkcol(&state, "", &headers).await,
        "COPY" => handle_copy(&state, "", &headers).await,
        "MOVE" => handle_move(&state, "", &headers).await,
        "PROPPATCH" => handle_proppatch(""),
        _ => method_not_allowed(),
    }
}

/// Unified WebDAV handler for `/{*path}`.
/// All WebDAV methods require auth when `--webdav-user` is configured.
pub async fn handle_webdav_path(
    method: Method,
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    if let Err(resp) = check_auth(&state, &headers) {
        return resp;
    }
    let rel_path = percent_encoding::percent_decode_str(&path)
        .decode_utf8_lossy()
        .to_string();
    match method.as_str() {
        "PROPFIND" => handle_propfind_inner(&state, &rel_path, &headers).await,
        "OPTIONS" => handle_options(),
        "LOCK" => handle_lock(&headers),
        "UNLOCK" => handle_unlock(),
        "PUT" => handle_put(&state, &rel_path, &headers, body).await,
        "DELETE" => handle_delete(&state, &rel_path, &headers).await,
        "MKCOL" => handle_mkcol(&state, &rel_path, &headers).await,
        "COPY" => handle_copy(&state, &rel_path, &headers).await,
        "MOVE" => handle_move(&state, &rel_path, &headers).await,
        "PROPPATCH" => handle_proppatch(&rel_path),
        _ => method_not_allowed(),
    }
}

// ---------------------------------------------------------------------------
// Internal handlers
// ---------------------------------------------------------------------------

/// Build an OPTIONS response advertising WebDAV compliance level 1 and 2.
fn handle_options() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("DAV", DAV_HEADER)
        .header("Allow", ALLOW_METHODS)
        .header("MS-Author-Via", "DAV")
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building OPTIONS response")
}

/// Handle LOCK requests with a fake lock token.
/// Finder requires LOCK to succeed during its connection handshake.
/// Since we're read-only, the lock is a no-op but returns a valid response.
fn handle_lock(headers: &HeaderMap) -> Response<Body> {
    let timeout = headers
        .get("Timeout")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Second-3600");

    let lock_token = "opaquelocktoken:echofs-readonly-lock";

    let mut w = XmlWriter::new();
    w.declaration()
        .open("D:prop xmlns:D=\"DAV:\"")
        .open("D:lockdiscovery")
        .open("D:activelock")
        .raw("<D:locktype><D:write/></D:locktype>\n")
        .raw("<D:lockscope><D:exclusive/></D:lockscope>\n")
        .tag("D:depth", "infinity")
        .raw("<D:owner><D:href>anonymous</D:href></D:owner>\n")
        .tag("D:timeout", timeout)
        .raw(&format!(
            "<D:locktoken><D:href>{}</D:href></D:locktoken>\n",
            lock_token
        ))
        .raw("<D:lockroot><D:href>/</D:href></D:lockroot>\n")
        .close("D:activelock")
        .close("D:lockdiscovery")
        .close("D:prop");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("Lock-Token", format!("<{}>", lock_token))
        .header("DAV", DAV_HEADER)
        .body(Body::from(w.finish()))
        .expect("building LOCK response")
}

/// Handle UNLOCK requests — always succeeds (no-op for read-only server).
fn handle_unlock() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("DAV", DAV_HEADER)
        .body(Body::empty())
        .expect("building UNLOCK response")
}

/// Return 405 Method Not Allowed for unsupported methods.
fn method_not_allowed() -> Response<Body> {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building 405 response")
}

// ---------------------------------------------------------------------------
// Authentication
// ---------------------------------------------------------------------------

/// Check Basic Auth credentials.
/// Returns Ok(()) if auth passes (or no auth configured), Err(response) with 401 otherwise.
#[allow(clippy::result_large_err)]
pub fn check_auth(state: &AppState, headers: &HeaderMap) -> Result<(), Response<Body>> {
    let expected_user = match &state.webdav_user {
        Some(u) => u,
        None => return Ok(()), // no auth configured
    };
    let expected_pass = state.webdav_pass.as_deref().unwrap_or("");

    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let authorized = match auth_header {
        Some(val) if val.starts_with("Basic ") => {
            // base64 decode — we use a simple manual decoder to avoid adding a dependency
            match base64_decode(&val[6..]) {
                Some(decoded) => {
                    if let Some((user, pass)) = decoded.split_once(':') {
                        user == expected_user && pass == expected_pass
                    } else {
                        false
                    }
                }
                None => false,
            }
        }
        _ => false,
    };

    if authorized {
        Ok(())
    } else {
        Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("WWW-Authenticate", "Basic realm=\"echofs\"")
            .header("DAV", DAV_HEADER)
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .expect("building 401 response"))
    }
}

/// Minimal base64 decoder (standard alphabet, no padding required).
/// Returns None on invalid input.
fn base64_decode(input: &str) -> Option<String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    let mut out = Vec::new();

    for &b in input.trim().as_bytes() {
        if b == b'=' {
            break;
        }
        let val = TABLE.iter().position(|&c| c == b)? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    String::from_utf8(out).ok()
}

// ---------------------------------------------------------------------------
// Write operations
// ---------------------------------------------------------------------------

/// PUT — Upload or overwrite a file.
async fn handle_put(state: &AppState, rel_path: &str, _headers: &HeaderMap, body: Body) -> Response<Body> {

    let target = match directory::safe_resolve_parent(&state.root, rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    let existed = target.exists();

    // Collect body bytes
    use http_body_util::BodyExt;
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => return error_to_webdav_response(AppError::Internal(format!("Failed to read request body: {}", e))),
    };

    let data = bytes.to_vec();
    match tokio::task::spawn_blocking(move || {
        std::fs::write(&target, &data)
    }).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return error_to_webdav_response(AppError::from(e)),
        Err(e) => return error_to_webdav_response(AppError::Internal(format!("Task join error: {}", e))),
    }

    let status = if existed { StatusCode::NO_CONTENT } else { StatusCode::CREATED };
    Response::builder()
        .status(status)
        .header("DAV", DAV_HEADER)
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building PUT response")
}

/// DELETE — Remove a file or directory.
async fn handle_delete(state: &AppState, rel_path: &str, _headers: &HeaderMap) -> Response<Body> {

    let resolved = match directory::safe_resolve(&state.root, rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    let is_dir = resolved.is_dir();
    match tokio::task::spawn_blocking(move || {
        if is_dir {
            std::fs::remove_dir_all(&resolved)
        } else {
            std::fs::remove_file(&resolved)
        }
    }).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return error_to_webdav_response(AppError::from(e)),
        Err(e) => return error_to_webdav_response(AppError::Internal(format!("Task join error: {}", e))),
    }

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("DAV", DAV_HEADER)
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building DELETE response")
}

/// MKCOL — Create a directory.
async fn handle_mkcol(state: &AppState, rel_path: &str, _headers: &HeaderMap) -> Response<Body> {

    let target = match directory::safe_resolve_parent(&state.root, rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    if target.exists() {
        return error_to_webdav_response(AppError::Conflict("Resource already exists".into()));
    }

    match tokio::task::spawn_blocking(move || {
        std::fs::create_dir(&target)
    }).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return error_to_webdav_response(AppError::from(e)),
        Err(e) => return error_to_webdav_response(AppError::Internal(format!("Task join error: {}", e))),
    }

    Response::builder()
        .status(StatusCode::CREATED)
        .header("DAV", DAV_HEADER)
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building MKCOL response")
}

/// COPY — Copy a file or directory.
async fn handle_copy(state: &AppState, rel_path: &str, headers: &HeaderMap) -> Response<Body> {

    let dest_rel = match parse_destination(headers) {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let overwrite = parse_overwrite(headers);

    // Resolve source
    let source = match directory::safe_resolve(&state.root, rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    // Resolve destination (parent must exist)
    let dest = match directory::safe_resolve_parent(&state.root, &dest_rel, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    let dest_existed = dest.exists();
    if dest_existed && !overwrite {
        return error_to_webdav_response(AppError::Conflict("Destination exists and Overwrite is F".into()));
    }

    let is_dir = source.is_dir();
    match tokio::task::spawn_blocking(move || {
        if is_dir {
            copy_dir_recursive(&source, &dest)
        } else {
            // If destination exists and is a directory, remove it first
            if dest_existed && dest.is_dir() {
                std::fs::remove_dir_all(&dest)?;
            }
            std::fs::copy(&source, &dest)?;
            Ok(())
        }
    }).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return error_to_webdav_response(AppError::from(e)),
        Err(e) => return error_to_webdav_response(AppError::Internal(format!("Task join error: {}", e))),
    }

    let status = if dest_existed { StatusCode::NO_CONTENT } else { StatusCode::CREATED };
    Response::builder()
        .status(status)
        .header("DAV", DAV_HEADER)
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building COPY response")
}

/// MOVE — Move/rename a file or directory.
async fn handle_move(state: &AppState, rel_path: &str, headers: &HeaderMap) -> Response<Body> {

    let dest_rel = match parse_destination(headers) {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let overwrite = parse_overwrite(headers);

    // Resolve source
    let source = match directory::safe_resolve(&state.root, rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    // Resolve destination parent
    let dest = match directory::safe_resolve_parent(&state.root, &dest_rel, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    let dest_existed = dest.exists();
    if dest_existed && !overwrite {
        return error_to_webdav_response(AppError::Conflict("Destination exists and Overwrite is F".into()));
    }

    match tokio::task::spawn_blocking(move || {
        // Remove existing destination if overwriting
        if dest_existed {
            if dest.is_dir() {
                std::fs::remove_dir_all(&dest)?;
            } else {
                std::fs::remove_file(&dest)?;
            }
        }
        std::fs::rename(&source, &dest)
    }).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return error_to_webdav_response(AppError::from(e)),
        Err(e) => return error_to_webdav_response(AppError::Internal(format!("Task join error: {}", e))),
    }

    let status = if dest_existed { StatusCode::NO_CONTENT } else { StatusCode::CREATED };
    Response::builder()
        .status(status)
        .header("DAV", DAV_HEADER)
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building MOVE response")
}

/// PROPPATCH — Stub that returns success (macOS Finder compatibility).
fn handle_proppatch(rel_path: &str) -> Response<Body> {
    let href = if rel_path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", rel_path.trim_start_matches('/'))
    };

    let mut w = XmlWriter::new();
    w.declaration()
        .open_attr("D:multistatus", "xmlns:D=\"DAV:\"")
        .open("D:response")
        .tag("D:href", &href)
        .open("D:propstat")
        .empty("D:prop")
        .tag("D:status", "HTTP/1.1 200 OK")
        .close("D:propstat")
        .close("D:response")
        .close("D:multistatus");

    Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("DAV", DAV_HEADER)
        .body(Body::from(w.finish()))
        .expect("building PROPPATCH response")
}

// ---------------------------------------------------------------------------
// Write operation helpers
// ---------------------------------------------------------------------------

/// Parse the `Destination` header and extract the relative path.
#[allow(clippy::result_large_err)]
fn parse_destination(headers: &HeaderMap) -> Result<String, Response<Body>> {
    let dest_str = headers
        .get("Destination")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error_to_webdav_response(AppError::BadRequest("Missing Destination header".into()))
        })?;

    // Parse as URI to extract just the path component
    let path = if let Ok(uri) = dest_str.parse::<Uri>() {
        uri.path().to_string()
    } else {
        // Try as plain path
        dest_str.to_string()
    };

    let decoded = percent_encoding::percent_decode_str(&path)
        .decode_utf8_lossy()
        .to_string();

    Ok(decoded)
}

/// Parse the `Overwrite` header. Default is true (`T`).
fn parse_overwrite(headers: &HeaderMap) -> bool {
    headers
        .get("Overwrite")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim() != "F")
        .unwrap_or(true)
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if dst.exists() {
        std::fs::remove_dir_all(dst)?;
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// PROPFIND
// ---------------------------------------------------------------------------

/// PROPFIND implementation.
async fn handle_propfind_inner(
    state: &AppState,
    rel_path: &str,
    headers: &HeaderMap,
) -> Response<Body> {
    let depth = parse_depth(headers);

    let resolved = match directory::safe_resolve(&state.root, rel_path, state.show_hidden, state.max_depth).await {
        Ok(p) => p,
        Err(e) => return error_to_webdav_response(e),
    };

    let mut resources: Vec<DavResource> = Vec::new();

    if resolved.is_dir() {
        let dir_href = if rel_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}/", rel_path.trim_start_matches('/'))
        };
        match dir_resource_props(&resolved, &dir_href).await {
            Ok(res) => resources.push(res),
            Err(e) => return error_to_webdav_response(e),
        }

        if depth >= 1 {
            match directory::list_directory(&state.root, rel_path, state.show_hidden, state.max_depth).await {
                Ok(listing) => {
                    for entry in &listing.entries {
                        let child_href = if entry.is_dir {
                            format!("{}/", entry.href)
                        } else {
                            entry.href.clone()
                        };
                        resources.push(entry_to_resource(entry, &child_href));
                    }
                }
                Err(e) => return error_to_webdav_response(e),
            }
        }
    } else if resolved.is_file() {
        let file_href = if rel_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", rel_path.trim_start_matches('/'))
        };
        match file_resource_props(&resolved, &file_href).await {
            Ok(res) => resources.push(res),
            Err(e) => return error_to_webdav_response(e),
        }
    } else {
        return error_to_webdav_response(AppError::NotFound("Path not found".into()));
    }

    let mut w = XmlWriter::new();
    w.declaration()
        .open_attr("D:multistatus", "xmlns:D=\"DAV:\"");
    for res in &resources {
        w.raw(&res.to_xml());
    }
    w.close("D:multistatus");

    Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("DAV", DAV_HEADER)
        .body(Body::from(w.finish()))
        .expect("building PROPFIND response")
}

/// Parse the `Depth` header. Returns 0 or 1; treats `infinity` and missing as 1.
fn parse_depth(headers: &HeaderMap) -> u32 {
    headers
        .get("Depth")
        .and_then(|v| v.to_str().ok())
        .map(|v| match v.trim() {
            "0" => 0,
            "1" => 1,
            _ => 1,
        })
        .unwrap_or(1)
}

// ---------------------------------------------------------------------------
// Metadata collectors → DavResource
// ---------------------------------------------------------------------------

/// Build a `DavResource` for a directory from filesystem metadata.
async fn dir_resource_props(path: &std::path::Path, href: &str) -> Result<DavResource, AppError> {
    let path = path.to_path_buf();
    let href = href.to_string();
    tokio::task::spawn_blocking(move || {
        let meta = std::fs::metadata(&path).map_err(AppError::from)?;

        let created = meta
            .created()
            .ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
            })
            .unwrap_or_default();

        let modified = meta
            .modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
            })
            .unwrap_or_default();

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());

        let etag = format!("\"dir-{}\"", simple_hash(&href));

        Ok(DavResource {
            href,
            display_name: name,
            is_dir: true,
            size: 0,
            content_type: String::new(),
            creation_date: created,
            last_modified: modified,
            etag,
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// Build a `DavResource` for a file from filesystem metadata.
async fn file_resource_props(path: &std::path::Path, href: &str) -> Result<DavResource, AppError> {
    let path = path.to_path_buf();
    let href = href.to_string();
    tokio::task::spawn_blocking(move || {
        let meta = std::fs::metadata(&path).map_err(AppError::from)?;

        let created = meta
            .created()
            .ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
            })
            .unwrap_or_default();

        let modified = meta
            .modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
            })
            .unwrap_or_default();

        let modified_ts = meta
            .modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.timestamp()
            })
            .unwrap_or(0);

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let mime = mime_utils::detect_mime(&path);
        let etag = format!("\"{:x}-{:x}\"", meta.len(), modified_ts);

        Ok(DavResource {
            href,
            display_name: name,
            is_dir: false,
            size: meta.len(),
            content_type: mime.to_string(),
            creation_date: created,
            last_modified: modified,
            etag,
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// Build a `DavResource` from a `DirEntry` (used for directory children).
fn entry_to_resource(entry: &directory::DirEntry, href: &str) -> DavResource {
    let modified = if entry.modified_ts > 0 {
        DateTime::from_timestamp(entry.modified_ts, 0)
            .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let created = if entry.created_ts > 0 {
        DateTime::from_timestamp(entry.created_ts, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let content_type = if entry.is_dir {
        String::new()
    } else {
        let guess = mime_guess::from_path(&entry.name);
        guess
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string())
    };

    let etag = if entry.is_dir {
        format!("\"dir-{}\"", simple_hash(href))
    } else {
        format!("\"{:x}-{:x}\"", entry.size, entry.modified_ts)
    };

    DavResource {
        href: href.to_string(),
        display_name: entry.name.clone(),
        is_dir: entry.is_dir,
        size: entry.size,
        content_type,
        creation_date: created,
        last_modified: modified,
        etag,
    }
}

// ---------------------------------------------------------------------------
// Error helper
// ---------------------------------------------------------------------------

/// Convert AppError to a WebDAV-appropriate HTTP response.
fn error_to_webdav_response(err: AppError) -> Response<Body> {
    let (status, message) = match &err {
        AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
    };

    let mut w = XmlWriter::new();
    w.declaration()
        .open("D:error xmlns:D=\"DAV:\"")
        .tag("D:message", &message)
        .close("D:error");

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("DAV", DAV_HEADER)
        .body(Body::from(w.finish()))
        .expect("building error response")
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Simple hash function for generating etag values. Not cryptographic.
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Helper: build DavResource with defaults ───

    fn make_resource(href: &str, name: &str, is_dir: bool) -> DavResource {
        DavResource {
            href: href.to_string(),
            display_name: name.to_string(),
            is_dir,
            size: 0,
            content_type: String::new(),
            creation_date: String::new(),
            last_modified: String::new(),
            etag: String::new(),
        }
    }

    fn make_dir_entry(name: &str, is_dir: bool, size: u64, created_ts: i64, modified_ts: i64) -> directory::DirEntry {
        directory::DirEntry {
            name: name.to_string(),
            is_dir,
            size,
            size_display: "-".to_string(),
            created: String::new(),
            modified: String::new(),
            created_ts,
            modified_ts,
            icon: String::new(),
            href: format!("/{}", name),
            media_type: if is_dir { "directory" } else { "other" }.to_string(),
        }
    }

    // ─── xml_escape ───

    #[test]
    fn xml_escape_cases() {
        let cases = [
            ("a&b",              "a&amp;b"),
            ("<tag>",            "&lt;tag&gt;"),
            ("he said \"hi\"",   "he said &quot;hi&quot;"),
            ("it's",             "it&apos;s"),
            ("hello world",      "hello world"),
        ];
        for (input, expected) in cases {
            assert_eq!(xml_escape(input), expected, "xml_escape({:?})", input);
        }
    }

    // ─── parse_depth ───

    #[test]
    fn parse_depth_cases() {
        // (header_value, expected) — None means missing header
        let cases: Vec<(Option<&str>, u32)> = vec![
            (Some("0"),        0),
            (Some("1"),        1),
            (Some("infinity"), 1),
            (None,             1),
        ];
        for (header_val, expected) in cases {
            let mut headers = HeaderMap::new();
            if let Some(v) = header_val {
                headers.insert("Depth", v.parse().unwrap());
            }
            assert_eq!(parse_depth(&headers), expected, "Depth: {:?}", header_val);
        }
    }

    // ─── XmlWriter ───

    #[test]
    fn xml_writer_methods() {
        // declaration
        let mut w = XmlWriter::new();
        w.declaration();
        assert_eq!(w.finish(), "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");

        // tag with escaping
        let mut w = XmlWriter::new();
        w.tag("name", "a&b<c>");
        assert_eq!(w.finish(), "<name>a&amp;b&lt;c&gt;</name>\n");

        // tag_if: empty → skipped, non-empty → rendered
        let mut w = XmlWriter::new();
        w.tag_if("a", "").tag_if("b", "value");
        assert_eq!(w.finish(), "<b>value</b>\n");

        // open + close nesting
        let mut w = XmlWriter::new();
        w.open("parent").tag("child", "text").close("parent");
        assert_eq!(w.finish(), "<parent>\n<child>text</child>\n</parent>\n");

        // open_attr
        let mut w = XmlWriter::new();
        w.open_attr("root", "xmlns:D=\"DAV:\"");
        assert_eq!(w.finish(), "<root xmlns:D=\"DAV:\">\n");

        // empty
        let mut w = XmlWriter::new();
        w.empty("D:resourcetype");
        assert_eq!(w.finish(), "<D:resourcetype/>\n");

        // raw passthrough
        let mut w = XmlWriter::new();
        w.raw("<already>escaped</already>\n");
        assert_eq!(w.finish(), "<already>escaped</already>\n");
    }

    // ─── DavResource::to_xml ───

    #[test]
    fn dav_resource_directory_xml() {
        let mut res = make_resource("/test/", "test", true);
        res.creation_date = "2024-01-01T00:00:00Z".into();
        res.last_modified = "Mon, 01 Jan 2024 00:00:00 GMT".into();
        res.etag = "\"dir-123\"".into();

        let xml = res.to_xml();
        assert!(xml.contains("<D:collection/>"));
        assert!(xml.contains("<D:displayname>test</D:displayname>"));
        assert!(xml.contains("<D:href>/test/</D:href>"));
        assert!(!xml.contains("<D:getcontentlength>"));
        assert!(!xml.contains("<D:getcontenttype>"));
        assert!(xml.contains("<D:creationdate>"));
        assert!(xml.contains("<D:getetag>"));
        assert!(xml.contains("<D:supportedlock>"));
    }

    #[test]
    fn dav_resource_file_xml() {
        let mut res = make_resource("/file.txt", "file.txt", false);
        res.size = 1024;
        res.content_type = "text/plain".into();
        res.creation_date = "2024-01-01T00:00:00Z".into();
        res.last_modified = "Mon, 01 Jan 2024 00:00:00 GMT".into();
        res.etag = "\"abc\"".into();

        let xml = res.to_xml();
        assert!(xml.contains("<D:resourcetype/>\n"));
        assert!(xml.contains("<D:getcontentlength>1024</D:getcontentlength>"));
        assert!(xml.contains("<D:getcontenttype>text/plain</D:getcontenttype>"));
        assert!(xml.contains("<D:displayname>file.txt</D:displayname>"));
        assert!(xml.contains("<D:creationdate>"));
        assert!(xml.contains("<D:getetag>"));
    }

    #[test]
    fn dav_resource_escapes_special_chars() {
        let mut res = make_resource("/a&b.txt", "a&b.txt", false);
        res.size = 10;
        res.content_type = "text/plain".into();
        let xml = res.to_xml();
        assert!(xml.contains("<D:href>/a&amp;b.txt</D:href>"));
        assert!(xml.contains("<D:displayname>a&amp;b.txt</D:displayname>"));
    }

    #[test]
    fn dav_resource_skips_empty_optional_fields() {
        let xml = make_resource("/test", "test", false).to_xml();
        assert!(!xml.contains("<D:getcontenttype>"));
        assert!(!xml.contains("<D:creationdate>"));
        assert!(!xml.contains("<D:getlastmodified>"));
        assert!(!xml.contains("<D:getetag>"));
    }

    // ─── entry_to_resource ───

    #[test]
    fn entry_to_resource_dir() {
        let entry = make_dir_entry("photos", true, 0, 1704067200, 1704067200);
        let xml = entry_to_resource(&entry, "/photos/").to_xml();
        assert!(xml.contains("<D:collection/>"));
        assert!(xml.contains("<D:href>/photos/</D:href>"));
        assert!(xml.contains("<D:creationdate>"));
        assert!(xml.contains("<D:supportedlock>"));
    }

    #[test]
    fn entry_to_resource_file() {
        let entry = make_dir_entry("readme.txt", false, 256, 0, 1704067200);
        let xml = entry_to_resource(&entry, "/readme.txt").to_xml();
        assert!(xml.contains("<D:resourcetype/>"));
        assert!(xml.contains("<D:getcontentlength>256</D:getcontentlength>"));
        assert!(xml.contains("text/plain"));
        assert!(xml.contains("<D:getetag>"));
    }

    // ─── Utilities ───

    #[test]
    fn simple_hash_deterministic() {
        assert_eq!(simple_hash("/test/"), simple_hash("/test/"));
        assert_ne!(simple_hash("/a/"), simple_hash("/b/"));
    }

    // ─── Response handlers ───

    #[test]
    fn handle_options_response() {
        let resp = handle_options();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("DAV").unwrap(), DAV_HEADER);
        let allow = resp.headers().get("Allow").unwrap().to_str().unwrap();
        assert!(allow.contains("LOCK"));
        assert!(allow.contains("UNLOCK"));
    }

    #[test]
    fn handle_lock_response() {
        let resp = handle_lock(&HeaderMap::new());
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get("Lock-Token").is_some());
    }

    #[test]
    fn handle_unlock_response() {
        assert_eq!(handle_unlock().status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn base64_decode_valid() {
        // "admin:secret" -> "YWRtaW46c2VjcmV0"
        assert_eq!(base64_decode("YWRtaW46c2VjcmV0"), Some("admin:secret".to_string()));
        // "user:pass" -> "dXNlcjpwYXNz"
        assert_eq!(base64_decode("dXNlcjpwYXNz"), Some("user:pass".to_string()));
        // empty
        assert_eq!(base64_decode(""), Some("".to_string()));
    }

    #[test]
    fn parse_overwrite_header() {
        let mut headers = HeaderMap::new();
        assert!(parse_overwrite(&headers)); // missing → true

        headers.insert("Overwrite", "T".parse().unwrap());
        assert!(parse_overwrite(&headers));

        headers.insert("Overwrite", "F".parse().unwrap());
        assert!(!parse_overwrite(&headers));
    }
}
