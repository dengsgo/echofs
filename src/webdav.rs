use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, Response, StatusCode, header};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::Arc;

use crate::directory;
use crate::error::AppError;
use crate::handlers::AppState;
use crate::mime_utils;

const DAV_HEADER: &str = "1, 2";
const ALLOW_METHODS: &str = "OPTIONS, GET, HEAD, PROPFIND, LOCK, UNLOCK";

/// Unified WebDAV handler for the root path `/`.
/// Dispatches PROPFIND, OPTIONS, LOCK, UNLOCK; rejects unsupported write methods.
pub async fn handle_webdav_root(
    method: Method,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response<Body> {
    match method.as_str() {
        "PROPFIND" => handle_propfind_inner(&state, "", &headers).await,
        "OPTIONS" => handle_options(),
        "LOCK" => handle_lock(&headers),
        "UNLOCK" => handle_unlock(),
        "PUT" | "DELETE" | "MKCOL" | "PROPPATCH" | "COPY" | "MOVE" => read_only_response(),
        _ => method_not_allowed(),
    }
}

/// Unified WebDAV handler for `/{*path}`.
pub async fn handle_webdav_path(
    method: Method,
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    match method.as_str() {
        "PROPFIND" => {
            let rel_path = percent_encoding::percent_decode_str(&path)
                .decode_utf8_lossy()
                .to_string();
            handle_propfind_inner(&state, &rel_path, &headers).await
        }
        "OPTIONS" => handle_options(),
        "LOCK" => handle_lock(&headers),
        "UNLOCK" => handle_unlock(),
        "PUT" | "DELETE" | "MKCOL" | "PROPPATCH" | "COPY" | "MOVE" => read_only_response(),
        _ => method_not_allowed(),
    }
}

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
    // Extract the requested timeout or use a default
    let timeout = headers
        .get("Timeout")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Second-3600");

    // Generate a deterministic fake lock token
    let lock_token = "opaquelocktoken:echofs-readonly-lock";

    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <D:prop xmlns:D=\"DAV:\">\n\
         <D:lockdiscovery>\n\
         <D:activelock>\n\
         <D:locktype><D:write/></D:locktype>\n\
         <D:lockscope><D:exclusive/></D:lockscope>\n\
         <D:depth>infinity</D:depth>\n\
         <D:owner><D:href>anonymous</D:href></D:owner>\n\
         <D:timeout>{}</D:timeout>\n\
         <D:locktoken><D:href>{}</D:href></D:locktoken>\n\
         <D:lockroot><D:href>/</D:href></D:lockroot>\n\
         </D:activelock>\n\
         </D:lockdiscovery>\n\
         </D:prop>\n",
        xml_escape(timeout),
        lock_token,
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("Lock-Token", format!("<{}>", lock_token))
        .header("DAV", DAV_HEADER)
        .body(Body::from(body))
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

/// Return 403 Forbidden for write operations on this read-only server.
fn read_only_response() -> Response<Body> {
    let body = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
                <D:error xmlns:D=\"DAV:\">\n\
                <D:message>This is a read-only WebDAV server</D:message>\n\
                </D:error>\n";

    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("DAV", DAV_HEADER)
        .body(Body::from(body))
        .expect("building read-only response")
}

/// Return 405 Method Not Allowed for unsupported methods.
fn method_not_allowed() -> Response<Body> {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header(header::CONTENT_LENGTH, "0")
        .body(Body::empty())
        .expect("building 405 response")
}

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

    let mut responses = Vec::new();

    if resolved.is_dir() {
        // Add the directory itself
        let dir_href = if rel_path.is_empty() { "/".to_string() } else { format!("/{}/", rel_path.trim_start_matches('/')) };
        match dir_resource_props(&resolved, &dir_href).await {
            Ok(xml) => responses.push(xml),
            Err(e) => return error_to_webdav_response(e),
        }

        // If Depth >= 1, add children
        if depth >= 1 {
            match directory::list_directory(&state.root, rel_path, state.show_hidden, state.max_depth).await {
                Ok(listing) => {
                    for entry in &listing.entries {
                        let child_href = if entry.is_dir {
                            format!("{}/", entry.href)
                        } else {
                            entry.href.clone()
                        };
                        responses.push(entry_resource_xml(entry, &child_href));
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
            Ok(xml) => responses.push(xml),
            Err(e) => return error_to_webdav_response(e),
        }
    } else {
        return error_to_webdav_response(AppError::NotFound("Path not found".into()));
    }

    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <D:multistatus xmlns:D=\"DAV:\">\n\
         {}\
         </D:multistatus>\n",
        responses.join("")
    );

    Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("DAV", DAV_HEADER)
        .body(Body::from(body))
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
            _ => 1, // "infinity" and other values → treat as 1
        })
        .unwrap_or(1)
}

