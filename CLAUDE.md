# CLAUDE.md

This file provides guidance to Claude Code when working on this repository.

## Project Overview

EchoFS is a single-binary file server written in Rust. It serves a local directory over HTTP with a browser-based UI for directory browsing, media preview, and file sharing. It also provides read-only WebDAV access, allowing file managers (macOS Finder, Windows Explorer, Linux Nautilus) to mount and browse the served directory as a network drive.

## Build & Run

```bash
# Check compilation
cargo check

# Build release binary (~1.4 MB)
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

# Limit download speed per request to 1MB/s
./target/release/echofs --speed-limit 1m

# Speed limit with suffix: 500k (KB/s), 2m (MB/s), 1g (GB/s)
./target/release/echofs -s 500k

# WebDAV is enabled by default (read-write); disable it with:
./target/release/echofs --no-webdav

# Require authentication for WebDAV access (does not affect browser/web UI)
./target/release/echofs --webdav-user admin --webdav-pass secret

# Mount via macOS Finder: Go → Connect to Server → http://localhost:8080
# Mount via Windows Explorer: Map Network Drive → \\localhost@8080\
```

## Architecture

Single-binary SPA architecture: the HTML/CSS/JS is embedded in `template.rs` and served inline. The frontend fetches directory data from JSON API endpoints and renders client-side.

### Source Files

- `lib.rs` — Library crate root: re-exports all modules as `pub mod` for use by `main.rs` and integration tests
- `main.rs` — Entry point: CLI parsing, LAN IP detection, server startup; imports modules from the `echofs` library crate
- `cli.rs` — clap derive CLI arguments (root, port, bind, open, log, show-hidden, max-depth, speed-limit, no-webdav, webdav-user, webdav-pass)
- `server.rs` — Axum router setup, CORS layer, access log middleware, TCP listener; conditionally registers WebDAV routes (PROPFIND, OPTIONS) via `any()` handlers
- `handlers.rs` — Route handlers: serves HTML for directories, streams files with Range support, JSON API; errors are dispatched via `AppError::into_response_for(&headers)` to return HTML for browsers or JSON for AJAX
- `range.rs` — HTTP Range header parsing, builds 200/206/416 responses with streaming body; supports optional per-request speed limiting via `ThrottledRead` wrapper
- `directory.rs` — Async directory listing with path traversal protection (`canonicalize` + `starts_with`), conditional hidden file access blocking (controlled by `--show-hidden` flag), and directory depth limiting (controlled by `--max-depth` flag); all filesystem I/O runs in `tokio::task::spawn_blocking` to avoid blocking the async runtime
- `template.rs` — Embedded SPA (HTML/CSS/JS) with dark/light theme, responsive layout, media preview modal, dynamic page title; also provides `error_html()` for styled error pages
- `mime_utils.rs` — MIME detection via `mime_guess`, file type icon mapping
- `error.rs` — `AppError` enum with dual-mode responses: `into_response_for(headers)` returns HTML error pages for browser requests and JSON for AJAX requests; also implements `IntoResponse` (JSON-only) as fallback
- `logging.rs` — Access log axum middleware; `LogTarget` enum (Stdout/Off/File) drives output; uses `ConnectInfo<SocketAddr>` for client IP and `tokio::sync::Mutex` for file writes
- `throttle.rs` — `ThrottledRead<R: AsyncRead>` wrapper that limits read throughput using a token-bucket algorithm; also provides `parse_speed()` for human-readable rate strings (e.g. `500k`, `1m`)
- `webdav.rs` — Full read-write WebDAV support: handles PROPFIND (Depth 0/1), OPTIONS, LOCK, UNLOCK (read methods) and PUT, DELETE, MKCOL, COPY, MOVE, PROPPATCH (write methods); generates `207 Multi-Status` XML responses (`DAV:multistatus`) with resource properties (`displayname`, `getcontentlength`, `getlastmodified`, `getcontenttype`, `resourcetype`); provides `check_auth()` for Basic Auth enforcement via `--webdav-user`/`--webdav-pass` CLI flags (protects all operations when configured); reuses `directory::safe_resolve()` and `directory::safe_resolve_parent()` for path safety; XML is built via `XmlWriter` helper with no external XML library; enabled by default, disabled with `--no-webdav`

