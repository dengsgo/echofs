# CLAUDE.md

## Overview

EchoFS: single-binary HTTP file server (Rust/Axum). Serves a local directory with browser UI + read-write WebDAV. HTML/CSS/JS embedded in binary (no template engine). Multi-theme support (Classic, Liquid Glass, Cartoon). Web UI file management (upload/rename/delete) powered by WebDAV. Optional native desktop GUI control panel (egui) behind the `gui` feature.

## Commands

```bash
cargo check          # Fast compilation check
cargo test           # Run all tests (unit + integration)
cargo build --release  # Build release binary
cargo check --features gui    # Compile-check with the desktop GUI
cargo build --release --features gui  # Build with the desktop GUI
./scripts/make-macos-app.sh --dmg     # Build macOS EchoFS.app + .dmg (universal, ad-hoc signed)
./scripts/make-icon.sh                # Regenerate assets/echofs.icns from assets/icon.svg
```

## Architecture

```
src/
  main.rs       — CLI parsing (clap), GUI/headless dispatch, manual Tokio runtime, server startup
  lib.rs        — Library crate root, re-exports all modules (gui module is cfg-gated)
  cli.rs        — CLI argument definitions (--gui flag is cfg(feature="gui"))
  config.rs     — ServerConfig: resolved startup config shared by CLI and GUI; From<&Args>
  server.rs     — Axum router (build_router), middleware layers, TCP listener; run() returns ServerHandle (graceful shutdown via oneshot)
  handlers.rs   — GET/HEAD route handlers, AppState struct, DirResponse (JSON with webdav capabilities)
  directory.rs  — Directory listing, path safety (safe_resolve)
  range.rs      — HTTP Range responses, streaming
  throttle.rs   — Token-bucket speed limiter (ThrottledRead)
  template.rs   — Embedded SPA HTML/CSS/JS: themes, file management UI, modals, toast notifications
  netinfo.rs    — LAN IP enumeration (local_ips): UDP probe + getifaddrs
  mime_utils.rs — MIME detection, file type icons
  error.rs      — AppError enum, dual-mode responses
  logging.rs    — Access log middleware (Stdout/Off/File/Channel; Channel feeds the GUI log panel)
  webdav.rs     — WebDAV (PROPFIND/PUT/DELETE/MKCOL/COPY/MOVE/PROPPATCH/LOCK/UNLOCK)
  gui.rs        — Optional egui desktop control panel; entire module is #[cfg(feature="gui")]
tests/
  integration_test.rs — Integration tests via tower::ServiceExt::oneshot(); plus lifecycle mod (real listener + graceful shutdown)
assets/
  icon.svg      — App icon source (Big Sur squircle + folder/echo-waves glyph); echofs.icns generated from it (committed)
scripts/
  make-icon.sh       — Rasterize icon.svg → echofs.icns (needs rsvg-convert + iconutil)
  make-macos-app.sh  — Build EchoFS.app (universal via lipo) + optional .dmg; ad-hoc codesigns
```

## Key Conventions