/// Build `<D:response>` XML for a directory from filesystem metadata.
async fn dir_resource_props(path: &PathBuf, href: &str) -> Result<String, AppError> {
    let path = path.clone();
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

        // Use inode or path hash as etag for directories
        let etag = format!("\"dir-{}\"", simple_hash(&href));

        Ok(format_response_xml(
            &href,
            &xml_escape(&name),
            true,
            0,
            &created,
            &modified,
            "",
            &etag,
        ))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// Build `<D:response>` XML for a file from filesystem metadata.
async fn file_resource_props(path: &PathBuf, href: &str) -> Result<String, AppError> {
    let path = path.clone();
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

        // Etag based on size + modified time
        let etag = format!("\"{:x}-{:x}\"", meta.len(), modified_ts);

        Ok(format_response_xml(
            &href,
            &xml_escape(&name),
            false,
            meta.len(),
            &created,
            &modified,
            &mime.to_string(),
            &etag,
        ))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// Build `<D:response>` XML from a `DirEntry` (used for directory children).
fn entry_resource_xml(entry: &directory::DirEntry, href: &str) -> String {
    // Use the entry's modified timestamp to build an RFC 2822 date
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

    format_response_xml(
        href,
        &xml_escape(&entry.name),
        entry.is_dir,
        entry.size,
        &created,
        &modified,
        &content_type,
        &etag,
    )
}

/// Format a single `<D:response>` XML block.
fn format_response_xml(
    href: &str,
    display_name: &str,
    is_dir: bool,
    size: u64,
    creation_date: &str,
    last_modified: &str,
    content_type: &str,
    etag: &str,
) -> String {
    let resource_type = if is_dir {
        "<D:resourcetype><D:collection/></D:resourcetype>"
    } else {
        "<D:resourcetype/>"
    };

    let content_length = if is_dir {
        String::new()
    } else {
        format!("<D:getcontentlength>{}</D:getcontentlength>\n", size)
    };

    let content_type_xml = if content_type.is_empty() {
        String::new()
    } else {
        format!(
            "<D:getcontenttype>{}</D:getcontenttype>\n",
            xml_escape(content_type)
        )
    };

    let last_modified_xml = if last_modified.is_empty() {
        String::new()
    } else {
        format!(
            "<D:getlastmodified>{}</D:getlastmodified>\n",
            xml_escape(last_modified)
        )
    };

    let creation_date_xml = if creation_date.is_empty() {
        String::new()
    } else {
        format!(
            "<D:creationdate>{}</D:creationdate>\n",
            xml_escape(creation_date)
        )
    };

    let etag_xml = if etag.is_empty() {
        String::new()
    } else {
        format!("<D:getetag>{}</D:getetag>\n", xml_escape(etag))
    };

    // supportedlock tells clients (Finder) that the server understands locking
    let supported_lock = "\
        <D:supportedlock>\n\
        <D:lockentry>\n\
        <D:lockscope><D:exclusive/></D:lockscope>\n\
        <D:locktype><D:write/></D:locktype>\n\
        </D:lockentry>\n\
        </D:supportedlock>\n";

    format!(
        "<D:response>\n\
         <D:href>{}</D:href>\n\
         <D:propstat>\n\
         <D:prop>\n\
         <D:displayname>{}</D:displayname>\n\
         {}\
         {}\
         {}\
         {}\
         {}\
         {}\
         {}\
         </D:prop>\n\
         <D:status>HTTP/1.1 200 OK</D:status>\n\
         </D:propstat>\n\
         </D:response>\n",
        xml_escape(href),
        display_name,
        resource_type,
        content_length,
        content_type_xml,
        creation_date_xml,
        last_modified_xml,
        etag_xml,
        supported_lock,
    )
}

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

/// Convert AppError to a WebDAV-appropriate HTTP response.
fn error_to_webdav_response(err: AppError) -> Response<Body> {
    let (status, message) = match &err {
        AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
    };

    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <D:error xmlns:D=\"DAV:\">\n\
         <D:message>{}</D:message>\n\
         </D:error>\n",
        xml_escape(&message)
    );

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("DAV", DAV_HEADER)
        .body(Body::from(body))
        .expect("building error response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape_ampersand() {
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_xml_escape_lt_gt() {
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_xml_escape_quotes() {
        assert_eq!(xml_escape("he said \"hi\""), "he said &quot;hi&quot;");
        assert_eq!(xml_escape("it's"), "it&apos;s");
    }

    #[test]
    fn test_xml_escape_no_change() {
        assert_eq!(xml_escape("hello world"), "hello world");
    }

    #[test]
    fn test_parse_depth_zero() {
        let mut headers = HeaderMap::new();
        headers.insert("Depth", "0".parse().unwrap());
        assert_eq!(parse_depth(&headers), 0);
    }

    #[test]
    fn test_parse_depth_one() {
        let mut headers = HeaderMap::new();
        headers.insert("Depth", "1".parse().unwrap());
        assert_eq!(parse_depth(&headers), 1);
    }

    #[test]
    fn test_parse_depth_infinity() {
        let mut headers = HeaderMap::new();
        headers.insert("Depth", "infinity".parse().unwrap());
        assert_eq!(parse_depth(&headers), 1);
    }

    #[test]
    fn test_parse_depth_missing() {
        let headers = HeaderMap::new();
        assert_eq!(parse_depth(&headers), 1);
    }

    #[test]
    fn test_format_response_xml_directory() {
        let xml = format_response_xml("/test/", "test", true, 0, "2024-01-01T00:00:00Z", "Mon, 01 Jan 2024 00:00:00 GMT", "", "\"dir-123\"");
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
    fn test_format_response_xml_file() {
        let xml = format_response_xml("/file.txt", "file.txt", false, 1024, "2024-01-01T00:00:00Z", "Mon, 01 Jan 2024 00:00:00 GMT", "text/plain", "\"abc\"");
        assert!(xml.contains("<D:resourcetype/>"));
        assert!(xml.contains("<D:getcontentlength>1024</D:getcontentlength>"));
        assert!(xml.contains("<D:getcontenttype>text/plain</D:getcontenttype>"));
        assert!(xml.contains("<D:displayname>file.txt</D:displayname>"));
        assert!(xml.contains("<D:creationdate>"));
        assert!(xml.contains("<D:getetag>"));
    }

    #[test]
    fn test_format_response_xml_escapes_name() {
        let xml = format_response_xml("/a&b.txt", "a&amp;b.txt", false, 10, "", "", "text/plain", "");
        assert!(xml.contains("<D:href>/a&amp;b.txt</D:href>"));
        assert!(xml.contains("<D:displayname>a&amp;b.txt</D:displayname>"));
    }

    #[test]
    fn test_entry_resource_xml_dir() {
        let entry = directory::DirEntry {
            name: "photos".to_string(),
            is_dir: true,
            size: 0,
            size_display: "-".to_string(),
            created: String::new(),
            modified: String::new(),
            created_ts: 1704067200, // 2024-01-01
            modified_ts: 1704067200,
            icon: String::new(),
            href: "/photos".to_string(),
            media_type: "directory".to_string(),
        };
        let xml = entry_resource_xml(&entry, "/photos/");
        assert!(xml.contains("<D:collection/>"));
        assert!(xml.contains("<D:href>/photos/</D:href>"));
        assert!(xml.contains("<D:creationdate>"));
        assert!(xml.contains("<D:supportedlock>"));
    }

    #[test]
    fn test_entry_resource_xml_file() {
        let entry = directory::DirEntry {
            name: "readme.txt".to_string(),
            is_dir: false,
            size: 256,
            size_display: "256 B".to_string(),
            created: String::new(),
            modified: String::new(),
            created_ts: 0,
            modified_ts: 1704067200,
            icon: String::new(),
            href: "/readme.txt".to_string(),
            media_type: "other".to_string(),
        };
        let xml = entry_resource_xml(&entry, "/readme.txt");
        assert!(xml.contains("<D:resourcetype/>"));
        assert!(xml.contains("<D:getcontentlength>256</D:getcontentlength>"));
        assert!(xml.contains("text/plain"));
        assert!(xml.contains("<D:getetag>"));
    }

    #[test]
    fn test_simple_hash_deterministic() {
        assert_eq!(simple_hash("/test/"), simple_hash("/test/"));
        assert_ne!(simple_hash("/a/"), simple_hash("/b/"));
    }

    #[test]
    fn test_handle_options_dav_header() {
        let resp = handle_options();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("DAV").unwrap(), DAV_HEADER);
        assert!(resp.headers().get("Allow").unwrap().to_str().unwrap().contains("LOCK"));
        assert!(resp.headers().get("Allow").unwrap().to_str().unwrap().contains("UNLOCK"));
    }

    #[test]
    fn test_handle_unlock() {
        let resp = handle_unlock();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_handle_lock() {
        let headers = HeaderMap::new();
        let resp = handle_lock(&headers);
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get("Lock-Token").is_some());
    }

    #[test]
    fn test_read_only_response() {
        let resp = read_only_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