### Tests

- `src/*.rs` — Each source module contains `#[cfg(test)] mod tests` with unit tests (48 total)
- `tests/integration_test.rs` — Integration tests (72 total) that build the Axum router directly via `tower::ServiceExt::oneshot()`, covering HTML serving, JSON API, file streaming, Range requests, path traversal security, hidden file blocking, `--show-hidden` flag behavior, `--max-depth` directory depth limiting, HEAD method support, HTML/JSON error page dispatch, MIME types, and WebDAV (PROPFIND/OPTIONS responses, Depth 0/1, hidden file blocking, max-depth enforcement, disabled-flag behavior, PUT/DELETE/MKCOL/COPY/MOVE/PROPPATCH write operations, Basic Auth enforcement)

### Routes

| Method | Path | Handler |
|--------|------|---------|
| GET, HEAD | `/` | `serve_index` — returns HTML page, or JSON listing if `X-Requested-With: XMLHttpRequest` header is present |
| GET, HEAD | `/{*path}` | `serve_path` — directory → HTML (or JSON with XHR header), file → streamed content; hidden paths (any component starting with `.`) are rejected with 403 unless `--show-hidden` is enabled |
| PROPFIND | `/` | `webdav::handle_webdav_root` — returns `207 Multi-Status` XML with directory/file properties; supports `Depth: 0` (self only) and `Depth: 1` (self + children); enabled by default, disabled with `--no-webdav` |
| PROPFIND | `/{*path}` | `webdav::handle_webdav_path` — same as above for subpaths; reuses `safe_resolve()` for path safety |
| PUT | `/`, `/{*path}` | `webdav::handle_webdav_root/path` — upload/overwrite files; returns 201 Created or 204 No Content; requires auth when configured |
| DELETE | `/`, `/{*path}` | `webdav::handle_webdav_root/path` — remove files or directories (recursive); returns 204 No Content; requires auth when configured |
| MKCOL | `/`, `/{*path}` | `webdav::handle_webdav_root/path` — create directory; returns 201 Created; 409 Conflict if already exists; requires auth when configured |
| COPY | `/`, `/{*path}` | `webdav::handle_webdav_root/path` — copy file/directory to `Destination` header path; supports `Overwrite: T/F`; requires auth when configured |
| MOVE | `/`, `/{*path}` | `webdav::handle_webdav_root/path` — move/rename file/directory to `Destination` header path; supports `Overwrite: T/F`; requires auth when configured |
| PROPPATCH | `/`, `/{*path}` | `webdav::handle_webdav_root/path` — stub returning 207 success (macOS Finder compatibility) |
| OPTIONS | `/`, `/{*path}` | `webdav::handle_webdav_root/path` — returns `DAV: 1, 2` header and `Allow: OPTIONS, GET, HEAD, PUT, DELETE, MKCOL, COPY, MOVE, PROPFIND, PROPPATCH, LOCK, UNLOCK` |

## Key Patterns

