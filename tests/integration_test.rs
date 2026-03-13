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
    let state = Arc::new(AppState { root });
    Router::new()
        .route("/", get(handlers::serve_index))
        .route("/{*path}", get(handlers::serve_path))
        .with_state(state)
}

async fn body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

// ─── HTML serving ───

#[tokio::test]
async fn get_root_returns_html() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<!DOCTYPE html>"));
    assert!(body.contains("EchoFS"));
}

#[tokio::test]
async fn get_subdir_returns_html() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    fs::create_dir(root.join("subdir")).unwrap();
    let app = test_app(root);

    let resp = app
        .oneshot(Request::get("/subdir").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("<!DOCTYPE html>"));
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
