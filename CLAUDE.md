# CLAUDE.md

## Overview

EchoFS: single-binary HTTP file server (Rust/Axum). Serves a local directory with browser UI + read-write WebDAV. HTML/CSS/JS embedded in binary (no template engine).

## Commands

```bash
cargo check          # Fast compilation check
cargo test           # Run all tests (unit + integration)
cargo build --release  # Build release binary
```

## Architecture

```
src/
  main.rs       — CLI parsing (clap), LAN IP detection, server startup
  lib.rs        — Library crate root, re-exports all modules
  cli.rs        — CLI argument definitions
  server.rs     — Axum router, middleware layers, TCP listener
  handlers.rs   — GET/HEAD route handlers, AppState struct
  directory.rs  — Directory listing, path safety (safe_resolve)
  range.rs      — HTTP Range responses, streaming
  throttle.rs   — Token-bucket speed limiter (ThrottledRead)
  template.rs   — Embedded SPA HTML + error_html()
  mime_utils.rs — MIME detection, file type icons
  error.rs      — AppError enum, dual-mode responses
  logging.rs    — Access log middleware
  webdav.rs     — WebDAV (PROPFIND/PUT/DELETE/MKCOL/COPY/MOVE/PROPPATCH/LOCK/UNLOCK)
tests/
  integration_test.rs — Integration tests via tower::ServiceExt::oneshot()
```

## Key Conventions

- **Rust 2024 edition**
- **No `unwrap()` in library code** — use `AppError` for error propagation; `expect()` only for infallible builder patterns
- **Minimal dependencies** — no XML library (hand-rolled `XmlWriter`), no template engine
- **All filesystem I/O in `spawn_blocking`** — never block the async runtime
- **Path safety via `safe_resolve()`** — canonicalize + starts_with root check; blocks hidden files (`.`-prefixed) unless `--show-hidden`; enforces `--max-depth`
- **Dual-mode error responses** — `AppError::into_response_for(&headers)` returns HTML for browsers, JSON for AJAX (`X-Requested-With: XMLHttpRequest`)
- **Streaming, never buffered** — files served via `ReaderStream`, optionally wrapped in `ThrottledRead`
- **WebDAV auth is independent** — `--webdav-user`/`--webdav-pass` only protects WebDAV operations, never browser GET/HEAD
- **SPA client routing** — `history.pushState` + XHR header triggers JSON response from same endpoint
