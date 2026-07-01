# OmniStream

OmniStream 是一个高性能、低资源占用的 B 站直播录制与上传工具。

## ✨ 特性

*   **全异步架构**: 基于 Tokio + Axum，高并发处理能力。
*   **自动监听**: 支持 Bilibili, Douyu, Huya, Twitch 等主流平台直播状态检测。
    *   **API 优先**: 优先使用官方 API 检测，极低资源消耗。
    *   **FFmpeg 兜底**: API 失效时自动降级为流探测。
*   **持久化存储**: 使用 SQLite 保存任务历史和状态，重启不丢失。
*   **Web UI**: 提供可视化界面管理任务和查看监听状态。

## 🚀 快速开始

### 1. 环境准备

*   **Rust**: 需要安装 Rust 工具链 (Cargo)。
*   **FFmpeg**: 必须安装并加入系统 PATH。
*   **Dioxus CLI** (可选，用于前端开发): `cargo install dioxus-cli`。
*   **Biliup CLI**: 系统需安装 Python 版或 Rust 版 `biliup` 命令行工具，并完成登录。

### 2. 数据目录与初始化

无需预置 `server/config.json`。下载任务、上传模板和录制设置都通过 Web UI/API 写入数据库。

默认运行时数据目录：

- `data/omnistream.db`：SQLite 数据库
- `data/cookies/`：账号 cookies 与 `accounts_meta.json`
- `data/recordings/`：录制文件（按任务名归档）

### 3. 一键启动

项目根目录下提供了启动脚本：

```bash
chmod +x start.sh
./start.sh
```

该脚本会：
1.  自动检查环境。
2.  启动后端 API Server (端口 3000)。
3.  启动前端 Web 界面 (通常端口 8080)。
4.  将日志输出到 `server.log` 和 `web.log`。

停止服务：

```bash
chmod +x stop.sh
./stop.sh
```

### 3.1 编译产物启动（推荐线上）

直接使用二进制启动脚本：

```bash
chmod +x start-bin.sh
./start-bin.sh
```

该脚本会：
1. 启动前自动执行后端编译：`cargo build --release -p server --bin server`。
2. 自动检测并构建 Dioxus 客户端资源，复制到 `target/release/public` 供 Fullstack SSR 使用。
3. 使用 `target/release/server` 启动一个 Rust 进程，同时提供 SSR、Server Functions 和兼容 `/api/*` 路由。
4. 将日志输出到 `server.log`。
5. 可使用 `./stop.sh` 一键停止服务。

### 4. 手动启动

**后端 Server**:
```bash
cargo run -p server --bin server
```

**前端 App (多端同一套代码)**:
```bash
cd web
dx serve --platform web
```

### 5. 多端运行（Dioxus）

前端目录统一为 `web/`（crate 名为 `app`），可直接切换平台：

```bash
cd web
dx serve --platform web
dx serve --platform desktop
dx serve --platform ios
dx serve --platform android
```

多端默认通过 Dioxus Server Functions 与同一服务进程通信；兼容 REST API 仍保留在 `/api/*`。

默认兼容 API 地址规则：

* `web`: `/api`（同域反向代理）
* `android`: `http://10.0.2.2:3000/api`
* 其他本地平台：`http://127.0.0.1:3000/api`

可在构建时覆盖：

```bash
BILIUP_API_URL=http://<server-ip>:3000/api dx serve --platform android
```

## 📘 前端流程文档

- `docs/web-flow.md`：Web 前端模块、状态流转、API 时序与页面逻辑图。
- `docs/release-deployment.md`：GitHub Actions Release、二进制产物和腾讯云部署方案。

## 📦 Release 与二进制部署

推送 `v*` tag 后，GitHub Actions 会生成 Dioxus Web 客户端资源包、Linux Fullstack 服务端整包，以及 Windows/macOS/Linux 桌面端下载包。腾讯云等 Linux 服务器推荐直接下载 `omnistream-linux-amd64.tar.gz` 或 `omnistream-linux-arm64.tar.gz` 运行。

桌面端下载建议：

- Windows x64：下载 `omnistream-desktop-windows-amd64.zip`，解压后运行 `OmniStream.exe`。
- macOS Apple Silicon：下载 `omnistream-desktop-macos-arm64.zip`，解压后运行 `OmniStream`。
- macOS Intel：下载 `omnistream-desktop-macos-amd64.zip`，解压后运行 `OmniStream`。
- Linux x64：下载 `omnistream-desktop-linux-amd64.tar.gz`，解压后运行 `OmniStream`。

