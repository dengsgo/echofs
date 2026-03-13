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
```

## Architecture

Single-binary SPA architecture: the HTML/CSS/JS is embedded in `template.rs` and served inline. The frontend fetches directory data from JSON API endpoints and renders client-side.

### Source Files

- `lib.rs` — Library crate root: re-exports all modules as `pub mod` for use by `main.rs` and integration tests
- `main.rs` — Entry point: CLI parsing, LAN IP detection, server startup; imports modules from the `echofs` library crate
- `cli.rs` — clap derive CLI arguments (root, port, bind, open, log)
- `server.rs` — Axum router setup, CORS layer, access log middleware, TCP listener
- `handlers.rs` — Route handlers: serves HTML for directories, streams files with Range support, JSON API
- `range.rs` — HTTP Range header parsing, builds 200/206/416 responses with streaming body
- `directory.rs` — Directory listing with path traversal protection (`canonicalize` + `starts_with`)
- `template.rs` — Embedded SPA (HTML/CSS/JS) with dark/light theme, responsive layout, media preview modal
- `mime_utils.rs` — MIME detection via `mime_guess`, file type icon mapping
- `error.rs` — `AppError` enum implementing `IntoResponse`
- `logging.rs` — Access log axum middleware; `LogTarget` enum (Stdout/Off/File) drives output; uses `ConnectInfo<SocketAddr>` for client IP and `tokio::sync::Mutex` for file writes

### Tests

- `src/*.rs` — Each source module contains `#[cfg(test)] mod tests` with unit tests (67 total)
- `tests/integration_test.rs` — Integration tests (16 total) that build the Axum router directly via `tower::ServiceExt::oneshot()`, covering HTML serving, JSON API, file streaming, Range requests, path traversal security, and MIME types

### Routes

| Method | Path | Handler |
|--------|------|---------|
| GET | `/` | `serve_index` — returns HTML page, or JSON listing if `X-Requested-With: XMLHttpRequest` header is present |
| GET | `/{*path}` | `serve_path` — directory → HTML (or JSON with XHR header), file → streamed content |

## Key Patterns

- **Path safety**: All user-supplied paths go through `directory::safe_resolve()` which canonicalizes and validates they stay within the root.
- **Streaming**: Files are served via `tokio_util::io::ReaderStream`, never loaded fully into memory. Range requests use `AsyncSeekExt` + `AsyncReadExt::take()`.
- **Frontend navigation**: The SPA uses `history.pushState` for client-side routing. All `<a data-nav>` clicks are intercepted and handled via `fetch` with an `X-Requested-With: XMLHttpRequest` header, which makes the server return JSON instead of HTML for the same path.
- **Platform-specific code**: `main.rs` uses `#[cfg(unix)]` with `libc::getifaddrs` for network interface enumeration.
- **Access logging**: Implemented as an axum `from_fn_with_state` middleware layer. `LogTarget` is passed as middleware state, separate from the app `AppState`. Log format: `[timestamp] ip method kind uri status elapsed_ms`, where `kind` is `A` (API/AJAX) or `P` (page).

## Code Style

- Rust 2024 edition
- No `unwrap()` in library code; errors flow through `AppError` which converts to proper HTTP status codes
- Dependencies are kept minimal; no template engine — HTML is a raw string in `template.rs`
