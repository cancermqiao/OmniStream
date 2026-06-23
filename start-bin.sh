#!/bin/bash

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== OmniStream Binary Startup ===${NC}"

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVER_BIN="target/release/server"
WEB_DIST_DIR=""
API_PORT="${API_PORT:-3000}"
PID_DIR=".run"
SERVER_PID_FILE="${PID_DIR}/server.pid"
DB_FILE="${ROOT_DIR}/data/omnistream.db"
COOKIES_DIR="${ROOT_DIR}/data/cookies"
RECORDINGS_DIR="${ROOT_DIR}/data/recordings"

mkdir -p "$PID_DIR" "$ROOT_DIR/data" "$COOKIES_DIR" "$RECORDINGS_DIR"

ensure_not_running() {
    local name="$1"
    local pid_file="$2"

    if [ ! -f "$pid_file" ]; then
        return 0
    fi

    local pid
    pid="$(cat "$pid_file" 2>/dev/null || true)"
    if [ -z "$pid" ]; then
        rm -f "$pid_file"
        return 0
    fi

    if ps -p "$pid" > /dev/null 2>&1; then
        echo -e "${RED}Error: ${name} is already running (PID: ${pid}).${NC}"
        echo "Run ./stop.sh first, then retry ./start-bin.sh"
        exit 1
    fi

    echo "${name}: stale pid file found, cleaning up (${pid_file})."
    rm -f "$pid_file"
}

ensure_not_running "Backend Server" "$SERVER_PID_FILE"

resolve_web_dir() {
    local candidates=(
        "target/dx/app/release/web/public"
        "target/dx/web/release/web/public"
        "web/dist"
    )

    for dir in "${candidates[@]}"; do
        if [ -f "${dir}/index.html" ]; then
            WEB_DIST_DIR="$dir"
            return 0
        fi
    done
    return 1
}

# 1. 检查运行依赖
if ! command -v ffmpeg &> /dev/null; then
    echo -e "${RED}Error: ffmpeg is not found in PATH.${NC}"
    echo "Please install ffmpeg first."
    exit 1
fi

if ! command -v streamlink &> /dev/null; then
    echo -e "${RED}Error: streamlink is not found in PATH.${NC}"
    echo "Please install streamlink first."
    exit 1
fi

if ! command -v curl &> /dev/null; then
    echo -e "${RED}Error: curl is not installed.${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo is required to build backend binary.${NC}"
    exit 1
fi

# 2. 自动编译后端二进制
echo -e "${GREEN}Building backend binary (release)...${NC}"
cargo build --release -p server --bin server || {
    echo -e "${RED}Error: failed to build backend binary.${NC}"
    exit 1
}

if [ ! -x "$SERVER_BIN" ]; then
    echo -e "${RED}Error: backend binary not found after build: ${SERVER_BIN}.${NC}"
    exit 1
fi

# 3. 检查/构建前端编译产物
rebuild_web_assets() {
    echo -e "${GREEN}Building release web assets...${NC}"
    (
        cd web
        dx build --platform web --release
    ) || {
        echo -e "${RED}Error: dx build failed.${NC}"
        exit 1
    }

    if ! resolve_web_dir; then
        echo -e "${RED}Error: web index.html still not found after build.${NC}"
        echo "Checked: target/dx/app/release/web/public, target/dx/web/release/web/public, web/dist"
        exit 1
    fi
}

web_assets_stale() {
    local index_html="$1"
    if [ ! -f "$index_html" ]; then
        echo "stale"
        return
    fi

    for path in "$ROOT_DIR/web/src" "$ROOT_DIR/web/assets" "$ROOT_DIR/web/Cargo.toml" "$ROOT_DIR/web/Dioxus.toml"; do
        if [ ! -e "$path" ]; then
            continue
        fi
        if [ -f "$path" ] && [ "$path" -nt "$index_html" ]; then
            echo "stale"
            return
        fi
        if [ -d "$path" ] && find "$path" -type f -newer "$index_html" | grep -q .; then
            echo "stale"
            return
        fi
    done

    echo "fresh"
}

