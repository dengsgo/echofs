# EchoFS

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.md)

一个轻量级的 Rust 文件服务器，编译为单个可执行文件。通过浏览器浏览目录、预览媒体文件、复制分享链接。

## 功能特性

- **单文件部署** — 编译产物仅 ~1.3 MB，无运行时依赖
- **目录浏览** — 现代化 Web UI，支持面包屑导航和多级目录
- **媒体预览** — 视频、音频、图片在浏览器内直接播放预览（HTML5）
- **链接分享** — 一键复制文件链接，可在 VLC、mpv 等外部播放器中使用
- **Range 请求** — 完整支持 HTTP `206 Partial Content`，视频可拖动进度条，支持断点续传
- **文件排序** — 按名称、大小、创建时间、修改时间升序/降序排列
- **主题切换** — 亮色/暗色一键切换，通过 `localStorage` 持久化
- **响应式布局** — 移动端卡片布局、桌面端表格布局，适配 iPad 和手机
- **毛玻璃效果** — 顶部导航栏 `backdrop-filter` 磨砂玻璃效果
- **局域网地址** — 绑定 `0.0.0.0` 时自动列出所有可访问的局域网 IP
- **安全防护** — 通过 `canonicalize` + `starts_with` 校验防止路径遍历攻击
- **访问日志** — 请求日志输出到控制台（默认）、文件，或完全关闭

## 快速开始

### 前置条件

- [Rust](https://www.rust-lang.org/tools/install) 1.70 或更高版本

### 编译

```bash
git clone https://github.com/dengsgo/echofs.git
cd echofs
cargo build --release
```

编译产物位于 `./target/release/echofs`。

### 运行

```bash
# 在默认端口 8080 上提供当前目录
./target/release/echofs

# 指定目录和端口
./target/release/echofs --root /path/to/files --port 9000

# 自动打开浏览器
./target/release/echofs --open
```

## 命令行参数

```
Usage: echofs [OPTIONS]

Options:
  -r, --root <ROOT>  服务根目录 [默认: .]
  -p, --port <PORT>  监听端口 [默认: 8080]
  -b, --bind <BIND>  绑定地址 [默认: 0.0.0.0]
  -o, --open         自动打开浏览器
  -l, --log <LOG>    访问日志输出："stdout"、"off" 或文件路径 [默认: stdout]
  -h, --help         打印帮助信息
```

### 使用示例

```bash
# 在端口 3000 上分享 Downloads 文件夹
echofs -r ~/Downloads -p 3000

# 仅绑定本机（不允许局域网访问）
echofs -b 127.0.0.1 -p 8080

# 启动并自动打开浏览器
echofs -r /media/videos --open

# 将访问日志写入文件
echofs --log /var/log/echofs.log

# 关闭访问日志
echofs --log off
```

绑定 `0.0.0.0` 时，控制台输出所有可访问地址：

```
EchoFS serving /home/user/files on http://0.0.0.0:8080
Available on:
  http://127.0.0.1:8080
  http://192.168.1.42:8080
Listening on 0.0.0.0:8080
```

访问日志格式（默认开启）：

```
[2025-01-15 10:30:00] 192.168.1.5 GET /api/ls 200 0.8ms
[2025-01-15 10:30:01] 192.168.1.5 GET /photos/sunset.jpg 206 1.2ms
```

## API 接口

EchoFS 提供 JSON 格式的目录列表 API：

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/ls` | 列出根目录 |
| `GET` | `/api/ls/{path}` | 列出子目录 |

响应示例：

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

## 项目结构

```
echofs/
├── Cargo.toml
└── src/
    ├── main.rs          入口：CLI 解析、局域网 IP 检测、启动服务器
    ├── cli.rs           命令行参数定义（clap derive）
    ├── server.rs        Axum 路由、CORS 中间件、TCP 监听
    ├── handlers.rs      路由处理器：HTML 页面、JSON API、文件流式传输
    ├── logging.rs       访问日志中间件（stdout / 文件 / 关闭）
    ├── range.rs         HTTP Range 请求解析与 206 响应构建
    ├── directory.rs     目录遍历、路径安全校验、文件条目收集
    ├── template.rs      内嵌 HTML/CSS/JS 单页应用
    ├── mime_utils.rs    MIME 类型检测与文件图标映射
    └── error.rs         统一错误类型，实现 IntoResponse
```

## 依赖项

| Crate | 用途 |
|-------|------|
| [axum](https://crates.io/crates/axum) | HTTP 框架 |
| [tokio](https://crates.io/crates/tokio) | 异步运行时 |
| [tower-http](https://crates.io/crates/tower-http) | CORS 中间件 |
| [tokio-util](https://crates.io/crates/tokio-util) | 流式文件 I/O |
| [clap](https://crates.io/crates/clap) | 命令行参数解析 |
| [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) | JSON 序列化 |
| [mime_guess](https://crates.io/crates/mime_guess) / [mime](https://crates.io/crates/mime) | MIME 类型检测 |
| [chrono](https://crates.io/crates/chrono) | 日期时间格式化 |
| [percent-encoding](https://crates.io/crates/percent-encoding) | URL 编码 |
| [open](https://crates.io/crates/open) | 打开浏览器 |
| [libc](https://crates.io/crates/libc) | 网络接口枚举 |

## 许可证

本项目采用 [MIT 许可证](LICENSE) 开源。