- **Rust 2024 edition**
- **No `unwrap()` in library code** — use `AppError` for error propagation; `expect()` only for infallible builder patterns
- **Minimal dependencies** — no XML library (hand-rolled `XmlWriter`), no template engine
- **GUI is an opt-in feature** — `gui.rs` and the `eframe`/`rfd`/`qrcode` deps compile ONLY with `--features gui`; the default build stays a lean, zero-runtime-dependency CLI. Keep GUI code and deps off the default path (cfg-gate new GUI surface).
- **GUI entry dispatch** — `main.rs` runs the GUI when a gui-enabled build is started with `--gui` OR with zero CLI args; any argument runs headless. The default (no-gui) build always runs headless.
- **Manual Tokio runtime** — `main.rs` builds the runtime explicitly (not `#[tokio::main]`) so the GUI can own the main thread (required by native windowing on macOS).
- **macOS .app bundle** — `scripts/make-macos-app.sh` builds `EchoFS.app` + `.dmg` + a standalone `echofs-<label>-gui.tar.gz`. With `--target <triple>` it builds one arch (label `darwin-amd64`/`darwin-arm64`); without, a universal (`lipo` x86_64+arm64, label `universal`). Ad-hoc signed (`codesign -s -`), NOT notarized — Gatekeeper warns on first launch (README documents the right-click-Open / `xattr` workaround). `echofs.icns` is committed so CI needs no SVG rasterizer. The `macos-app` CI job is a 2-arch matrix producing per-chip `.dmg` + GUI-binary release artifacts (Intel and Apple Silicon shipped separately, not universal).
- **Server lifecycle** — `server::run(ServerConfig, LogTarget)` binds the listener and returns `Result<ServerHandle, StartError>` (no `process::exit` inside); supports port 0 (OS-assigned, read back via `local_addr`) and graceful shutdown via `ServerHandle::stop()`. CLI/GUI build `ServerConfig`; handlers still use `AppState` (built inside `run`).
- **egui 0.35 note** — `App::ui(&mut self, &mut Ui, ..)` (not `update`); wrap the body in `CentralPanel` or empty regions show the near-black window clear-color. Panels (`Panel::top`/`CentralPanel`) take `&mut Ui`. Verify egui APIs against the installed crate source — the version here postdates older API memory.
- **GUI i18n** — the desktop GUI is bilingual (English / 简体中文), hand-rolled in `gui.rs` (no i18n crate): a `Lang` enum + `tr(lang, en, zh)` helper with translations inline at each call site. `Lang::detect()` picks the initial language from the OS locale via `system_locale()` (checks `LC_*`/`LANG` env vars, then a per-OS fallback for GUI launches that inherit none — `defaults read -g AppleLocale` on macOS, `GetUserDefaultLocaleName` FFI on Windows; Linux desktop sessions propagate `LANG`). `from_locale()` is split out and unit-tested. A header combo box switches language at runtime (even while running). The choice is not persisted — re-detected each launch.
- **GUI CJK fonts** — egui bundles only Latin/Cyrillic faces, so CJK text renders as tofu boxes without a fallback. `install_cjk_font()` (called in the `eframe::run_native` creation callback) loads the first available *system* CJK font from a per-OS `CJK_FONT_CANDIDATES` list (macOS/Windows/Linux) and registers it as a `FontPriority::Lowest` fallback for both families — Latin defaults stay primary. No font is embedded (they are 15–50 MB; keeps the binary lean); a no-op with a stderr warning if none is found. All of this is inside the `cfg(feature="gui")` module.
- **egui uses skrifa** — text shaping/rasterization is via `skrifa` (not `ab_glyph`); `FontData.index` selects a face in a `.ttc`/`.otf` collection, so system TrueType Collections load directly.
- **All filesystem I/O in `spawn_blocking`** — never block the async runtime
- **Path safety via `safe_resolve()`** — canonicalize + starts_with root check; blocks hidden files (`.`-prefixed) unless `--show-hidden`; enforces `--max-depth`
- **Dual-mode error responses** — `AppError::into_response_for(&headers)` returns HTML for browsers, JSON for AJAX (`X-Requested-With: XMLHttpRequest`)
- **Streaming, never buffered** — files served via `ReaderStream`, optionally wrapped in `ThrottledRead`
- **WebDAV auth is shared** — `--webdav-user`/`--webdav-pass` protects both WebDAV client operations and web UI file management (upload/rename/delete); browser GET/HEAD remains open
- **SPA client routing** — `history.pushState` + XHR header triggers JSON response from same endpoint
- **Theme system** — CSS variables + `data-style` (classic/glass/cartoon) + `data-mode` (light/dark) attributes on `<html>`; persisted in localStorage
- **Web file ops reuse WebDAV** — frontend JS calls MOVE/DELETE/PUT directly to existing WebDAV handlers; no separate backend endpoints
