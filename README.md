# EchoFS

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

[中文文档](README_cn.md)

A lightweight, single-binary file server written in Rust. Browse directories, preview media files, and share links — all from your terminal. 

This is a native AI project, entirely written by AI.

## Features

- **Single Binary** — Compiles to one ~1.3 MB executable, no runtime dependencies
- **Directory Browsing** — Modern web UI with breadcrumb navigation and multi-level directory support
- **Media Preview** — Play video/audio and view images directly in the browser (HTML5)
- **Shareable Links** — Copy file links that work in external players like VLC and mpv
- **HTTP Range Requests** — Full support for `206 Partial Content`, enabling video seeking and resumable downloads
- **Sortable File List** — Sort by name, size, created time, or modified time (ascending/descending)
- **Dark / Light Theme** — Toggle with one click, persisted via `localStorage`
- **Responsive Design** — Card-based layout on mobile, table layout on desktop, optimized for iPad and phones
- **Frosted Glass Header** — Sticky header with `backdrop-filter` blur effect
- **LAN Access Info** — When binding to `0.0.0.0`, displays all available local network addresses
- **Security** — Path traversal protection via `canonicalize` + `starts_with` validation
- **Access Logging** — Request logs to stdout (default), a file, or disabled entirely

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.90 or later

### Build

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
| `GET` | `/` | List root directory (with `X-Requested-With: XMLHttpRequest` header) |
| `GET` | `/{path}` | List subdirectory (with `X-Requested-With: XMLHttpRequest` header) |

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
│   ├── handlers.rs      Route handlers: HTML pages, JSON API, file streaming
│   ├── logging.rs       Access log middleware (stdout, file, or off)
│   ├── range.rs         HTTP Range request parsing and 206 response builder
│   ├── directory.rs     Directory traversal, path safety checks, entry collection
│   ├── template.rs      Embedded HTML/CSS/JS single-page application
│   ├── mime_utils.rs    MIME type detection and file icon mapping
│   └── error.rs         Unified error type implementing IntoResponse
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

The project includes 83 automated tests:

- **Unit tests** (67) — embedded in each source module via `#[cfg(test)]`, covering range parsing, directory listing, MIME detection, error handling, logging, and template content
- **Integration tests** (16) — in `tests/integration_test.rs`, testing the full Axum router via `tower::ServiceExt::oneshot()`: HTML serving, JSON API responses, file streaming with Range requests, path traversal security, and MIME types

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
