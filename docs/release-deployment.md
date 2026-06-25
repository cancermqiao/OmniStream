# Release 与生产部署设计

本文档说明 OmniStream 在 GitHub Actions 上的质量校验、Release 产物构建，以及腾讯云等 Linux 服务器上的二进制部署方式。

## 目标

- 每次 push / PR 自动执行质量门禁，尽早发现格式、测试、Clippy、WASM 前端构建问题。
- 每次推送 `v*` tag 自动生成 GitHub Release。
- Release 附带 Dioxus Web 客户端资源包、Linux Fullstack 服务端整包、Windows/macOS/Linux 桌面端包。
- 腾讯云服务器可以不安装 Rust，不拉源码，直接下载 Release 二进制整包运行。
- Docker 镜像发布继续保留，适合已有 Docker Compose 部署的环境。

## 工作流

### CI

`.github/workflows/ci.yml` 在 push 和 PR 时运行：

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p app --target wasm32-unknown-unknown`
- `cargo deny check advisories bans sources`

这里单独加入 WASM 检查，是因为普通 `cargo test` 不会覆盖 Dioxus Web 的 `wasm32-unknown-unknown` 编译路径。

### Release

`.github/workflows/release.yml` 在推送 `v*` tag 或手动触发时运行：

- `quality-gate`：Release 前再次执行格式、测试、Clippy 和 WASM 检查。
- `web-static`：构建 Dioxus Web 客户端资源，供 Fullstack 服务端整包使用。
- `linux-package`：产出 `omnistream-linux-amd64.tar.gz` 和 `omnistream-linux-arm64.tar.gz`。
- `desktop-package`：产出 Linux x64、macOS Intel、macOS Apple Silicon、Windows x64 桌面端下载包。
- `android-package`：手动触发且勾选 `build_android` 时，尝试产出未签名 Android 包。
- `publish-github-release`：汇总所有产物并发布到 GitHub Release。

## Release 产物

### 推荐服务器产物

腾讯云 Linux 服务器优先使用：

- `omnistream-linux-amd64.tar.gz`：普通 x86_64 云服务器。
- `omnistream-linux-arm64.tar.gz`：ARM 云服务器。

整包目录结构：

```text
omnistream-linux-amd64/
├── bin/
│   ├── server
│   └── public/
├── scripts/
│   ├── release-start.sh
│   └── release-stop.sh
└── data/
```

Release 整包只启动一个 Rust 后端进程。后端通过 Dioxus Fullstack 提供 SSR、客户端资源和 Server Functions；保留的 `/api/*` REST 路由仅用于兼容旧调用。

Dioxus 默认从服务端二进制同级的 `public/` 目录加载客户端资源，因此 Release 包把资源放在 `bin/public/`，启动脚本不再需要额外配置 Web 静态目录。

### PC 桌面端产物

- `omnistream-desktop-linux-amd64.tar.gz`
- `omnistream-desktop-macos-amd64.zip`
- `omnistream-desktop-macos-arm64.zip`
- `omnistream-desktop-windows-amd64.zip`

桌面端用于本地可视化操作，仍然需要能访问后端 API。每个桌面端包都包含：

- `OmniStream` 或 `OmniStream.exe`：桌面端可执行文件。
- `README.txt`：当前平台运行说明、远程后端地址配置方式和 macOS 未签名提示。

下载选择：

- Windows 10/11 x64：下载 `omnistream-desktop-windows-amd64.zip`，解压后双击 `OmniStream.exe`。
- macOS Apple Silicon/M 系列芯片：下载 `omnistream-desktop-macos-arm64.zip`，解压后运行 `OmniStream`。
- macOS Intel 芯片：下载 `omnistream-desktop-macos-amd64.zip`，解压后运行 `OmniStream`。
- Linux x64 桌面环境：下载 `omnistream-desktop-linux-amd64.tar.gz`，解压后运行 `OmniStream`。

如果后端部署在远程服务器，启动桌面端前设置：

```bash
BILIUP_API_URL=http://<server-ip>:3000/api ./OmniStream
```

macOS 产物当前未做 Apple Developer 签名。如果系统提示来源不明，可在解压目录执行：

```bash
xattr -dr com.apple.quarantine OmniStream
```

### 手机端产物

Android 产物需要手动触发 Release workflow 并勾选 `build_android`。默认不在 tag Release 中强制构建手机端，原因是正式上架或分发需要签名证书、包名、权限和渠道策略。当前设计先产出未签名包，后续可接入 GitHub Secrets 中的 keystore 完成正式签名。

iOS 产物需要 Apple Developer 证书和 provisioning profile，建议后续单独接入 macOS runner 和签名密钥。

## 腾讯云服务器部署

### 1. 安装运行依赖

Release 二进制不要求安装 Rust，但运行录制需要系统工具：

```bash
sudo apt-get update
sudo apt-get install -y ca-certificates curl ffmpeg streamlink
```

### 2. 一键安装 Release

OmniStream 官方仓库可直接使用：

```bash
curl -fsSL https://raw.githubusercontent.com/cancermqiao/OmniStream/main/scripts/install-release.sh \
  | bash -s -- --repo cancermqiao/OmniStream --tag latest --arch linux-amd64 --dir /opt/omnistream
```

注意：命令中的 URL 不要加反引号。安装脚本会直接下载 GitHub Release asset，不依赖 GitHub API，避免匿名 API rate limit 导致 403。
该命令依赖 GitHub Release 中已经存在 `omnistream-linux-amd64.tar.gz`。如果下载产物返回 404，请先推送新的 `v*` tag，并等待 Release workflow 构建完成。

如果使用 Fork 仓库，将 `cancermqiao/OmniStream` 替换为实际 GitHub 仓库。

ARM 服务器使用：

```bash
curl -fsSL https://raw.githubusercontent.com/cancermqiao/OmniStream/main/scripts/install-release.sh \
  | bash -s -- --repo cancermqiao/OmniStream --tag latest --arch linux-arm64 --dir /opt/omnistream
```

### 3. 启动

```bash
cd /opt/omnistream
API_PORT=3000 ./scripts/release-start.sh
```

启动后：

- Web UI: `http://服务器IP:3000`
- Server Functions: 由 Dioxus 自动生成并挂载在同一服务进程内。
- Legacy API: `http://服务器IP:3000/api`
- 数据库: `/opt/omnistream/data/omnistream.db`
- 录制文件: `/opt/omnistream/data/recordings`
- 日志: `/opt/omnistream/server.log`

### 4. 停止

```bash
cd /opt/omnistream
./scripts/release-stop.sh
```

### 5. 升级

再次执行安装命令即可覆盖程序文件，不会删除 `data/`。

```bash
curl -fsSL https://raw.githubusercontent.com/cancermqiao/OmniStream/main/scripts/install-release.sh \
  | bash -s -- --repo cancermqiao/OmniStream --tag v0.2.0 --arch linux-amd64 --dir /opt/omnistream

cd /opt/omnistream
./scripts/release-stop.sh
./scripts/release-start.sh
```

## Docker 部署备选

如果服务器使用 Docker，可以继续使用 `release-images.yml` 发布到 GHCR 的镜像：

- `ghcr.io/<owner>/omnistream-server:<tag>`

部署方式：

```bash
cd /path/to/OmniStream
vim deploy/.env.prod
./deploy/deploy.sh
```

Docker 适合多实例、反向代理、自动重启和统一日志采集；二进制整包适合轻量服务器和最小依赖部署。

## 推荐发布流程

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p app --target wasm32-unknown-unknown

git tag v0.1.0
git push origin v0.1.0
```

Tag 推送后等待 GitHub Actions 的 `Release` workflow 完成，再从 GitHub Release 页面下载产物。
