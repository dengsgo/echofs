# EchoFS

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

[中文文档](README_cn.md)

A lightweight, single-binary file server written in Rust.

Browse directories, preview media files, and share links — all from your terminal. With full read-write WebDAV support, mount your shared directory as a network drive on desktop, phone, or tablet — access and manage your files seamlessly from any device.

## Features

- **Single Binary** — ~1.4 MB executable, no runtime dependencies
- **Directory Browsing** — Modern web UI with breadcrumb navigation
- **Media Preview** — Play video/audio and view images directly in the browser
- **Shareable Links** — Copy file links for external players (VLC, mpv, etc.)
- **Range Requests** — Video seeking and resumable downloads via HTTP 206
- **Sortable File List** — Sort by name, size, created/modified time
- **Dark / Light Theme** — One-click toggle, auto-persisted
- **Responsive Design** — Card layout on mobile, table on desktop
- **Security** — Path traversal protection; hidden files (`.env`, `.git`, etc.) blocked by default
- **Depth Limiting** — Restrict browsing depth with `--max-depth`
- **Speed Limiting** — Throttle per-request download speed with `--speed-limit`
- **WebDAV (Read-Write)** — Mount as network drive in Finder / Explorer / Nautilus; supports upload, delete, copy, move, mkdir; optional Basic Auth via `--webdav-user` / `--webdav-pass`; disable with `--no-webdav`
- **Access Logging** — Log to stdout, file, or disable entirely

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
      --no-webdav    Disable WebDAV access [default: enabled]
      --webdav-user <WEBDAV_USER>  WebDAV username (enables Basic Auth for WebDAV access; does not affect web UI)
      --webdav-pass <WEBDAV_PASS>  WebDAV password (used with --webdav-user)
  -l, --log <LOG>    Access log output: "stdout", "off", or a file path [default: stdout]
  -h, --help         Print help
```

### Examples

```bash
# Share your Downloads folder on port 3000
echofs -r ~/Downloads -p 3000

# Bind to localhost only (no LAN access)
echofs -b 127.0.0.1

# Show hidden files (e.g., .env, .git)
echofs --show-hidden

# Limit browsing to root directory only
echofs -d 0

# Limit download speed to 1MB/s per request
echofs -s 1m

# Log to file / disable logging
echofs --log /var/log/echofs.log
echofs --log off

# Disable WebDAV
echofs --no-webdav

# Require WebDAV authentication (does not affect browser access)
echofs --webdav-user admin --webdav-pass secret
```

#### WebDAV

WebDAV is enabled by default with full read-write support. Mount the served directory as a network drive:

- **macOS Finder**: Go → Connect to Server (⌘K) → `http://localhost:8080`
- **Windows Explorer**: Map Network Drive → `\\localhost@8080\`
- **Linux Nautilus**: Connect to Server → `dav://localhost:8080`

When `--webdav-user` and `--webdav-pass` are set, all WebDAV operations (browsing, uploading, deleting, etc.) require Basic Auth. **This does not affect browser/web UI access** — the web interface remains open.

Supported WebDAV methods: `PROPFIND`, `OPTIONS`, `LOCK`, `UNLOCK`, `PUT`, `DELETE`, `MKCOL`, `COPY`, `MOVE`, `PROPPATCH`.

When binding to `0.0.0.0`, the console shows all reachable LAN addresses.

## API

EchoFS provides a JSON API for directory listings. Add the `X-Requested-With: XMLHttpRequest` header to get JSON instead of HTML:

| Method | Path | Description |
|--------|------|-------------|
| `GET` `HEAD` | `/` `/{path}` | Directory → HTML or JSON (with XHR header); File → streamed content |
| `PROPFIND` | `/` `/{path}` | WebDAV metadata — `207 Multi-Status` XML (`Depth: 0` or `1`) |
| `OPTIONS` | `/` `/{path}` | WebDAV capability discovery |
| `PUT` | `/{path}` | Upload or overwrite a file (201 Created / 204 No Content) |
| `DELETE` | `/{path}` | Delete a file or directory (204 No Content) |
| `MKCOL` | `/{path}` | Create a directory (201 Created) |
| `COPY` | `/{path}` | Copy a file/directory (`Destination` header required) |
| `MOVE` | `/{path}` | Move/rename a file/directory (`Destination` header required) |
| `PROPPATCH` | `/{path}` | Property update stub (207 Multi-Status) |
| `LOCK` `UNLOCK` | `/` `/{path}` | Lock management (compatibility stubs for Finder/Explorer) |

<details>
<summary>JSON response example</summary>

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
</details>

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
│   └── webdav.rs        WebDAV: PROPFIND/OPTIONS/PUT/DELETE/MKCOL/COPY/MOVE/PROPPATCH, auth, XML generation
└── tests/
    └── integration_test.rs   Integration tests (router, API, file serving, security)
```

## Testing

```bash
cargo test
```

126 tests covering unit tests (each module) and integration tests (full HTTP router via `tower::oneshot`).

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
