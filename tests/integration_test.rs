use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::routing::get;
use axum::Router;
use http_body_util::BodyExt;
use std::fs;
use std::sync::Arc;
use tower::ServiceExt;

use echofs::handlers::{self, AppState};

/// Build a test router without logging middleware.
fn test_app(root: std::path::PathBuf) -> Router {
    test_app_with_options(root, false, -1)
}

/// Build a test router with configurable options.
fn test_app_with_options(root: std::path::PathBuf, show_hidden: bool, max_depth: i32) -> Router {
    let state = Arc::new(AppState { root, show_hidden, max_depth, speed_limit: None, webdav: false });
    Router::new()
        .route("/", get(handlers::serve_index))
        .route("/{*path}", get(handlers::serve_path))
        .with_state(state)
}

async fn body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

// ─── JSON API (via X-Requested-With header) ───

#[tokio::test]
async fn api_root_json_structure() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("file.txt"), "hello").unwrap();
    fs::create_dir(root.join("dir")).unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["path"].is_string());
    assert!(json["breadcrumbs"].is_array());
    assert!(json["entries"].is_array());
}

#[tokio::test]
async fn api_subdir_json() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/inner.txt"), "data").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/sub")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "inner.txt");
}

#[tokio::test]
async fn api_hidden_files_excluded() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".hidden"), "secret").unwrap();
    fs::write(root.join("visible.txt"), "ok").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "visible.txt");
}

#[tokio::test]
async fn api_dirs_sorted_before_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("afile.txt"), "a").unwrap();
    fs::create_dir(root.join("zdir")).unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert!(entries[0]["is_dir"].as_bool().unwrap());
    assert!(!entries[1]["is_dir"].as_bool().unwrap());
}

#[tokio::test]
async fn api_nonexistent_dir_404() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/nonexistent")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn api_entry_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("test.txt"), "content").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entry = &json["entries"][0];
    assert!(entry["name"].is_string());
    assert!(entry["is_dir"].is_boolean());
    assert!(entry["size"].is_number());
    assert!(entry["size_display"].is_string());
    assert!(entry["icon"].is_string());
    assert!(entry["href"].is_string());
    assert!(entry["media_type"].is_string());
}

#[tokio::test]
async fn api_breadcrumbs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("a/b")).unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/a/b")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let crumbs = json["breadcrumbs"].as_array().unwrap();
    assert_eq!(crumbs.len(), 3);
    assert_eq!(crumbs[0]["name"], "Home");
    assert_eq!(crumbs[1]["name"], "a");
    assert_eq!(crumbs[2]["name"], "b");
}

// ─── AJAX dispatch: same path returns HTML or JSON ───

#[tokio::test]
async fn root_without_xhr_returns_html_with_xhr_returns_json() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("file.txt"), "hello").unwrap();
    let app = test_app(root.clone());

    // Without XHR header → HTML
    let resp = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<!DOCTYPE html>"));

    // With XHR header → JSON
    let app = test_app(root);
    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["entries"].is_array());
}

#[tokio::test]
async fn subdir_without_xhr_returns_html_with_xhr_returns_json() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("mydir")).unwrap();
    fs::write(root.join("mydir/test.txt"), "data").unwrap();
    let app = test_app(root.clone());

    // Without XHR header → HTML
    let resp = app
        .oneshot(Request::get("/mydir").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<!DOCTYPE html>"));

    // With XHR header → JSON
    let app = test_app(root);
    let resp = app
        .oneshot(
            Request::get("/mydir")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "test.txt");
}

// ─── File serving ───

#[tokio::test]
async fn serve_file_full_content() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("hello.txt"), "Hello, world!").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(Request::get("/hello.txt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(header::ACCEPT_RANGES)
            .unwrap()
            .to_str()
            .unwrap(),
        "bytes"
    );
    assert_eq!(
        resp.headers()
            .get(header::CONTENT_LENGTH)
            .unwrap()
            .to_str()
            .unwrap(),
        "13"
    );
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("text/plain"));
    let body = body_string(resp.into_body()).await;
    assert_eq!(body, "Hello, world!");
}

#[tokio::test]
async fn serve_file_range_206() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("data.txt"), "0123456789").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/data.txt")
                .header(header::RANGE, "bytes=0-4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
    assert!(resp.headers().get(header::CONTENT_RANGE).is_some());
    let body = body_string(resp.into_body()).await;
    assert_eq!(body, "01234");
}