桌面端主要用于本地可视化操作，仍需要能访问 OmniStream 后端服务。后端不在本机时，可用 `BILIUP_API_URL=http://<server-ip>:3000/api` 指定地址。

macOS 下载并启动最新版桌面端示例：

```bash
REPO="cancermqiao/OmniStream"
MAC_ARCH="$(uname -m)"

if [ "$MAC_ARCH" = "arm64" ]; then
  ASSET="omnistream-desktop-macos-arm64.zip"
else
  ASSET="omnistream-desktop-macos-amd64.zip"
fi

curl -L -o "$ASSET" "https://github.com/$REPO/releases/latest/download/$ASSET"
rm -rf OmniStream-desktop
unzip -o "$ASSET" -d OmniStream-desktop

cd "OmniStream-desktop/${ASSET%.zip}"
chmod +x OmniStream

# 当前 macOS 产物未做 Apple Developer 签名，如被系统拦截，执行一次解除隔离标记。
xattr -dr com.apple.quarantine OmniStream

# 本机后端示例。若后端部署在服务器，把地址改成 http://<server-ip>:3000/api。
BILIUP_API_URL=http://127.0.0.1:3000/api ./OmniStream
```

其他 PC 端可在 GitHub Release 页面下载：

- Windows x64：`https://github.com/cancermqiao/OmniStream/releases/latest/download/omnistream-desktop-windows-amd64.zip`
- Linux x64：`https://github.com/cancermqiao/OmniStream/releases/latest/download/omnistream-desktop-linux-amd64.tar.gz`

服务器最小运行依赖：

Ubuntu / Debian：

```bash
sudo apt-get update
sudo apt-get install -y ca-certificates curl ffmpeg
```

CentOS Stream / RHEL 系：

```bash
sudo dnf install -y ca-certificates curl dnf-plugins-core epel-release
sudo dnf install -y "https://download1.rpmfusion.org/free/el/rpmfusion-free-release-$(rpm -E %rhel).noarch.rpm"
sudo dnf install -y ffmpeg
```

`streamlink` 统一使用 `uv` 安装。请使用运行 OmniStream 服务的同一个用户执行，确保 `streamlink` 在该用户的 `PATH` 中：

```bash
command -v uv >/dev/null 2>&1 || curl -LsSf https://astral.sh/uv/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
uv tool install streamlink
streamlink --version
```

一键安装最新 Release：

```bash
curl -fsSL https://raw.githubusercontent.com/cancermqiao/OmniStream/main/scripts/install-release.sh \
  | bash -s -- --repo cancermqiao/OmniStream --tag latest --arch linux-amd64 --dir /opt/omnistream
```

注意：命令中的 URL 不要加反引号。如果安装到 `/opt/omnistream` 遇到权限问题，将 `bash` 改为 `sudo bash`。
该命令依赖 GitHub Release 中已经存在 `omnistream-linux-amd64.tar.gz`。如果下载产物返回 404，请先推送新的 `v*` tag，并等待 Release workflow 构建完成。

启动服务：

```bash
cd /opt/omnistream
API_PORT=3000 ./scripts/release-start.sh
```

更多发布与部署细节见 `docs/release-deployment.md`。

## 🛠️ 项目结构

*   `server/`: Rust 后端，负责 Monitor, Recorder, Database。
*   `web/`: Dioxus 多端前端（Web/Desktop/iOS/Android），crate 名为 `app`。
*   `shared/`: 前后端共享的数据结构定义。

## 📝 数据库

项目使用 SQLite（默认 `data/omnistream.db`）存储数据。数据库文件和表结构会在首次启动时自动创建。

可通过环境变量覆盖数据库路径：

```bash
BILIUP_DB_PATH=/tmp/omnistream.db cargo run -p server
```

## ✅ 开发规范与质量门禁

项目使用统一的 Rust 工程规范：

* `rust-toolchain.toml`: 锁定工具链版本，避免环境漂移。
* `LICENSE`: 开源许可证文本（MIT）。
* `CHANGELOG.md`: 变更记录，按版本持续维护。
* `SECURITY.md`: 漏洞提交流程与响应策略。
* `.github/CODEOWNERS`: 代码归属与评审责任边界。
* `.editorconfig`: 统一换行、缩进与结尾空格规则。
* `rustfmt.toml`: 统一格式化策略。
* `clippy.toml`: 统一 Clippy 规则基线。
* `.cargo/config.toml`: 统一开发命令别名。
* `Justfile`: 标准化本地开发命令入口。
* `deny.toml`: 依赖安全与许可证审计规则。
* `.github/workflows/ci.yml`: CI 自动执行格式检查、测试和 Clippy。

