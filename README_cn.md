# EchoFS

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

[English](README.md)

一个轻量级的 Rust 文件服务器，编译为单个可执行文件。通过浏览器浏览目录、预览媒体文件、复制分享链接。 

这是一个原生AI项目，完全由AI编写。

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
- **安全防护** — 通过 `canonicalize` + `starts_with` 校验防止路径遍历攻击；隐藏文件/目录（`.env`、`.git` 等）默认在目录列表和直接 URL 访问中均被拦截（可通过 `--show-hidden` 允许访问）
- **目录深度限制** — 通过 `--max-depth` 限制用户浏览目录的深度（如 `--max-depth 1` 只允许一级子目录，`0` 只允许根目录）
- **HTML 错误页面** — 浏览器访问返回风格化的错误页面（404、403 等），API 请求返回 JSON 错误
- **异步 I/O** — 所有文件系统操作在 `spawn_blocking` 中运行，避免阻塞异步运行时
- **访问日志** — 请求日志输出到控制台（默认）、文件，或完全关闭

## 快速开始

### 下载预编译二进制文件

你可以直接从 [GitHub Releases](https://github.com/dengsgo/echofs/releases) 下载预编译的二进制文件：

| 平台 | 架构 | 下载链接 |
|------|------|----------|
| Linux | AMD64 (x86_64) | [echofs-linux-amd64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-linux-amd64.tar.gz) |
| Linux | ARM64 | [echofs-linux-arm64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-linux-arm64.tar.gz) |
| macOS | AMD64 (Intel) | [echofs-darwin-amd64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-darwin-amd64.tar.gz) |
| macOS | ARM64 (Apple Silicon) | [echofs-darwin-arm64.tar.gz](https://github.com/dengsgo/echofs/releases/latest/download/echofs-darwin-arm64.tar.gz) |
| Windows | AMD64 (x86_64) | [echofs-windows-amd64.zip](https://github.com/dengsgo/echofs/releases/latest/download/echofs-windows-amd64.zip) |

**快速安装（Linux/macOS）：**
```bash
# 下载并解压（根据你的平台替换链接）
curl -LO https://github.com/dengsgo/echofs/releases/latest/download/echofs-linux-amd64.tar.gz
tar xzf echofs-linux-amd64.tar.gz
sudo mv echofs /usr/local/bin/
```

### 从源码编译

如果你希望从源码编译：

#### 前置条件

- [Rust](https://www.rust-lang.org/tools/install) 1.90 或更高版本

#### 编译

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
  -H, --show-hidden  显示隐藏文件和目录（以 '.' 开头的文件/目录）
  -d, --max-depth <MAX_DEPTH>  目录浏览最大深度（-1 为不限制）[默认: -1]
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

# 显示隐藏文件（如 .env、.git）
echofs --show-hidden

# 限制目录浏览深度为一级子目录
echofs --max-depth 1

# 只允许浏览根目录
echofs -d 0
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
[2025-01-15 10:30:00] 192.168.1.5 GET P / 200 0.8ms
[2025-01-15 10:30:00] 192.168.1.5 GET A / 200 0.5ms
[2025-01-15 10:30:01] 192.168.1.5 GET P /photos/sunset.jpg 206 1.2ms
```

## API 接口

EchoFS 提供 JSON 格式的目录列表 API。在请求任意目录路径时添加 `X-Requested-With: XMLHttpRequest` 请求头即可获取 JSON 响应：

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` `HEAD` | `/` | 根目录 — 返回 HTML 页面或 JSON 列表（需添加 `X-Requested-With: XMLHttpRequest` 请求头） |
| `GET` `HEAD` | `/{path}` | 子目录或文件 — 目录返回 HTML/JSON，文件流式传输；隐藏路径（`.` 开头）默认返回 403，可通过 `--show-hidden` 允许访问 |

不带该请求头时，目录路径返回 HTML 页面；带上该请求头时返回 JSON 数据。

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
├── src/
│   ├── lib.rs           库 crate 根：re-export 所有模块
│   ├── main.rs          入口：CLI 解析、局域网 IP 检测、启动服务器
│   ├── cli.rs           命令行参数定义（clap derive）
│   ├── server.rs        Axum 路由、CORS 中间件、TCP 监听
│   ├── handlers.rs      路由处理器：HTML 页面、JSON API、文件流式传输、错误分发
│   ├── logging.rs       访问日志中间件（stdout / 文件 / 关闭）
│   ├── range.rs         HTTP Range 请求解析与 206 响应构建
│   ├── directory.rs     异步目录遍历、路径安全校验、隐藏文件拦截
│   ├── template.rs      内嵌 HTML/CSS/JS 单页应用及错误页面
│   ├── mime_utils.rs    MIME 类型检测与文件图标映射
│   └── error.rs         统一错误类型，支持双模式响应（HTML/JSON）
└── tests/
    └── integration_test.rs   集成测试（路由、API、文件服务、安全性）
```

## 测试

```bash
# 运行全部测试
cargo test

# 带详细输出运行
cargo test --verbose
```

项目包含超过 120 个自动化测试：

- **单元测试** — 内嵌在各源码模块的 `#[cfg(test)]` 中，覆盖 Range 解析、目录列表、MIME 检测、错误处理、日志、模板内容、隐藏文件拦截、目录深度限制、HTML/JSON 错误分发
- **集成测试** — 位于 `tests/integration_test.rs`，通过 `tower::ServiceExt::oneshot()` 测试完整的 Axum 路由：HTML 服务、JSON API 响应、Range 文件流、路径遍历安全、隐藏文件访问拒绝、目录深度限制、HEAD 方法支持、错误页面格式分发、MIME 类型

CI 在 Linux、macOS、Windows 三平台上运行 `cargo test`，通过后才构建发布产物。

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

## 特别声明

**本项目是一个 100% 由 AI 编写的项目。** 仓库中的每一行代码、文档和配置文件均由 AI 生成。

这个项目有两个目的：一是构建一个满足个人实际需求的工具，二是探索 AI 驱动软件开发的能力边界。这也是为什么选择了作者并不擅长的 Rust 语言来编写——正是为了真正检验 AI 的编码能力。

## 贡献指南

欢迎贡献代码，但有一个原则：**提交的代码必须由 AI 生成。**

本项目是一次 AI 优先开发的实验。手动编写的 Pull Request 将不会被接受。我们认为在 AI 时代，人工编码是低效且过时的方式，这不符合本项目的立意。

提交 PR 时，请同时注明所使用的 **AI 工具和具体模型**。示例：

- `AI Tool: Claude Code / claude-opus-4.6`
- `AI Tool: Cursor / gpt-4o`
- `AI Tool: GitHub Copilot / gemini-2.5-pro`

## 许可证

本项目采用 [Apache License 2.0](LICENSE) 开源。