#[tokio::test]
async fn serve_file_invalid_range_416() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("data.txt"), "0123456789").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/data.txt")
                .header(header::RANGE, "bytes=100-200")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE);
}

#[tokio::test]
async fn serve_file_suffix_range() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("data.txt"), "0123456789").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/data.txt")
                .header(header::RANGE, "bytes=-3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
    let body = body_string(resp.into_body()).await;
    assert_eq!(body, "789");
}

// ─── Security ───

#[tokio::test]
async fn hidden_file_direct_access_denied() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".env"), "SECRET=key").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(Request::get("/.env").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn hidden_dir_child_access_denied() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::write(root.join(".git/config"), "[core]").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(Request::get("/.git/config").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn hidden_file_percent_encoded_denied() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".env"), "SECRET=key").unwrap();
    let app = test_app(root);

    // Try percent-encoded dot: %2E = '.'
    let resp = app
        .oneshot(Request::get("/%2Eenv").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = resp.status();
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "expected 403 or 404, got {}",
        status
    );
}

#[tokio::test]
async fn path_traversal_denied() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/..%2F..%2F..%2Fetc%2Fpasswd")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "expected 403 or 404, got {}",
        status
    );
}

#[tokio::test]
async fn nonexistent_file_404() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/no-such-file.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ─── Error page HTML vs JSON ───

#[tokio::test]
async fn browser_404_returns_html_error_page() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/no-such-file.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("text/html"), "expected text/html, got {}", ct);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<!DOCTYPE html>"));
    assert!(body.contains("404"));
    assert!(body.contains("Not Found"));
    assert!(body.contains("Back to Home"));
}

#[tokio::test]
async fn xhr_404_returns_json_error() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/no-such-file.txt")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("application/json"), "expected application/json, got {}", ct);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].is_string());
}

#[tokio::test]
async fn browser_403_returns_html_error_page() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".env"), "SECRET=key").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::get("/.env")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("text/html"), "expected text/html, got {}", ct);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("403"));
    assert!(body.contains("Forbidden"));
}

// ─── HEAD method support ───

#[tokio::test]
async fn head_root_returns_ok_no_body() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::head("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.is_empty(), "HEAD response should have no body");
}

#[tokio::test]
async fn head_file_returns_headers_no_body() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("hello.txt"), "Hello, world!").unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::head("/hello.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get(header::CONTENT_TYPE).is_some());
    assert!(resp.headers().get(header::CONTENT_LENGTH).is_some());
    assert_eq!(
        resp.headers()
            .get(header::ACCEPT_RANGES)
            .unwrap()
            .to_str()
            .unwrap(),
        "bytes"
    );
    let body = body_string(resp.into_body()).await;
    assert!(body.is_empty(), "HEAD response should have no body");
}

// ─── MIME types ───

#[tokio::test]
async fn serve_png_with_correct_mime() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    // Write a minimal valid PNG (1x1 transparent pixel)
    let png_data = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, // 1x1 RGBA
        0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
        0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE5, // compressed data
        0x27, 0xDE, 0xFC, // CRC
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    fs::write(root.join("image.png"), &png_data).unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(Request::get("/image.png").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(ct, "image/png");
}

// ─── show_hidden tests ───

#[tokio::test]
async fn show_hidden_allows_dotfile_access() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".env"), "SECRET=key").unwrap();
    let app = test_app_with_options(root, true, -1);

    let resp = app
        .oneshot(Request::get("/.env").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert_eq!(body, "SECRET=key");
}

#[tokio::test]
async fn show_hidden_includes_dotfiles_in_listing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".hidden"), "secret").unwrap();
    fs::write(root.join("visible.txt"), "ok").unwrap();
    let app = test_app_with_options(root, true, -1);

    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    let names: Vec<&str> = entries.iter().map(|e| e["name"].as_str().unwrap()).collect();
    assert!(names.contains(&".hidden"));
    assert!(names.contains(&"visible.txt"));
}

#[tokio::test]
async fn show_hidden_still_blocks_path_traversal() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app_with_options(root, true, -1);

    let resp = app
        .oneshot(
            Request::get("/..%2F..%2F..%2Fetc%2Fpasswd")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "expected 403 or 404, got {}",
        status
    );
}

// ─── max_depth tests ───

