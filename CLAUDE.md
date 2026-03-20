# CLAUDE.md

This file provides guidance to Claude Code when working on this repository.

## Project Overview

EchoFS is a single-binary file server written in Rust. It serves a local directory over HTTP with a browser-based UI for directory browsing, media preview, and file sharing.

## Build & Run

```bash
# Check compilation
cargo check

# Build release binary (~1.3 MB)
cargo build --release

# Run (serves current directory on port 8080)
./target/release/echofs

# Run with options
./target/release/echofs --root /path/to/dir --port 9000 --open

# Log to file instead of stdout
./target/release/echofs --log /var/log/echofs.log

# Disable access logging
./target/release/echofs --log off

# Show hidden files and directories
./target/release/echofs --show-hidden

# Limit directory browsing depth
./target/release/echofs --max-depth 1

# Only allow browsing root directory (no subdirectory access)
./target/release/echofs -d 0
```

## Architecture

Single-binary SPA architecture: the HTML/CSS/JS is embedded in `template.rs` and served inline. The frontend fetches directory data from JSON API endpoints and renders client-side.

### Source Files

- `lib.rs` — Library crate root: re-exports all modules as `pub mod` for use by `main.rs` and integration tests
- `main.rs` — Entry point: CLI parsing, LAN IP detection, server startup; imports modules from the `echofs` library crate
- `cli.rs` — clap derive CLI arguments (root, port, bind, open, log, show-hidden, max-depth)
- `server.rs` — Axum router setup, CORS layer, access log middleware, TCP listener
- `handlers.rs` — Route handlers: serves HTML for directories, streams files with Range support, JSON API; errors are dispatched via `AppError::into_response_for(&headers)` to return HTML for browsers or JSON for AJAX
- `range.rs` — HTTP Range header parsing, builds 200/206/416 responses with streaming body
- `directory.rs` — Async directory listing with path traversal protection (`canonicalize` + `starts_with`), conditional hidden file access blocking (controlled by `--show-hidden` flag), and directory depth limiting (controlled by `--max-depth` flag); all filesystem I/O runs in `tokio::task::spawn_blocking` to avoid blocking the async runtime
- `template.rs` — Embedded SPA (HTML/CSS/JS) with dark/light theme, responsive layout, media preview modal, dynamic page title; also provides `error_html()` for styled error pages
- `mime_utils.rs` — MIME detection via `mime_guess`, file type icon mapping
- `error.rs` — `AppError` enum with dual-mode responses: `into_response_for(headers)` returns HTML error pages for browser requests and JSON for AJAX requests; also implements `IntoResponse` (JSON-only) as fallback
- `logging.rs` — Access log axum middleware; `LogTarget` enum (Stdout/Off/File) drives output; uses `ConnectInfo<SocketAddr>` for client IP and `tokio::sync::Mutex` for file writes

### Tests

- `src/*.rs` — Each source module contains `#[cfg(test)] mod tests` with unit tests (84 total)
- `tests/integration_test.rs` — Integration tests (38 total) that build the Axum router directly via `tower::ServiceExt::oneshot()`, covering HTML serving, JSON API, file streaming, Range requests, path traversal security, hidden file blocking, `--show-hidden` flag behavior, `--max-depth` directory depth limiting, HEAD method support, HTML/JSON error page dispatch, and MIME types

### Routes

| Method | Path | Handler |
|--------|------|---------|
| GET, HEAD | `/` | `serve_index` — returns HTML page, or JSON listing if `X-Requested-With: XMLHttpRequest` header is present |
| GET, HEAD | `/{*path}` | `serve_path` — directory → HTML (or JSON with XHR header), file → streamed content; hidden paths (any component starting with `.`) are rejected with 403 unless `--show-hidden` is enabled |

## Key Patterns

- **Path safety**: All user-supplied paths go through `directory::safe_resolve()` which blocks hidden file access (any path component starting with `.`) unless `--show-hidden` is enabled, enforces directory depth limits via `--max-depth`, then canonicalizes and validates paths stay within the root. All filesystem I/O (`canonicalize`, `read_dir`, `metadata`) runs inside `tokio::task::spawn_blocking` to avoid blocking the async runtime.
- **Hidden file protection**: Hidden files/directories (names starting with `.`) are both excluded from directory listings and blocked from direct URL access (e.g., `/.env`, `/.git/config`), including percent-encoded variants. This behavior can be disabled with the `--show-hidden` (`-H`) CLI flag, while path traversal protection remains enforced regardless.
- **Directory depth limiting**: The `--max-depth` (`-d`) flag controls how deep users can browse into the directory tree. Depth 0 = root only, depth 1 = one level of subdirectories, -1 = unlimited (default). When at maximum depth, subdirectories are hidden from listings, and paths exceeding the depth are rejected with 403. Depth checks run in `safe_resolve()` before any filesystem I/O.
- **Streaming**: Files are served via `tokio_util::io::ReaderStream`, never loaded fully into memory. Range requests use `AsyncSeekExt` + `AsyncReadExt::take()`.
- **Error responses**: `AppError::into_response_for(&headers)` provides dual-mode error handling — styled HTML error pages for browser requests, JSON `{"error": "..."}` for AJAX requests.
- **Frontend navigation**: The SPA uses `history.pushState` for client-side routing. All `<a data-nav>` clicks are intercepted and handled via `fetch` with an `X-Requested-With: XMLHttpRequest` header, which makes the server return JSON instead of HTML for the same path. The page `<title>` updates dynamically to reflect the current directory name.
- **Platform-specific code**: `main.rs` uses `#[cfg(unix)]` with `libc::getifaddrs` for network interface enumeration.
- **Access logging**: Implemented as an axum `from_fn_with_state` middleware layer. `LogTarget` is passed as middleware state, separate from the app `AppState`. Log format: `[timestamp] ip method kind uri status elapsed_ms`, where `kind` is `A` (API/AJAX) or `P` (page).

## Code Style

- Rust 2024 edition
- No `unwrap()` in library code; `expect()` is used only for infallible builder patterns with clear messages; errors flow through `AppError` which converts to proper HTTP status codes
- Dependencies are kept minimal; no template engine — HTML is a raw string in `template.rs`
