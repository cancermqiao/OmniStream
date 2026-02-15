#!/bin/bash

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== OmniStream Startup ===${NC}"
ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
PID_DIR=".run"
SERVER_PID_FILE="${PID_DIR}/server.pid"
WEB_PID_FILE="${PID_DIR}/web.pid"

mkdir -p "$PID_DIR"

# 1. 检查环境
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust (cargo) is not installed.${NC}"
    exit 1
fi

if ! command -v ffmpeg &> /dev/null; then
    echo -e "${RED}Error: ffmpeg is not found in PATH.${NC}"
    echo "Please install ffmpeg first."
    exit 1
fi

# 2. 准备数据库文件 (如果不存在)
# server/src/db.rs 会自动创建表，但我们需要确保 sqlite 文件能被创建
DB_FILE="${ROOT_DIR}/data/omnistream.db"
COOKIES_DIR="${ROOT_DIR}/data/cookies"
RECORDINGS_DIR="${ROOT_DIR}/data/recordings"
mkdir -p "${ROOT_DIR}/data" "${COOKIES_DIR}" "${RECORDINGS_DIR}"

if [ ! -f "$DB_FILE" ]; then
    echo -e "${GREEN}Database file not found, it will be created automatically by the server.${NC}"
fi

# 3. 启动后端 Server
echo -e "${GREEN}Starting Backend Server...${NC}"
cd server
BILIUP_DB_PATH="$DB_FILE" BILIUP_COOKIES_DIR="$COOKIES_DIR" BILIUP_RECORDINGS_DIR="$RECORDINGS_DIR" nohup cargo run --bin server > ../server.log 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" > "../${SERVER_PID_FILE}"
echo "Backend Server PID: $SERVER_PID"
cd ..

# 循环检查端口 3000 是否已启动 (最多等待 60 秒)
echo "Waiting for Server to be ready on port 3000..."
MAX_RETRIES=60
COUNT=0
SERVER_READY=false

while [ $COUNT -lt $MAX_RETRIES ]; do
    if lsof -i:3000 -t >/dev/null 2>&1 || curl -s http://localhost:3000/api/tasks >/dev/null 2>&1; then
        SERVER_READY=true
        break
    fi
    # 检查进程是否意外退出
    if ! ps -p $SERVER_PID > /dev/null; then
        echo -e "${RED}Server process exited unexpectedly. Check server.log for details.${NC}"
        cat server.log
        exit 1
    fi
    echo -n "."
    sleep 1
    ((COUNT++))
done

echo "" # 换行

if [ "$SERVER_READY" = true ]; then
    echo -e "${GREEN}Server is UP and Ready!${NC}"
else
    echo -e "${RED}Server failed to start within $MAX_RETRIES seconds.${NC}"
    echo "Check server.log for details:"
    tail -n 10 server.log
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# 4. 启动前端 Web
echo -e "${GREEN}Starting Frontend Web...${NC}"
cd web
# 确保 wasm-pack 或 dioxus cli 已安装，这里假设使用 dioxus cli
if command -v dx &> /dev/null; then
    # Dioxus CLI 启动
    nohup dx serve > ../web.log 2>&1 &
    WEB_PID=$!
    echo "Frontend Web PID: $WEB_PID"
else
    echo -e "${RED}Dioxus CLI (dx) not found.${NC}"
    echo "Please install dioxus-cli: cargo install dioxus-cli"
    kill $SERVER_PID
    rm -f "$SERVER_PID_FILE"
    exit 1
fi
echo "$WEB_PID" > "../${WEB_PID_FILE}"
cd ..

echo -e "${BLUE}=== Services Started ===${NC}"
echo "Server logs: tail -f server.log"
echo "Web logs:    tail -f web.log"
echo "Press Ctrl+C to stop all services."

cleanup() {
    echo "Stopping services..."
    kill "$SERVER_PID" "$WEB_PID" 2>/dev/null || true
    rm -f "$SERVER_PID_FILE" "$WEB_PID_FILE"
}

# 捕获退出信号以清理进程
trap cleanup INT TERM EXIT

# 保持脚本运行
wait
