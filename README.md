# EchoFS

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

[中文文档](README_cn.md)

A lightweight, single-binary file server written in Rust. Browse directories, preview media files, and share links — all from your terminal. Supports read-only WebDAV for mounting as a network drive in file managers.

This is a native AI project, entirely written by AI.

## Features

- **Single Binary** — Compiles to one ~1.4 MB executable, no runtime dependencies
- **Directory Browsing** — Modern web UI with breadcrumb navigation and multi-level directory support
- **Media Preview** — Play video/audio and view images directly in the browser (HTML5)
- **Shareable Links** — Copy file links that work in external players like VLC and mpv
- **HTTP Range Requests** — Full support for `206 Partial Content`, enabling video seeking and resumable downloads
- **Sortable File List** — Sort by name, size, created time, or modified time (ascending/descending)
- **Dark / Light Theme** — Toggle with one click, persisted via `localStorage`
- **Responsive Design** — Card-based layout on mobile, table layout on desktop, optimized for iPad and phones
- **Frosted Glass Header** — Sticky header with `backdrop-filter` blur effect
- **LAN Access Info** — When binding to `0.0.0.0`, displays all available local network addresses
- **Security** — Path traversal protection via `canonicalize` + `starts_with`; hidden files/directories (`.env`, `.git`, etc.) are blocked from both listings and direct URL access by default (use `--show-hidden` to allow)
- **Directory Depth Limiting** — Restrict how deep users can browse with `--max-depth` (e.g., `--max-depth 1` for one level of subdirectories, `0` for root only)
- **Per-Request Speed Limiting** — Throttle download speed per request with `--speed-limit` (e.g., `1m` for 1 MB/s, `500k` for 500 KB/s); uses token-bucket algorithm with async sleep, zero overhead when disabled
- **WebDAV (Read-Only)** — Enabled by default; mount the served directory as a network drive in macOS Finder, Windows Explorer, or Linux file managers via standard WebDAV protocol (PROPFIND/OPTIONS); respects all security settings (hidden files, path traversal, depth limits); disable with `--no-webdav`
- **HTML Error Pages** — Styled error pages for browser requests (404, 403, etc.), JSON errors for API clients
- **Async I/O** — All filesystem operations run in `spawn_blocking` to avoid blocking the async runtime
- **Access Logging** — Request logs to stdout (default), a file, or disabled entirely

## Quick Start

### Download Pre-built Binaries