- **Path safety**: All user-supplied paths go through `directory::safe_resolve()` which blocks hidden file access (any path component starting with `.`) unless `--show-hidden` is enabled, enforces directory depth limits via `--max-depth`, then canonicalizes and validates paths stay within the root. All filesystem I/O (`canonicalize`, `read_dir`, `metadata`) runs inside `tokio::task::spawn_blocking` to avoid blocking the async runtime.
- **Hidden file protection**: Hidden files/directories (names starting with `.`) are both excluded from directory listings and blocked from direct URL access (e.g., `/.env`, `/.git/config`), including percent-encoded variants. This behavior can be disabled with the `--show-hidden` (`-H`) CLI flag, while path traversal protection remains enforced regardless.
- **Directory depth limiting**: The `--max-depth` (`-d`) flag controls how deep users can browse into the directory tree. Depth 0 = root only, depth 1 = one level of subdirectories, -1 = unlimited (default). When at maximum depth, subdirectories are hidden from listings, and paths exceeding the depth are rejected with 403. Depth checks run in `safe_resolve()` before any filesystem I/O.
- **Per-request speed limiting**: The `--speed-limit` (`-s`) flag caps download throughput for each request independently. `ThrottledRead` wraps the file's `AsyncRead` with a token-bucket that refills at the configured rate (tokens capped at one second's worth to prevent bursts). Waiting for tokens uses `tokio::time::Sleep` so the async runtime is never blocked. Supports human-readable rates: `500k`, `1m`, `1.5g`. When unset, no wrapper is applied and the stream path has zero overhead.
- **Streaming**: Files are served via `tokio_util::io::ReaderStream`, never loaded fully into memory. Range requests use `AsyncSeekExt` + `AsyncReadExt::take()`. When `--speed-limit` is set, the reader is wrapped in `ThrottledRead` which uses a token-bucket algorithm to cap throughput per request; without the flag the stream path is unchanged (zero overhead).
- **Error responses**: `AppError::into_response_for(&headers)` provides dual-mode error handling — styled HTML error pages for browser requests, JSON `{"error": "..."}` for AJAX requests.
- **Frontend navigation**: The SPA uses `history.pushState` for client-side routing. All `<a data-nav>` clicks are intercepted and handled via `fetch` with an `X-Requested-With: XMLHttpRequest` header, which makes the server return JSON instead of HTML for the same path. The page `<title>` updates dynamically to reflect the current directory name.
- **Platform-specific code**: `main.rs` uses `#[cfg(unix)]` with `libc::getifaddrs` for network interface enumeration.
- **Access logging**: Implemented as an axum `from_fn_with_state` middleware layer. `LogTarget` is passed as middleware state, separate from the app `AppState`. Log format: `[timestamp] ip method kind uri status elapsed_ms`, where `kind` is `A` (API/AJAX) or `P` (page).
- **WebDAV (read-write)**: Enabled by default (disable with `--no-webdav`). Implements RFC 4918 compliance level 1 and 2: `OPTIONS` advertises `DAV: 1, 2`, `PROPFIND` returns `207 Multi-Status` XML with resource metadata. Write operations: `PUT` (upload/overwrite), `DELETE` (remove file/directory), `MKCOL` (create directory), `COPY` (copy with `Destination` header), `MOVE` (rename/move with `Destination` header), `PROPPATCH` (stub returning success for Finder compatibility). `LOCK`/`UNLOCK` return valid responses for client compatibility. Optional Basic Auth via `--webdav-user`/`--webdav-pass` protects **all WebDAV operations** (both read and write) when configured; browser/web UI access (GET/HEAD via HTML pages) is never affected by auth. Auth is enforced at the top of the WebDAV dispatch handlers (`handle_webdav_root`/`handle_webdav_path`). Depth header is parsed (`0` = self, `1` = self + children, `infinity` → capped to `1`). All path safety mechanisms (`safe_resolve`, `safe_resolve_parent`, hidden file blocking, `max-depth`) are fully enforced. XML is generated via `XmlWriter` helper with proper XML escaping — no external XML library. File downloads go through the existing GET/Range handler path, so speed limiting and streaming work transparently for WebDAV clients.

## Code Style

- Rust 2024 edition
- No `unwrap()` in library code; `expect()` is used only for infallible builder patterns with clear messages; errors flow through `AppError` which converts to proper HTTP status codes
- Dependencies are kept minimal; no template engine — HTML is a raw string in `template.rs`
