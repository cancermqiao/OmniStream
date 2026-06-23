# Release 与生产部署设计

本文档说明 OmniStream 在 GitHub Actions 上的质量校验、Release 产物构建，以及腾讯云等 Linux 服务器上的二进制部署方式。

## 目标

- 每次 push / PR 自动执行质量门禁，尽早发现格式、测试、Clippy、WASM 前端构建问题。
- 每次推送 `v*` tag 自动生成 GitHub Release。
- Release 附带 Web 静态包、Linux 服务端整包、PC 桌面端包。
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
- `web-static`：构建 `dx build --platform web --release`，产出 `omnistream-web-static.tar.gz`。
- `linux-package`：产出 `omnistream-linux-amd64.tar.gz` 和 `omnistream-linux-arm64.tar.gz`。
- `desktop-package`：产出 Linux、macOS、Windows 桌面端二进制包。
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
│   └── server
├── web/
│   └── public/
├── scripts/
│   ├── release-start.sh
│   ├── release-stop.sh
│   └── web_proxy_server.py
└── data/
```

### PC 桌面端产物

- `omnistream-desktop-linux-amd64.tar.gz`
- `omnistream-desktop-macos-arm64.tar.gz`
- `omnistream-desktop-windows-amd64.zip`

桌面端用于本地可视化操作，仍然需要能访问后端 API。

### 手机端产物

Android 产物需要手动触发 Release workflow 并勾选 `build_android`。默认不在 tag Release 中强制构建手机端，原因是正式上架或分发需要签名证书、包名、权限和渠道策略。当前设计先产出未签名包，后续可接入 GitHub Secrets 中的 keystore 完成正式签名。

iOS 产物需要 Apple Developer 证书和 provisioning profile，建议后续单独接入 macOS runner 和签名密钥。

## 腾讯云服务器部署

### 1. 安装运行依赖

Release 二进制不要求安装 Rust，但运行录制需要系统工具：

```bash
sudo apt-get update
sudo apt-get install -y ca-certificates curl ffmpeg python3 streamlink
```

### 2. 一键安装 Release

将 `OWNER/REPO` 替换为实际 GitHub 仓库。

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/REPO/main/scripts/install-release.sh \
  | bash -s -- --repo OWNER/REPO --tag latest --arch linux-amd64 --dir /opt/omnistream
```

ARM 服务器使用：

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/REPO/main/scripts/install-release.sh \
  | bash -s -- --repo OWNER/REPO --tag latest --arch linux-arm64 --dir /opt/omnistream
```

### 3. 启动

```bash
cd /opt/omnistream
WEB_PORT=8080 API_PORT=3000 ./scripts/release-start.sh
```

启动后：

- Web UI: `http://服务器IP:8080`
- API: `http://服务器IP:3000`
- 数据库: `/opt/omnistream/data/omnistream.db`
- 录制文件: `/opt/omnistream/data/recordings`
- 日志: `/opt/omnistream/server.log`、`/opt/omnistream/web.log`

### 4. 停止

```bash
cd /opt/omnistream
./scripts/release-stop.sh
```

### 5. 升级

再次执行安装命令即可覆盖程序文件，不会删除 `data/`。

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/REPO/main/scripts/install-release.sh \
  | bash -s -- --repo OWNER/REPO --tag v0.2.0 --arch linux-amd64 --dir /opt/omnistream

cd /opt/omnistream
./scripts/release-stop.sh
./scripts/release-start.sh
```

## Docker 部署备选

如果服务器使用 Docker，可以继续使用 `release-images.yml` 发布到 GHCR 的镜像：

- `ghcr.io/<owner>/omnistream-server:<tag>`
- `ghcr.io/<owner>/omnistream-web:<tag>`

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