You can download pre-built binaries directly from [GitHub Releases](https://github.com/dengsgo/echofs/releases):

| Platform | Architecture | Download |
|----------|-------------|----------|
| Linux | AMD64 (x86_64) | [echofs-linux-amd64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-linux-amd64.tar.gz) |
| Linux | ARM64 | [echofs-linux-arm64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-linux-arm64.tar.gz) |
| macOS | AMD64 (Intel) | [echofs-darwin-amd64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-darwin-amd64.tar.gz) |
| macOS | ARM64 (Apple Silicon) | [echofs-darwin-arm64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-darwin-arm64.tar.gz) |
| Windows | AMD64 (x86_64) | [echofs-windows-amd64.zip](https://github.com/dengsgo/echofs/releases/latest/download/echofs-windows-amd64.zip) |

**Quick install (Linux/macOS):**
```bash
# Download and extract (replace with your platform)
curl -LO https://github.com/dengsgo/echofs/releases/latest/download/echofs-linux-amd64.tar.gz
tar xzf echofs-linux-amd64.tar.gz
sudo mv echofs /usr/local/bin/
```

### Build from Source

If you prefer to build from source:

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.90 or later

#### Build

```bash
git clone https://github.com/dengsgo/echofs.git
cd echofs
cargo build --release
```

The binary is at `./target/release/echofs`.

### Run

```bash
# Serve current directory on port 8080
./target/release/echofs

# Serve a specific directory on a custom port
./target/release/echofs --root /path/to/files --port 9000

# Auto-open browser
./target/release/echofs --open
```

## Usage

```
Usage: echofs [OPTIONS]

Options:
  -r, --root <ROOT>  Root directory to serve [default: .]
  -p, --port <PORT>  Port to listen on [default: 8080]
  -b, --bind <BIND>  Bind address [default: 0.0.0.0]
  -o, --open         Open browser automatically
  -H, --show-hidden  Show hidden files and directories (names starting with '.')
  -d, --max-depth <MAX_DEPTH>  Maximum directory depth for browsing (-1 for unlimited) [default: -1]
  -s, --speed-limit <SPEED_LIMIT>  Speed limit per request, e.g. 500k, 1m, 10m [default: unlimited]
      --no-webdav    Disable read-only WebDAV access [default: enabled]
  -l, --log <LOG>    Access log output: "stdout", "off", or a file path [default: stdout]
  -h, --help         Print help
```

### Examples

```bash
# Share your Downloads folder on port 3000
echofs -r ~/Downloads -p 3000

# Bind to localhost only (no LAN access)
echofs -b 127.0.0.1 -p 8080

# Serve and open browser
echofs -r /media/videos --open

# Log access to a file
echofs --log /var/log/echofs.log

# Disable access logging
echofs --log off

# Show hidden files (e.g., .env, .git)
echofs --show-hidden

# Limit browsing depth to one level of subdirectories
echofs --max-depth 1

# Only allow browsing root directory
echofs -d 0

# Limit download speed to 1MB/s per request
echofs --speed-limit 1m

# Limit download speed to 500KB/s per request
echofs -s 500k

# Disable WebDAV access
echofs --no-webdav
```

#### WebDAV Access

WebDAV is enabled by default. File managers can mount the served directory as a network drive:

```bash
# macOS Finder: Go → Connect to Server (⌘K)
# Enter: http://localhost:8080

# Windows Explorer: Map Network Drive
# Enter: \\localhost@8080\

# Linux (GNOME Files / Nautilus): Connect to Server
# Enter: dav://localhost:8080

# Command line (curl)
curl -X PROPFIND http://localhost:8080/ -H "Depth: 1"
curl -X OPTIONS http://localhost:8080/
```

When binding to `0.0.0.0`, the console shows all reachable addresses:

```
EchoFS serving /home/user/files on http://0.0.0.0:8080
Available on:
  http://127.0.0.1:8080
  http://192.168.1.42:8080
Listening on 0.0.0.0:8080
```

Access log format (enabled by default):

```
[2025-01-15 10:30:00] 192.168.1.5 GET P / 200 0.8ms
[2025-01-15 10:30:00] 192.168.1.5 GET A / 200 0.5ms
[2025-01-15 10:30:01] 192.168.1.5 GET P /photos/sunset.jpg 206 1.2ms
```

## API

EchoFS provides a JSON API for directory listings. To get JSON instead of HTML for any directory path, include the `X-Requested-With: XMLHttpRequest` header in your request:

| Method | Path | Description |
|--------|------|-------------|
| `GET` `HEAD` | `/` | Root directory — HTML UI or JSON listing (with `X-Requested-With: XMLHttpRequest` header) |
| `GET` `HEAD` | `/{path}` | Subdirectory or file — directories return HTML/JSON, files are streamed; hidden paths (`.`-prefixed) return 403 unless `--show-hidden` is enabled |
| `PROPFIND` | `/` `/{path}` | WebDAV directory/file metadata — returns `207 Multi-Status` XML; supports `Depth: 0` (self) and `Depth: 1` (self + children); enabled by default, disabled with `--no-webdav` |
| `OPTIONS` | `/` `/{path}` | WebDAV capability discovery — returns `DAV: 1` header and allowed methods |

Without the header, directory paths return the HTML UI. With the header, they return JSON.

Response example:

```json
{
  "path": "/photos",
  "breadcrumbs": [
    { "name": "Home", "href": "/" },
    { "name": "photos", "href": "/photos" }
  ],
  "entries": [
    {
      "name": "sunset.jpg",
      "is_dir": false,
      "size": 2048000,
      "size_display": "2.0 MB",
      "created": "2025-01-15 10:30:00",
      "modified": "2025-01-15 10:30:00",
      "created_ts": 1736934600,
      "modified_ts": 1736934600,
      "icon": "image",
      "href": "/photos/sunset.jpg",
      "media_type": "image"
    }
  ]
}
```

## Project Structure

```
echofs/
├── Cargo.toml
├── src/
│   ├── lib.rs           Library crate root: re-exports all modules
│   ├── main.rs          Entry point: CLI parsing, LAN IP detection, server startup
│   ├── cli.rs           Command-line argument definitions (clap derive)
│   ├── server.rs        Axum router, CORS middleware, TCP listener
│   ├── handlers.rs      Route handlers: HTML pages, JSON API, file streaming, error dispatch
│   ├── logging.rs       Access log middleware (stdout, file, or off)
│   ├── range.rs         HTTP Range request parsing and 206 response builder
│   ├── directory.rs     Async directory traversal, path safety, hidden file blocking
│   ├── template.rs      Embedded HTML/CSS/JS single-page application and error pages
│   ├── mime_utils.rs    MIME type detection and file icon mapping
│   ├── error.rs         Unified error type with dual-mode responses (HTML/JSON)
│   ├── throttle.rs      Per-request speed limiting (token-bucket ThrottledRead wrapper)
│   └── webdav.rs        Read-only WebDAV: PROPFIND/OPTIONS handlers, XML response generation
└── tests/
    └── integration_test.rs   Integration tests (router, API, file serving, security)
```

## Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test --verbose
```

The project includes over 150 automated tests:

- **Unit tests** — embedded in each source module via `#[cfg(test)]`, covering range parsing, directory listing, MIME detection, error handling, logging, template content, hidden file blocking, directory depth limiting, speed limit parsing, throttled read throughput, HTML/JSON error dispatch, WebDAV XML generation, Depth header parsing, and XML escaping
- **Integration tests** — in `tests/integration_test.rs`, testing the full Axum router via `tower::ServiceExt::oneshot()`: HTML serving, JSON API responses, file streaming with Range requests, path traversal security, hidden file access denial, directory depth enforcement, HEAD method support, error page format dispatch, MIME types, and WebDAV (PROPFIND/OPTIONS responses, Depth 0/1 behavior, hidden file blocking, max-depth enforcement, disabled-flag behavior)

CI runs `cargo test` on Linux, macOS, and Windows before building release artifacts.

## Dependencies

| Crate | Purpose |
|-------|---------|
| [axum](https://crates.io/crates/axum) | HTTP framework |
| [tokio](https://crates.io/crates/tokio) | Async runtime |
| [tower-http](https://crates.io/crates/tower-http) | CORS middleware |
| [tokio-util](https://crates.io/crates/tokio-util) | Streaming file I/O |
| [clap](https://crates.io/crates/clap) | CLI argument parsing |
| [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) | JSON serialization |
| [mime_guess](https://crates.io/crates/mime_guess) / [mime](https://crates.io/crates/mime) | MIME type detection |
| [chrono](https://crates.io/crates/chrono) | Date/time formatting |
| [percent-encoding](https://crates.io/crates/percent-encoding) | URL encoding |
| [open](https://crates.io/crates/open) | Open browser |
| [libc](https://crates.io/crates/libc) | Network interface enumeration |

## Disclaimer

**This is a 100% AI-written project.** Every line of code, documentation, and configuration in this repository was generated by AI.

The goal of this project is twofold: to build a tool that fulfills a personal need, and to explore the capabilities of AI-driven software development. This is also why the project is written in Rust — a language the author is not proficient in — to put AI coding ability to a real test.

## Contributing

Contributions are welcome, with one rule: **all submitted code must be AI-generated.**

This project is an experiment in AI-first development. Pull requests written by hand will not be accepted. We believe that in the era of AI, manual coding is inefficient and outdated, which goes against the purpose of this project.

When submitting a PR, please indicate the **AI tool and the specific model** used. For example:

- `AI Tool: Claude Code / claude-opus-4.6`
- `AI Tool: Cursor / gpt-4o`
- `AI Tool: GitHub Copilot / gemini-2.5-pro`

## License

This project is licensed under the [Apache License 2.0](LICENSE).