#[tokio::test]
async fn max_depth_blocks_deep_directory_access() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("a/b")).unwrap();
    let app = test_app_with_options(root, false, 0);

    let resp = app
        .oneshot(Request::get("/a").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_blocks_deep_file_access() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/secret.txt"), "data").unwrap();
    let app = test_app_with_options(root, false, 0);

    let resp = app
        .oneshot(Request::get("/sub/secret.txt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_hides_subdirs_in_listing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("mydir")).unwrap();
    fs::write(root.join("file.txt"), "data").unwrap();
    let app = test_app_with_options(root, false, 0);

    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "file.txt");
    assert!(!entries[0]["is_dir"].as_bool().unwrap());
}

#[tokio::test]
async fn max_depth_allows_within_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/file.txt"), "hello").unwrap();
    fs::write(root.join("file.txt"), "root").unwrap();
    let app = test_app_with_options(root, false, 1);

    // Access to depth-1 directory should work, and root listing should include it
    let resp = app
        .oneshot(
            Request::get("/")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    let has_dir = entries.iter().any(|e| e["is_dir"].as_bool().unwrap() && e["name"] == "sub");
    assert!(has_dir, "root listing should include subdirectory when below max_depth");
}

#[tokio::test]
async fn max_depth_unlimited_allows_deep_access() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("a/b/c")).unwrap();
    fs::write(root.join("a/b/c/deep.txt"), "deep content").unwrap();
    let app = test_app_with_options(root, false, -1);

    let resp = app
        .oneshot(Request::get("/a/b/c/deep.txt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert_eq!(body, "deep content");
}

#[tokio::test]
async fn max_depth_zero_allows_root_file_access() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("hello.txt"), "hello world").unwrap();
    let app = test_app_with_options(root, false, 0);

    let resp = app
        .oneshot(Request::get("/hello.txt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert_eq!(body, "hello world");
}

#[tokio::test]
async fn max_depth_one_blocks_depth_two_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("a/b")).unwrap();
    let app = test_app_with_options(root, false, 1);

    // depth=1 allows /a (depth 1) but blocks /a/b (depth 2)
    let resp = app
        .oneshot(Request::get("/a/b").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_boundary_at_exact_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("a/b")).unwrap();
    let app = test_app_with_options(root, false, 2);

    // depth=2 should allow /a/b (depth 2) — boundary is inclusive (<=)
    let resp = app
        .oneshot(
            Request::get("/a/b")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn max_depth_one_allows_file_in_allowed_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/readme.txt"), "hello").unwrap();
    let app = test_app_with_options(root, false, 1);

    // depth=1: file in /sub (parent depth 1 <= max_depth) should be accessible
    let resp = app
        .oneshot(Request::get("/sub/readme.txt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert_eq!(body, "hello");
}

#[tokio::test]
async fn max_depth_one_blocks_file_in_deep_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("a/b")).unwrap();
    fs::write(root.join("a/b/secret.txt"), "data").unwrap();
    let app = test_app_with_options(root, false, 1);

    // depth=1: file at /a/b/secret.txt (parent depth 2 > max_depth) should be blocked
    let resp = app
        .oneshot(Request::get("/a/b/secret.txt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_listing_hides_grandchild_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("sub/child")).unwrap();
    fs::write(root.join("sub/file.txt"), "data").unwrap();
    let app = test_app_with_options(root, false, 1);

    // depth=1: listing of /sub (at depth 1 = max_depth) should hide child dirs
    let resp = app
        .oneshot(
            Request::get("/sub")
                .header("X-Requested-With", "XMLHttpRequest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "file.txt");
    assert!(!entries[0]["is_dir"].as_bool().unwrap());
}

// ─── WebDAV Integration Tests ───

/// Build a test router with WebDAV enabled.
fn test_app_webdav(root: std::path::PathBuf) -> Router {
    test_app_webdav_with_options(root, false, -1)
}

fn test_app_webdav_with_options(root: std::path::PathBuf, show_hidden: bool, max_depth: i32) -> Router {
    let state = Arc::new(AppState { root, show_hidden, max_depth, speed_limit: None, webdav: true });
    Router::new()
        .route("/", get(handlers::serve_index))
        .route("/{*path}", get(handlers::serve_path))
        .route("/", axum::routing::any(echofs::webdav::handle_webdav_root))
        .route("/{*path}", axum::routing::any(echofs::webdav::handle_webdav_path))
        .with_state(state)
}

#[tokio::test]
async fn webdav_options_returns_dav_header() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("DAV").unwrap(), "1, 2");
    assert!(resp.headers().get("Allow").unwrap().to_str().unwrap().contains("PROPFIND"));
    assert!(resp.headers().get("Allow").unwrap().to_str().unwrap().contains("LOCK"));
}

#[tokio::test]
async fn webdav_propfind_root_depth_0() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("file.txt"), "hello").unwrap();
    fs::create_dir(root.join("subdir")).unwrap();

    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/")
                .header("Depth", "0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::MULTI_STATUS);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<D:multistatus"));
    assert!(body.contains("<D:collection/>"));
    // Depth 0 should NOT include children
    assert!(!body.contains("file.txt"));
    assert!(!body.contains("subdir"));
}