if ! resolve_web_dir; then
    if ! command -v dx &> /dev/null; then
        echo -e "${RED}Error: web build artifacts not found and dx is not installed.${NC}"
        echo "Install dioxus-cli first:"
        echo "  cargo install dioxus-cli"
        exit 1
    fi
    rebuild_web_assets
else
    INDEX_HTML="${WEB_DIST_DIR}/index.html"
    if [ "$(web_assets_stale "$INDEX_HTML")" = "stale" ]; then
        if ! command -v dx &> /dev/null; then
            echo -e "${RED}Error: web sources changed but dioxus-cli (dx) is not installed.${NC}"
            echo "Current assets are stale: ${WEB_DIST_DIR}"
            echo "Install dioxus-cli and rebuild: cargo install dioxus-cli && (cd web && dx build --platform web --release)"
            exit 1
        fi
        echo -e "${BLUE}Detected stale web assets, rebuilding...${NC}"
        rebuild_web_assets
    fi
fi

echo "Using web static directory: ${WEB_DIST_DIR}"

# 3.1 确保 favicon 存在并注入到静态 index.html（部分浏览器不会识别运行时注入的 icon）
FAVICON_SRC="${ROOT_DIR}/web/assets/favicon.svg"
INDEX_HTML="${WEB_DIST_DIR}/index.html"
if [ -f "$FAVICON_SRC" ]; then
    mkdir -p "${WEB_DIST_DIR}/assets"
    cp -f "$FAVICON_SRC" "${WEB_DIST_DIR}/assets/favicon.svg"

    if [ -f "$INDEX_HTML" ] && ! grep -q 'rel="icon"' "$INDEX_HTML"; then
        perl -0pi -e 's#</head>#    <link rel="icon" type="image/svg+xml" href="/assets/favicon.svg">\n    <link rel="shortcut icon" href="/assets/favicon.svg">\n</head>#s' "$INDEX_HTML"
    fi
else
    echo -e "${RED}Warning: favicon source not found: ${FAVICON_SRC}${NC}"
fi

# 4. 启动后端（内置 Web 静态文件托管）
echo -e "${GREEN}Starting Backend Server Binary with embedded Web UI...${NC}"
API_PORT="$API_PORT" BILIUP_WEB_DIR="$WEB_DIST_DIR" BILIUP_DB_PATH="$DB_FILE" BILIUP_COOKIES_DIR="$COOKIES_DIR" BILIUP_RECORDINGS_DIR="$RECORDINGS_DIR" nohup "$SERVER_BIN" > server.log 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" > "$SERVER_PID_FILE"
echo "Backend Server PID: $SERVER_PID"

echo "Waiting for Server to be ready on port ${API_PORT}..."
MAX_RETRIES=60
COUNT=0
SERVER_READY=false

while [ $COUNT -lt $MAX_RETRIES ]; do
    if lsof -i:${API_PORT} -t >/dev/null 2>&1 || curl -s "http://127.0.0.1:${API_PORT}/api/tasks" >/dev/null 2>&1; then
        SERVER_READY=true
        break
    fi
    if ! ps -p $SERVER_PID > /dev/null; then
        echo -e "${RED}Server process exited unexpectedly. Check server.log for details.${NC}"
        tail -n 50 server.log
        exit 1
    fi
    echo -n "."
    sleep 1
    ((COUNT++))
done

echo ""

if [ "$SERVER_READY" != true ]; then
    echo -e "${RED}Server failed to start within $MAX_RETRIES seconds.${NC}"
    tail -n 50 server.log
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

echo -e "${GREEN}Server is UP and Ready!${NC}"

echo -e "${BLUE}=== Services Started (Detached) ===${NC}"
echo "Web/API URL: http://127.0.0.1:${API_PORT}"
echo "Server PID:  ${SERVER_PID}"
echo "Server logs: tail -f server.log"
echo "Stop all:    ./stop.sh"
exit 0
