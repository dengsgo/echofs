# EchoFS

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

[English](README.md)

**分享、播放、挂载——一个二进制全包了。**

EchoFS 是一个用 Rust 编写的轻量级 HTTP 文件服务器：把任意目录变成一个带现代化 Web UI 的迷你站点，影音随点随播，链接和二维码一键分享；同时提供完整读写 WebDAV，让你在 Finder、资源管理器、手机文件管理器里把它当作网络驱动器使用。

## 功能特性

- **单文件部署** — 极小体积，无运行时依赖，即开即用，极速启动
- **多平台支持** — 提供 Linux（AMD64/ARM64）、macOS（Intel/Apple Silicon）、Windows（AMD64）的预编译二进制；跨操作系统、跨架构体验一致
- **适用场景** — 团队、朋友间的文件共享；PC / 手机 / 平板跨设备无缝互联；轻量 NAS：文件管理与大屏在线观影
- **Web 界面** — 内置现代化响应式 Web UI，支持面包屑导航、列表/网格双视图、多列排序。可在浏览器中预览图片（画廊模式，滑动/键盘导航）、播放音频与视频；视频支持 YouTube/Bilibili 风格的长按 3× 倍速播放。一键复制文件链接或二维码用于分享。可在浏览器中直接管理文件（上传、重命名、删除、移动等）。风格迥异有趣的三套主题随心换，支持明暗色模式
- **WebDAV（读写）** — 可在 Finder / 资源管理器 / Nautilus 中挂载为网络驱动器，支持上传、删除、复制、移动、创建目录；可通过 `--webdav-user` / `--webdav-pass` 启用 Basic Auth；通过 `--no-webdav` 禁用
- **HTTP Range** — 视频拖动进度条与断点续传（HTTP 206）
- **安全防护** — 路径遍历防护；隐藏文件（`.env`、`.git` 等）默认拦截；通过 `--max-depth` 限制浏览深度
- **单请求限速** — 通过 `--speed-limit` 限制下载速度
- **访问日志** — 输出到控制台、文件，或完全关闭

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
  -s, --speed-limit <SPEED_LIMIT>  每个请求的速度限制，如 500k、1m、10m [默认: 不限制]
      --no-webdav    禁用 WebDAV 访问 [默认: 启用]
      --webdav-user <WEBDAV_USER>  WebDAV 用户名（设置后所有 WebDAV 操作需认证；不影响网页访问）
      --webdav-pass <WEBDAV_PASS>  WebDAV 密码（与 --webdav-user 配合使用）
  -l, --log <LOG>    访问日志输出："stdout"、"off" 或文件路径 [默认: stdout]
  -h, --help         打印帮助信息
```

### 使用示例

```bash
# 在端口 3000 上分享 Downloads 文件夹
echofs -r ~/Downloads -p 3000

# 仅绑定本机（不允许局域网访问）
echofs -b 127.0.0.1

# 显示隐藏文件（如 .env、.git）
echofs --show-hidden

# 只允许浏览根目录
echofs -d 0

# 限制下载速度为 1MB/s
echofs -s 1m

# 日志写入文件 / 关闭日志
echofs --log /var/log/echofs.log
echofs --log off

# 禁用 WebDAV
echofs --no-webdav

# 设置 WebDAV 认证（不影响网页浏览）
echofs --webdav-user admin --webdav-pass secret
```

#### WebDAV

WebDAV 默认启用，支持完整的读写操作。文件管理器可将服务目录挂载为网络驱动器：

- **macOS Finder**：前往 → 连接服务器（⌘K）→ `http://localhost:8080`
- **Windows 资源管理器**：映射网络驱动器 → `\\localhost@8080\`
- **Linux Nautilus**：连接到服务器 → `dav://localhost:8080`

设置 `--webdav-user` 和 `--webdav-pass` 后，所有 WebDAV 操作（浏览、上传、删除等）都需要 Basic Auth 认证。**网页端文件管理功能（上传/重命名/删除）共享相同的认证凭据** — 操作时会弹出登录对话框。

支持的 WebDAV 方法：`PROPFIND`、`OPTIONS`、`LOCK`、`UNLOCK`、`PUT`、`DELETE`、`MKCOL`、`COPY`、`MOVE`、`PROPPATCH`。

绑定 `0.0.0.0` 时，控制台会显示所有可访问的局域网地址。

## API 接口

EchoFS 提供 JSON 格式的目录列表 API。在请求中添加 `X-Requested-With: XMLHttpRequest` 头即可获取 JSON：

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` `HEAD` | `/` `/{path}` | 目录 → HTML 或 JSON（带 XHR 头）；文件 → 流式传输 |
| `PROPFIND` | `/` `/{path}` | WebDAV 元数据 — `207 Multi-Status` XML（`Depth: 0` 或 `1`） |
| `OPTIONS` | `/` `/{path}` | WebDAV 能力发现 |
| `PUT` | `/{path}` | 上传或覆盖文件（201 Created / 204 No Content） |
| `DELETE` | `/{path}` | 删除文件或目录（204 No Content） |
| `MKCOL` | `/{path}` | 创建目录（201 Created） |
| `COPY` | `/{path}` | 复制文件/目录（需 `Destination` 头） |
| `MOVE` | `/{path}` | 移动/重命名文件/目录（需 `Destination` 头） |
| `PROPPATCH` | `/{path}` | 属性更新存根（207 Multi-Status） |
| `LOCK` `UNLOCK` | `/` `/{path}` | 锁管理（Finder/资源管理器兼容性存根） |

<details>
<summary>JSON 响应示例</summary>

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
  ],
  "webdav": true,
  "webdav_auth": false
}
```
</details>

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
│   ├── template.rs      SPA 组装器：拼接 HTML 标记与内嵌 CSS/JS
│   ├── template.css     内嵌样式表（主题、布局、模态框、Plyr 样式覆盖）
│   ├── template.js      内嵌 SPA 逻辑（路由、文件操作、Plyr 懒加载、3× 倍速手势）
│   ├── mime_utils.rs    MIME 类型检测与文件图标映射
│   ├── error.rs         统一错误类型，支持双模式响应（HTML/JSON）
│   ├── throttle.rs      单请求限速（令牌桶 ThrottledRead 包装器）
│   └── webdav.rs        WebDAV：PROPFIND/OPTIONS/PUT/DELETE/MKCOL/COPY/MOVE/PROPPATCH 处理器、认证、XML 响应生成
└── tests/
    └── integration_test.rs   集成测试（路由、API、文件服务、安全性）
```

## 测试

```bash
cargo test
```

145 项测试覆盖各模块单元测试以及通过 `tower::oneshot` 的完整 HTTP 路由集成测试。

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
