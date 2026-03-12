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

- `main.rs` — Entry point: CLI parsing, LAN IP detection, server startup
- `cli.rs` — clap derive CLI arguments (root, port, bind, open, log)
- `server.rs` — Axum router setup, CORS layer, access log middleware, TCP listener
- `handlers.rs` — Route handlers: serves HTML for directories, streams files with Range support, JSON API
- `range.rs` — HTTP Range header parsing, builds 200/206/416 responses with streaming body
- `directory.rs` — Directory listing with path traversal protection (`canonicalize` + `starts_with`)
- `template.rs` — Embedded SPA (HTML/CSS/JS) with dark/light theme, responsive layout, media preview modal
- `mime_utils.rs` — MIME detection via `mime_guess`, file type icon mapping
- `error.rs` — `AppError` enum implementing `IntoResponse`
- `logging.rs` — Access log axum middleware; `LogTarget` enum (Stdout/Off/File) drives output; uses `ConnectInfo<SocketAddr>` for client IP and `tokio::sync::Mutex` for file writes

### Routes

| Method | Path | Handler |
|--------|------|---------|
| GET | `/` | `serve_index` — returns embedded HTML page |
| GET | `/{*path}` | `serve_path` — directory → HTML, file → streamed content |
| GET | `/api/ls` | `api_ls_root` — root directory JSON listing |
| GET | `/api/ls/{*path}` | `api_ls_path` — subdirectory JSON listing |

## Key Patterns

- **Path safety**: All user-supplied paths go through `directory::safe_resolve()` which canonicalizes and validates they stay within the root.
- **Streaming**: Files are served via `tokio_util::io::ReaderStream`, never loaded fully into memory. Range requests use `AsyncSeekExt` + `AsyncReadExt::take()`.
- **Frontend navigation**: The SPA uses `history.pushState` for client-side routing. All `<a data-nav>` clicks are intercepted and handled via `fetch` to the `/api/ls` endpoint.
- **Platform-specific code**: `main.rs` uses `#[cfg(unix)]` with `libc::getifaddrs` for network interface enumeration.
- **Access logging**: Implemented as an axum `from_fn_with_state` middleware layer. `LogTarget` is passed as middleware state, separate from the app `AppState`. Log format: `[timestamp] ip method uri status elapsed_ms`.

## Code Style

- Rust 2024 edition
- No `unwrap()` in library code; errors flow through `AppError` which converts to proper HTTP status codes
- Dependencies are kept minimal; no template engine — HTML is a raw string in `template.rs`