#[tokio::test]
async fn webdav_propfind_root_depth_1() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("file.txt"), "hello").unwrap();
    fs::create_dir(root.join("subdir")).unwrap();

    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/")
                .header("Depth", "1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::MULTI_STATUS);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<D:multistatus"));
    // Should include both the root collection and children
    assert!(body.contains("<D:collection/>"));
    assert!(body.contains("file.txt"));
    assert!(body.contains("subdir"));
}

#[tokio::test]
async fn webdav_propfind_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("readme.txt"), "content here").unwrap();

    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/readme.txt")
                .header("Depth", "0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::MULTI_STATUS);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<D:resourcetype/>"));
    assert!(body.contains("<D:getcontentlength>12</D:getcontentlength>"));
    assert!(body.contains("text/plain"));
    assert!(body.contains("readme.txt"));
}

#[tokio::test]
async fn webdav_propfind_nonexistent_returns_404() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();

    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/nonexistent")
                .header("Depth", "0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn webdav_propfind_hidden_file_returns_403() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".secret"), "hidden").unwrap();

    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/.secret")
                .header("Depth", "0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn webdav_propfind_hidden_file_allowed_with_show_hidden() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join(".secret"), "hidden").unwrap();

    let app = test_app_webdav_with_options(root, true, -1);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/.secret")
                .header("Depth", "0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::MULTI_STATUS);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains(".secret"));
}

#[tokio::test]
async fn webdav_propfind_subdirectory_depth_1() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(root.join("docs/a.txt"), "aaa").unwrap();
    fs::write(root.join("docs/b.txt"), "bbb").unwrap();

    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/docs")
                .header("Depth", "1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::MULTI_STATUS);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("a.txt"));
    assert!(body.contains("b.txt"));
    assert!(body.contains("<D:collection/>"));
}

#[tokio::test]
async fn webdav_propfind_without_webdav_flag_returns_405() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();

    // Use the non-webdav test app
    let app = test_app(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/")
                .header("Depth", "0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn webdav_options_on_subpath() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("folder")).unwrap();

    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/folder")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("DAV").unwrap(), "1, 2");
}

#[tokio::test]
async fn webdav_lock_returns_lock_token() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("LOCK")
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get("Lock-Token").is_some());
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<D:lockdiscovery>"));
    assert!(body.contains("<D:locktoken>"));
}

#[tokio::test]
async fn webdav_unlock_returns_no_content() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("UNLOCK")
                .uri("/")
                .header("Lock-Token", "<opaquelocktoken:test>")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn webdav_put_returns_forbidden() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/newfile.txt")
                .body(Body::from("data"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn webdav_delete_returns_forbidden() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("file.txt"), "data").unwrap();
    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/file.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn webdav_propfind_includes_supportedlock() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::write(root.join("file.txt"), "hello").unwrap();
    let app = test_app_webdav(root);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/")
                .header("Depth", "1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::MULTI_STATUS);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<D:supportedlock>"));
    assert!(body.contains("<D:getetag>"));
    assert!(body.contains("<D:creationdate>") || body.contains("<D:getlastmodified>"));
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("a/b")).unwrap();
    fs::write(root.join("a/b/deep.txt"), "data").unwrap();

    // max_depth = 0 means root only
    let app = test_app_webdav_with_options(root, false, 0);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PROPFIND")
                .uri("/a")
                .header("Depth", "0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Depth 1 dir "a" should be rejected by max_depth 0
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
