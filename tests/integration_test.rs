use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::routing::get;
use axum::Router;
use http_body_util::BodyExt;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

use echofs::handlers::{self, AppState};

// ═══════════════════════════════════════════════════════════════════════════
// Test helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Test environment: creates a temp directory and provides helpers
/// for building routers and making requests.
struct TestEnv {
    root: PathBuf,
    show_hidden: bool,
    max_depth: i32,
    webdav: bool,
    webdav_user: Option<String>,
    webdav_pass: Option<String>,
    _tmp: tempfile::TempDir, // prevent cleanup until TestEnv is dropped
}

impl TestEnv {
    /// Create a new test environment with default options.
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        Self { root, show_hidden: false, max_depth: -1, webdav: false, webdav_user: None, webdav_pass: None, _tmp: tmp }
    }

    fn show_hidden(mut self) -> Self {
        self.show_hidden = true;
        self
    }

    fn max_depth(mut self, d: i32) -> Self {
        self.max_depth = d;
        self
    }

    fn webdav(mut self) -> Self {
        self.webdav = true;
        self
    }

    fn auth(mut self, user: &str, pass: &str) -> Self {
        self.webdav_user = Some(user.to_string());
        self.webdav_pass = Some(pass.to_string());
        self
    }

    /// Write a file relative to root (creates parent dirs as needed).
    fn write(&self, path: &str, content: &str) -> &Self {
        let full = self.root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
        self
    }

    /// Write raw bytes to a file.
    fn write_bytes(&self, path: &str, content: &[u8]) -> &Self {
        let full = self.root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
        self
    }

    /// Create a directory (including parents) relative to root.
    fn mkdir(&self, path: &str) -> &Self {
        fs::create_dir_all(self.root.join(path)).unwrap();
        self
    }

    /// Build a Router from the current configuration.
    fn router(&self) -> Router {
        let state = Arc::new(AppState {
            root: self.root.clone(),
            show_hidden: self.show_hidden,
            max_depth: self.max_depth,
            speed_limit: None,
            webdav: self.webdav,
            webdav_user: self.webdav_user.clone(),
            webdav_pass: self.webdav_pass.clone(),
        });
        let mut router = Router::new()
            .route("/", get(handlers::serve_index))
            .route("/{*path}", get(handlers::serve_path));
        if self.webdav {
            router = router
                .route("/", axum::routing::any(echofs::webdav::handle_webdav_root))
                .route("/{*path}", axum::routing::any(echofs::webdav::handle_webdav_path));
        }
        router.with_state(state)
    }

    /// Send a GET request and return the response.
    async fn get(&self, uri: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a GET request with XHR header (returns JSON from server).
    async fn xhr(&self, uri: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(
                Request::get(uri)
                    .header("X-Requested-With", "XMLHttpRequest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a HEAD request.
    async fn head(&self, uri: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(Request::head(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a GET request with a Range header.
    async fn get_range(&self, uri: &str, range: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(
                Request::get(uri)
                    .header(header::RANGE, range)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a PROPFIND request with a Depth header.
    async fn propfind(&self, uri: &str, depth: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(
                Request::builder()
                    .method("PROPFIND")
                    .uri(uri)
                    .header("Depth", depth)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a request with an arbitrary method.
    async fn method(&self, method: &str, uri: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a request with an arbitrary method and body.
    async fn method_with_body(&self, method: &str, uri: &str, body: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a LOCK request.
    async fn lock(&self, uri: &str) -> TestResponse {
        self.method("LOCK", uri).await
    }

    /// Send an UNLOCK request with a Lock-Token header.
    async fn unlock(&self, uri: &str) -> TestResponse {
        let resp = self
            .router()
            .oneshot(
                Request::builder()
                    .method("UNLOCK")
                    .uri(uri)
                    .header("Lock-Token", "<opaquelocktoken:test>")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send a request with arbitrary method, body, and Basic Auth.
    async fn authed(&self, method: &str, uri: &str, body: &str, user: &str, pass: &str) -> TestResponse {
        use std::fmt::Write;
        // Simple base64 encode for "user:pass"
        let credentials = format!("{}:{}", user, pass);
        let encoded = simple_base64_encode(credentials.as_bytes());
        let auth_value = format!("Basic {}", encoded);
        let resp = self
            .router()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("Authorization", &auth_value)
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        TestResponse(resp)
    }

    /// Send an authenticated request with extra headers.
    async fn authed_with_headers(&self, method: &str, uri: &str, body: &str, user: &str, pass: &str, extra_headers: Vec<(&str, &str)>) -> TestResponse {
        let credentials = format!("{}:{}", user, pass);
        let encoded = simple_base64_encode(credentials.as_bytes());
        let auth_value = format!("Basic {}", encoded);
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("Authorization", &auth_value);
        for (k, v) in extra_headers {
            builder = builder.header(k, v);
        }
        let resp = self
            .router()
            .oneshot(builder.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();
        TestResponse(resp)
    }
}

/// Simple base64 encoder for test auth headers.
fn simple_base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        let remaining = data.len() - i;
        result.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if remaining > 1 {
            result.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if remaining > 2 {
            result.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

/// Wrapper around an HTTP response providing convenient assertion methods.
struct TestResponse(axum::http::Response<Body>);

impl TestResponse {
    fn status(&self) -> StatusCode {
        self.0.status()
    }

    fn assert_status(self, expected: StatusCode) -> Self {
        assert_eq!(self.0.status(), expected, "unexpected status code");
        self
    }

    fn header(&self, name: &str) -> Option<String> {
        self.0.headers().get(name).map(|v| v.to_str().unwrap().to_string())
    }

    fn assert_header(self, name: &str, expected: &str) -> Self {
        let val = self.header(name).unwrap_or_else(|| panic!("missing header: {}", name));
        assert_eq!(val, expected, "header {} mismatch", name);
        self
    }

    fn assert_header_contains(self, name: &str, substr: &str) -> Self {
        let val = self.header(name).unwrap_or_else(|| panic!("missing header: {}", name));
        assert!(val.contains(substr), "header {} = {:?} doesn't contain {:?}", name, val, substr);
        self
    }

    fn assert_header_exists(self, name: &str) -> Self {
        assert!(self.0.headers().get(name).is_some(), "missing header: {}", name);
        self
    }

    /// Consume response and return body as string.
    async fn text(self) -> String {
        let bytes = self.0.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    /// Consume response and return parsed JSON.
    async fn json(self) -> serde_json::Value {
        let text = self.text().await;
        serde_json::from_str(&text).unwrap()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// JSON API (via X-Requested-With header)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn api_root_json_structure() {
    let env = TestEnv::new();
    env.write("file.txt", "hello");
    env.mkdir("dir");

    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
    assert!(json["path"].is_string());
    assert!(json["breadcrumbs"].is_array());
    assert!(json["entries"].is_array());
}

#[tokio::test]
async fn api_subdir_json() {
    let env = TestEnv::new();
    env.write("sub/inner.txt", "data");

    let json = env.xhr("/sub").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "inner.txt");
}

#[tokio::test]
async fn api_hidden_files_excluded() {
    let env = TestEnv::new();
    env.write(".hidden", "secret");
    env.write("visible.txt", "ok");

    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "visible.txt");
}

#[tokio::test]
async fn api_dirs_sorted_before_files() {
    let env = TestEnv::new();
    env.write("afile.txt", "a");
    env.mkdir("zdir");

    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert!(entries[0]["is_dir"].as_bool().unwrap());
    assert!(!entries[1]["is_dir"].as_bool().unwrap());
}

#[tokio::test]
async fn api_nonexistent_dir_404() {
    let env = TestEnv::new();
    env.xhr("/nonexistent").await.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn api_entry_fields() {
    let env = TestEnv::new();
    env.write("test.txt", "content");

    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
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
    let env = TestEnv::new();
    env.mkdir("a/b");

    let json = env.xhr("/a/b").await.assert_status(StatusCode::OK).json().await;
    let crumbs = json["breadcrumbs"].as_array().unwrap();
    assert_eq!(crumbs.len(), 3);
    assert_eq!(crumbs[0]["name"], "Home");
    assert_eq!(crumbs[1]["name"], "a");
    assert_eq!(crumbs[2]["name"], "b");
}

// ═══════════════════════════════════════════════════════════════════════════
// AJAX dispatch: same path returns HTML or JSON
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn root_without_xhr_returns_html_with_xhr_returns_json() {
    let env = TestEnv::new();
    env.write("file.txt", "hello");

    // Without XHR header → HTML
    let body = env.get("/").await.assert_status(StatusCode::OK).text().await;
    assert!(body.contains("<!DOCTYPE html>"));

    // With XHR header → JSON
    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
    assert!(json["entries"].is_array());
}

#[tokio::test]
async fn subdir_without_xhr_returns_html_with_xhr_returns_json() {
    let env = TestEnv::new();
    env.write("mydir/test.txt", "data");

    // Without XHR header → HTML
    let body = env.get("/mydir").await.assert_status(StatusCode::OK).text().await;
    assert!(body.contains("<!DOCTYPE html>"));

    // With XHR header → JSON
    let json = env.xhr("/mydir").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "test.txt");
}

// ═══════════════════════════════════════════════════════════════════════════
// File serving
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn serve_file_full_content() {
    let env = TestEnv::new();
    env.write("hello.txt", "Hello, world!");

    let resp = env.get("/hello.txt").await.assert_status(StatusCode::OK);
    let resp = resp
        .assert_header("accept-ranges", "bytes")
        .assert_header("content-length", "13")
        .assert_header_contains("content-type", "text/plain");
    assert_eq!(resp.text().await, "Hello, world!");
}

#[tokio::test]
async fn serve_file_range_206() {
    let env = TestEnv::new();
    env.write("data.txt", "0123456789");

    let resp = env.get_range("/data.txt", "bytes=0-4").await;
    let resp = resp.assert_status(StatusCode::PARTIAL_CONTENT).assert_header_exists("content-range");
    assert_eq!(resp.text().await, "01234");
}

#[tokio::test]
async fn serve_file_invalid_range_416() {
    let env = TestEnv::new();
    env.write("data.txt", "0123456789");

    env.get_range("/data.txt", "bytes=100-200")
        .await
        .assert_status(StatusCode::RANGE_NOT_SATISFIABLE);
}

#[tokio::test]
async fn serve_file_suffix_range() {
    let env = TestEnv::new();
    env.write("data.txt", "0123456789");

    let resp = env.get_range("/data.txt", "bytes=-3").await.assert_status(StatusCode::PARTIAL_CONTENT);
    assert_eq!(resp.text().await, "789");
}

// ═══════════════════════════════════════════════════════════════════════════
// Security
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn hidden_file_direct_access_denied() {
    let env = TestEnv::new();
    env.write(".env", "SECRET=key");
    env.get("/.env").await.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn hidden_dir_child_access_denied() {
    let env = TestEnv::new();
    env.write(".git/config", "[core]");
    env.get("/.git/config").await.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn hidden_file_percent_encoded_denied() {
    let env = TestEnv::new();
    env.write(".env", "SECRET=key");

    let status = env.get("/%2Eenv").await.status();
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "expected 403 or 404, got {}", status
    );
}

#[tokio::test]
async fn path_traversal_denied() {
    let env = TestEnv::new();

    let status = env.get("/..%2F..%2F..%2Fetc%2Fpasswd").await.status();
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "expected 403 or 404, got {}", status
    );
}

#[tokio::test]
async fn nonexistent_file_404() {
    let env = TestEnv::new();
    env.get("/no-such-file.txt").await.assert_status(StatusCode::NOT_FOUND);
}

// ═══════════════════════════════════════════════════════════════════════════
// Error page HTML vs JSON
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn browser_404_returns_html_error_page() {
    let env = TestEnv::new();

    let resp = env.get("/no-such-file.txt").await.assert_status(StatusCode::NOT_FOUND);
    let resp = resp.assert_header_contains("content-type", "text/html");
    let body = resp.text().await;
    assert!(body.contains("<!DOCTYPE html>"));
    assert!(body.contains("404"));
    assert!(body.contains("Not Found"));
    assert!(body.contains("Back to Home"));
}

#[tokio::test]
async fn xhr_404_returns_json_error() {
    let env = TestEnv::new();

    let resp = env.xhr("/no-such-file.txt").await.assert_status(StatusCode::NOT_FOUND);
    let resp = resp.assert_header_contains("content-type", "application/json");
    let json = resp.json().await;
    assert!(json["error"].is_string());
}

#[tokio::test]
async fn browser_403_returns_html_error_page() {
    let env = TestEnv::new();
    env.write(".env", "SECRET=key");

    let resp = env.get("/.env").await.assert_status(StatusCode::FORBIDDEN);
    let resp = resp.assert_header_contains("content-type", "text/html");
    let body = resp.text().await;
    assert!(body.contains("403"));
    assert!(body.contains("Forbidden"));
}

// ═══════════════════════════════════════════════════════════════════════════
// HEAD method support
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn head_root_returns_ok_no_body() {
    let env = TestEnv::new();

    let body = env.head("/").await.assert_status(StatusCode::OK).text().await;
    assert!(body.is_empty(), "HEAD response should have no body");
}

#[tokio::test]
async fn head_file_returns_headers_no_body() {
    let env = TestEnv::new();
    env.write("hello.txt", "Hello, world!");

    let resp = env.head("/hello.txt").await.assert_status(StatusCode::OK);
    let resp = resp
        .assert_header_exists("content-type")
        .assert_header_exists("content-length")
        .assert_header("accept-ranges", "bytes");
    let body = resp.text().await;
    assert!(body.is_empty(), "HEAD response should have no body");
}

// ═══════════════════════════════════════════════════════════════════════════
// MIME types
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn serve_png_with_correct_mime() {
    let env = TestEnv::new();
    // Minimal valid PNG (1x1 transparent pixel)
    let png_data: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89,
        0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54,
        0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE5,
        0x27, 0xDE, 0xFC,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
        0xAE, 0x42, 0x60, 0x82,
    ];
    env.write_bytes("image.png", png_data);

    env.get("/image.png")
        .await
        .assert_status(StatusCode::OK)
        .assert_header("content-type", "image/png");
}

// ═══════════════════════════════════════════════════════════════════════════
// show_hidden tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn show_hidden_allows_dotfile_access() {
    let env = TestEnv::new().show_hidden();
    env.write(".env", "SECRET=key");

    let body = env.get("/.env").await.assert_status(StatusCode::OK).text().await;
    assert_eq!(body, "SECRET=key");
}

#[tokio::test]
async fn show_hidden_includes_dotfiles_in_listing() {
    let env = TestEnv::new().show_hidden();
    env.write(".hidden", "secret");
    env.write("visible.txt", "ok");

    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    let names: Vec<&str> = entries.iter().map(|e| e["name"].as_str().unwrap()).collect();
    assert!(names.contains(&".hidden"));
    assert!(names.contains(&"visible.txt"));
}

#[tokio::test]
async fn show_hidden_still_blocks_path_traversal() {
    let env = TestEnv::new().show_hidden();

    let status = env.get("/..%2F..%2F..%2Fetc%2Fpasswd").await.status();
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "expected 403 or 404, got {}", status
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// max_depth tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn max_depth_blocks_deep_directory_access() {
    let env = TestEnv::new().max_depth(0);
    env.mkdir("a/b");
    env.get("/a").await.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_blocks_deep_file_access() {
    let env = TestEnv::new().max_depth(0);
    env.write("sub/secret.txt", "data");
    env.get("/sub/secret.txt").await.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_hides_subdirs_in_listing() {
    let env = TestEnv::new().max_depth(0);
    env.mkdir("mydir");
    env.write("file.txt", "data");

    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "file.txt");
    assert!(!entries[0]["is_dir"].as_bool().unwrap());
}

#[tokio::test]
async fn max_depth_allows_within_limit() {
    let env = TestEnv::new().max_depth(1);
    env.write("sub/file.txt", "hello");
    env.write("file.txt", "root");

    let json = env.xhr("/").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    let has_dir = entries.iter().any(|e| e["is_dir"].as_bool().unwrap() && e["name"] == "sub");
    assert!(has_dir, "root listing should include subdirectory when below max_depth");
}

#[tokio::test]
async fn max_depth_unlimited_allows_deep_access() {
    let env = TestEnv::new(); // default max_depth = -1 (unlimited)
    env.write("a/b/c/deep.txt", "deep content");

    let body = env.get("/a/b/c/deep.txt").await.assert_status(StatusCode::OK).text().await;
    assert_eq!(body, "deep content");
}

#[tokio::test]
async fn max_depth_zero_allows_root_file_access() {
    let env = TestEnv::new().max_depth(0);
    env.write("hello.txt", "hello world");

    let body = env.get("/hello.txt").await.assert_status(StatusCode::OK).text().await;
    assert_eq!(body, "hello world");
}

#[tokio::test]
async fn max_depth_one_blocks_depth_two_dir() {
    let env = TestEnv::new().max_depth(1);
    env.mkdir("a/b");
    env.get("/a/b").await.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_boundary_at_exact_limit() {
    let env = TestEnv::new().max_depth(2);
    env.mkdir("a/b");

    // depth=2 should allow /a/b (depth 2) — boundary is inclusive (<=)
    env.xhr("/a/b").await.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn max_depth_one_allows_file_in_allowed_dir() {
    let env = TestEnv::new().max_depth(1);
    env.write("sub/readme.txt", "hello");

    let body = env.get("/sub/readme.txt").await.assert_status(StatusCode::OK).text().await;
    assert_eq!(body, "hello");
}

#[tokio::test]
async fn max_depth_one_blocks_file_in_deep_dir() {
    let env = TestEnv::new().max_depth(1);
    env.write("a/b/secret.txt", "data");
    env.get("/a/b/secret.txt").await.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn max_depth_listing_hides_grandchild_dirs() {
    let env = TestEnv::new().max_depth(1);
    env.mkdir("sub/child");
    env.write("sub/file.txt", "data");

    // depth=1: listing of /sub (at depth 1 = max_depth) should hide child dirs
    let json = env.xhr("/sub").await.assert_status(StatusCode::OK).json().await;
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "file.txt");
    assert!(!entries[0]["is_dir"].as_bool().unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════
// WebDAV Integration Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn webdav_options_returns_dav_header() {
    let env = TestEnv::new().webdav();

    let resp = env.method("OPTIONS", "/").await.assert_status(StatusCode::OK);
    let resp = resp.assert_header("DAV", "1, 2");
    let allow = resp.header("Allow").unwrap();
    assert!(allow.contains("PROPFIND"));
    assert!(allow.contains("LOCK"));
}

#[tokio::test]
async fn webdav_propfind_root_depth_0() {
    let env = TestEnv::new().webdav();
    env.write("file.txt", "hello");
    env.mkdir("subdir");

    let body = env.propfind("/", "0").await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains("<D:multistatus"));
    assert!(body.contains("<D:collection/>"));
    // Depth 0 should NOT include children
    assert!(!body.contains("file.txt"));
    assert!(!body.contains("subdir"));
}

#[tokio::test]
async fn webdav_propfind_root_depth_1() {
    let env = TestEnv::new().webdav();
    env.write("file.txt", "hello");
    env.mkdir("subdir");

    let body = env.propfind("/", "1").await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains("<D:multistatus"));
    assert!(body.contains("<D:collection/>"));
    assert!(body.contains("file.txt"));
    assert!(body.contains("subdir"));
}

#[tokio::test]
async fn webdav_propfind_file() {
    let env = TestEnv::new().webdav();
    env.write("readme.txt", "content here");

    let body = env.propfind("/readme.txt", "0").await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains("<D:resourcetype/>"));
    assert!(body.contains("<D:getcontentlength>12</D:getcontentlength>"));
    assert!(body.contains("text/plain"));
    assert!(body.contains("readme.txt"));
}

#[tokio::test]
async fn webdav_propfind_nonexistent_returns_404() {
    let env = TestEnv::new().webdav();
    env.propfind("/nonexistent", "0").await.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn webdav_propfind_hidden_file_returns_403() {
    let env = TestEnv::new().webdav();
    env.write(".secret", "hidden");
    env.propfind("/.secret", "0").await.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn webdav_propfind_hidden_file_allowed_with_show_hidden() {
    let env = TestEnv::new().webdav().show_hidden();
    env.write(".secret", "hidden");

    let body = env.propfind("/.secret", "0").await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains(".secret"));
}

#[tokio::test]
async fn webdav_propfind_subdirectory_depth_1() {
    let env = TestEnv::new().webdav();
    env.write("docs/a.txt", "aaa");
    env.write("docs/b.txt", "bbb");

    let body = env.propfind("/docs", "1").await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains("a.txt"));
    assert!(body.contains("b.txt"));
    assert!(body.contains("<D:collection/>"));
}

#[tokio::test]
async fn webdav_propfind_without_webdav_flag_returns_405() {
    let env = TestEnv::new(); // webdav NOT enabled
    env.propfind("/", "0").await.assert_status(StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn webdav_options_on_subpath() {
    let env = TestEnv::new().webdav();
    env.mkdir("folder");

    env.method("OPTIONS", "/folder")
        .await
        .assert_status(StatusCode::OK)
        .assert_header("DAV", "1, 2");
}

#[tokio::test]
async fn webdav_lock_returns_lock_token() {
    let env = TestEnv::new().webdav();

    let resp = env.lock("/").await.assert_status(StatusCode::OK).assert_header_exists("Lock-Token");
    let body = resp.text().await;
    assert!(body.contains("<D:lockdiscovery>"));
    assert!(body.contains("<D:locktoken>"));
}

#[tokio::test]
async fn webdav_unlock_returns_no_content() {
    let env = TestEnv::new().webdav();
    env.unlock("/").await.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn webdav_put_creates_file() {
    let env = TestEnv::new().webdav();
    env.method_with_body("PUT", "/newfile.txt", "data").await.assert_status(StatusCode::CREATED);
    // Verify file was created
    assert!(env.root.join("newfile.txt").exists());
    assert_eq!(std::fs::read_to_string(env.root.join("newfile.txt")).unwrap(), "data");
}

#[tokio::test]
async fn webdav_put_requires_auth_when_configured() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.method_with_body("PUT", "/newfile.txt", "data").await.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn webdav_delete_removes_file() {
    let env = TestEnv::new().webdav();
    env.write("file.txt", "data");
    env.method("DELETE", "/file.txt").await.assert_status(StatusCode::NO_CONTENT);
    assert!(!env.root.join("file.txt").exists());
}

#[tokio::test]
async fn webdav_delete_requires_auth_when_configured() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.write("file.txt", "data");
    env.method("DELETE", "/file.txt").await.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn webdav_propfind_includes_supportedlock() {
    let env = TestEnv::new().webdav();
    env.write("file.txt", "hello");

    let body = env.propfind("/", "1").await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains("<D:supportedlock>"));
    assert!(body.contains("<D:getetag>"));
    assert!(body.contains("<D:creationdate>") || body.contains("<D:getlastmodified>"));

    // Also test max_depth enforcement via WebDAV
    let env2 = TestEnv::new().webdav().max_depth(0);
    env2.mkdir("a/b");
    env2.write("a/b/deep.txt", "data");
    env2.propfind("/a", "0").await.assert_status(StatusCode::FORBIDDEN);
}

// ═══════════════════════════════════════════════════════════════════════════
// WebDAV Write Operation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn webdav_put_overwrites_existing_file() {
    let env = TestEnv::new().webdav();
    env.write("file.txt", "old");
    env.method_with_body("PUT", "/file.txt", "new").await.assert_status(StatusCode::NO_CONTENT);
    assert_eq!(std::fs::read_to_string(env.root.join("file.txt")).unwrap(), "new");
}

#[tokio::test]
async fn webdav_put_with_correct_auth_succeeds() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.authed("PUT", "/newfile.txt", "data", "admin", "secret").await.assert_status(StatusCode::CREATED);
    assert!(env.root.join("newfile.txt").exists());
}

#[tokio::test]
async fn webdav_put_with_wrong_auth_fails() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.authed("PUT", "/newfile.txt", "data", "admin", "wrong").await.assert_status(StatusCode::UNAUTHORIZED);
    assert!(!env.root.join("newfile.txt").exists());
}

#[tokio::test]
async fn webdav_mkcol_creates_directory() {
    let env = TestEnv::new().webdav();
    env.method("MKCOL", "/newdir").await.assert_status(StatusCode::CREATED);
    assert!(env.root.join("newdir").is_dir());
}

#[tokio::test]
async fn webdav_mkcol_conflict_if_exists() {
    let env = TestEnv::new().webdav();
    env.mkdir("existing");
    env.method("MKCOL", "/existing").await.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn webdav_mkcol_requires_auth() {
    let env = TestEnv::new().webdav().auth("user", "pass");
    env.method("MKCOL", "/newdir").await.assert_status(StatusCode::UNAUTHORIZED);
    assert!(!env.root.join("newdir").exists());
}

#[tokio::test]
async fn webdav_delete_removes_directory() {
    let env = TestEnv::new().webdav();
    env.mkdir("mydir");
    env.write("mydir/file.txt", "data");
    env.method("DELETE", "/mydir").await.assert_status(StatusCode::NO_CONTENT);
    assert!(!env.root.join("mydir").exists());
}

#[tokio::test]
async fn webdav_delete_nonexistent_returns_404() {
    let env = TestEnv::new().webdav();
    env.method("DELETE", "/nonexistent.txt").await.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn webdav_copy_file() {
    let env = TestEnv::new().webdav();
    env.write("source.txt", "hello");
    env.authed_with_headers("COPY", "/source.txt", "", "", "", vec![("Destination", "/dest.txt")]).await.assert_status(StatusCode::CREATED);
    assert!(env.root.join("dest.txt").exists());
    assert_eq!(std::fs::read_to_string(env.root.join("dest.txt")).unwrap(), "hello");
    // Source should still exist
    assert!(env.root.join("source.txt").exists());
}

#[tokio::test]
async fn webdav_move_file() {
    let env = TestEnv::new().webdav();
    env.write("source.txt", "hello");
    env.authed_with_headers("MOVE", "/source.txt", "", "", "", vec![("Destination", "/dest.txt")]).await.assert_status(StatusCode::CREATED);
    assert!(env.root.join("dest.txt").exists());
    assert_eq!(std::fs::read_to_string(env.root.join("dest.txt")).unwrap(), "hello");
    // Source should be gone
    assert!(!env.root.join("source.txt").exists());
}

#[tokio::test]
async fn webdav_move_requires_destination_header() {
    let env = TestEnv::new().webdav();
    env.write("source.txt", "hello");
    env.method("MOVE", "/source.txt").await.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn webdav_copy_overwrite_false_conflicts() {
    let env = TestEnv::new().webdav();
    env.write("source.txt", "hello");
    env.write("dest.txt", "existing");
    env.authed_with_headers("COPY", "/source.txt", "", "", "", vec![("Destination", "/dest.txt"), ("Overwrite", "F")]).await.assert_status(StatusCode::CONFLICT);
    // dest.txt should be unchanged
    assert_eq!(std::fs::read_to_string(env.root.join("dest.txt")).unwrap(), "existing");
}

#[tokio::test]
async fn webdav_proppatch_returns_multistatus() {
    let env = TestEnv::new().webdav();
    env.write("file.txt", "data");
    let body = env.method("PROPPATCH", "/file.txt").await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains("<D:multistatus"));
    assert!(body.contains("HTTP/1.1 200 OK"));
}

#[tokio::test]
async fn webdav_put_hidden_file_denied() {
    let env = TestEnv::new().webdav();
    env.method_with_body("PUT", "/.secret", "data").await.assert_status(StatusCode::FORBIDDEN);
    assert!(!env.root.join(".secret").exists());
}

#[tokio::test]
async fn webdav_put_hidden_file_allowed_with_show_hidden() {
    let env = TestEnv::new().webdav().show_hidden();
    env.method_with_body("PUT", "/.secret", "data").await.assert_status(StatusCode::CREATED);
    assert!(env.root.join(".secret").exists());
}

#[tokio::test]
async fn webdav_copy_with_auth() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.write("source.txt", "hello");
    // Without auth → 401
    env.authed_with_headers("COPY", "/source.txt", "", "", "", vec![("Destination", "/dest.txt")]).await.assert_status(StatusCode::UNAUTHORIZED);
    // With auth → success
    env.authed_with_headers("COPY", "/source.txt", "", "admin", "secret", vec![("Destination", "/dest.txt")]).await.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn webdav_options_includes_write_methods() {
    let env = TestEnv::new().webdav();
    let resp = env.method("OPTIONS", "/").await.assert_status(StatusCode::OK);
    let allow = resp.header("Allow").unwrap();
    assert!(allow.contains("PUT"));
    assert!(allow.contains("DELETE"));
    assert!(allow.contains("MKCOL"));
    assert!(allow.contains("COPY"));
    assert!(allow.contains("MOVE"));
    assert!(allow.contains("PROPPATCH"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth protects WebDAV operations only (not browser/web page access)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn auth_does_not_block_browser_get() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.write("file.txt", "hello");
    // Browser GET should work without auth
    let body = env.get("/file.txt").await.assert_status(StatusCode::OK).text().await;
    assert_eq!(body, "hello");
}

#[tokio::test]
async fn auth_does_not_block_browser_directory_listing() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.write("file.txt", "data");
    // Browser directory listing should work without auth
    env.get("/").await.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn auth_blocks_propfind_when_configured() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.write("file.txt", "data");
    env.propfind("/", "1").await.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_allows_propfind_with_correct_credentials() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.write("file.txt", "data");
    let body = env.authed_with_headers("PROPFIND", "/", "", "admin", "secret", vec![("Depth", "1")]).await.assert_status(StatusCode::MULTI_STATUS).text().await;
    assert!(body.contains("file.txt"));
}

#[tokio::test]
async fn auth_blocks_all_webdav_methods() {
    let env = TestEnv::new().webdav().auth("admin", "secret");
    env.write("file.txt", "data");
    // All WebDAV methods should require auth
    env.propfind("/", "0").await.assert_status(StatusCode::UNAUTHORIZED);
    env.method("MKCOL", "/newdir").await.assert_status(StatusCode::UNAUTHORIZED);
    env.method_with_body("PUT", "/new.txt", "data").await.assert_status(StatusCode::UNAUTHORIZED);
    env.method("DELETE", "/file.txt").await.assert_status(StatusCode::UNAUTHORIZED);
    env.method("LOCK", "/").await.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn no_auth_allows_all_operations() {
    // Without auth configured, everything is open
    let env = TestEnv::new().webdav();
    env.write("file.txt", "hello");
    env.get("/file.txt").await.assert_status(StatusCode::OK);
    env.propfind("/", "1").await.assert_status(StatusCode::MULTI_STATUS);
    env.method_with_body("PUT", "/new.txt", "data").await.assert_status(StatusCode::CREATED);
}