本地开发建议在提交前执行：

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

如果安装了 `just`，也可以使用：

```bash
just ci
```

## 🚢 生产部署（推荐）

项目已提供容器化部署骨架，包含：

* `Dockerfile.server`：Fullstack 服务镜像（Rust 服务端二进制 + Dioxus 客户端资源 + `streamlink`/`ffmpeg`）
* `docker-compose.prod.yml`：生产编排
* `deploy/deploy.sh`：发布脚本（DB 备份、健康检查、失败回滚）
* `deploy/.env.prod`：生产环境变量文件（直接编辑）

### 1. 准备环境变量

直接编辑 `deploy/.env.prod`，按需修改：

* `REGISTRY`：镜像仓库地址（如 `ghcr.io/your-org`）
* `IMAGE_TAG`：发布版本（如 `v0.1.0` 或 Git SHA）
* `API_PORT`：Fullstack 服务端口（默认 `3000`）

### 2. 执行发布

```bash
./deploy/deploy.sh
```

脚本行为：

1. 备份 `data/omnistream.db`
2. 拉取并启动新版本容器
3. 健康检查 `http://127.0.0.1:${API_PORT}/api/tasks`
4. 失败时自动回滚到上一个成功版本 tag

### 3. 数据目录

默认挂载 `./data -> /data`，包含：

* `omnistream.db`（SQLite）
* `cookies/`（账号登录态）
* 录制视频文件（`recordings/<下载任务名>/`）

### 4. 访问地址（单机）

* Web UI / Server Functions / API：`http://127.0.0.1:${API_PORT}`（默认 `http://127.0.0.1:3000`）

可通过环境变量覆盖录制目录：

```bash
BILIUP_RECORDINGS_DIR=/data/recordings
```

## 🧱 GitHub Workflow 发布镜像（详细流程）

已提供工作流：`.github/workflows/release-images.yml`。

它会在 GitHub Actions 里完成以下事情：

1. 使用 `Dockerfile.server` 构建 Fullstack 镜像（包含 Rust 服务端二进制、Dioxus 客户端资源、ffmpeg 和 streamlink）。
2. 推送到镜像仓库（默认 GHCR）。

这意味着腾讯云服务器只需要拉取镜像运行，不需要安装 Rust 或前端编译工具。

### 1. 仓库准备

1. 把代码推送到 GitHub。
2. 确认仓库开启了 Actions 权限（默认开启）。
3. 建议仓库可见性先用 private，稳定后再按需调整。

### 2. 工作流触发方式

支持两种：

1. 打 tag 自动触发：`v*`，例如 `v0.1.0`。
2. 手动触发：Actions 页面点击 `Release Images` -> `Run workflow`（可填 `image_tag`）。

### 3. 使用 tag 发布（推荐）

在本地执行：

```bash
git tag v0.1.0
git push origin v0.1.0
```

随后 GitHub Actions 会构建并推送：

1. `ghcr.io/<你的GitHub用户名或组织>/omnistream-server:v0.1.0`
2. `ghcr.io/<你的GitHub用户名或组织>/omnistream-server:latest`

### 4. 腾讯云服务器部署

1. 修改 `deploy/.env.prod`：

```env
REGISTRY=ghcr.io/<你的GitHub用户名或组织>
IMAGE_TAG=v0.1.0
API_PORT=3000
RUST_LOG=info
```

2. 服务器登录 GHCR（如果仓库私有）：

```bash
echo <GHCR_TOKEN> | docker login ghcr.io -u <GitHub用户名> --password-stdin
```

3. 执行部署：

```bash
./deploy/deploy.sh
```

### 5. 关于“是否在 Workflow 中编译二进制”

是的，后端二进制在 `docker build` 的 builder 阶段完成编译，再复制到运行镜像里。

因此服务器端不需要再执行 `cargo build`。

### 6. 如果你要用腾讯云 TCR（可选）

你可以把工作流里的登录步骤改为 TCR 登录（用户名/密码放到 GitHub Secrets），并把 `REGISTRY` 改成：

`<你的TCR域名>`

其余流程不变，腾讯云机器直接 `docker pull` + `./deploy/deploy.sh` 即可。
