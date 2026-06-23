#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

WEB_PORT="${WEB_PORT:-8080}"
API_PORT="${API_PORT:-3000}"
PID_DIR=".run"
SERVER_PID_FILE="$PID_DIR/server.pid"
WEB_PID_FILE="$PID_DIR/web.pid"
SERVER_BIN="$ROOT_DIR/bin/server"
WEB_DIST_DIR="$ROOT_DIR/web/public"
WEB_PROXY_SCRIPT="$ROOT_DIR/scripts/web_proxy_server.py"
DB_FILE="${BILIUP_DB_PATH:-$ROOT_DIR/data/omnistream.db}"
COOKIES_DIR="${BILIUP_COOKIES_DIR:-$ROOT_DIR/data/cookies}"
RECORDINGS_DIR="${BILIUP_RECORDINGS_DIR:-$ROOT_DIR/data/recordings}"

mkdir -p "$PID_DIR" "$(dirname "$DB_FILE")" "$COOKIES_DIR" "$RECORDINGS_DIR"

ensure_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1"
    exit 1
  fi
}

ensure_not_running() {
  local name="$1"
  local pid_file="$2"
  if [[ ! -f "$pid_file" ]]; then
    return
  fi

  local pid
  pid="$(cat "$pid_file" 2>/dev/null || true)"
  if [[ -n "$pid" ]] && ps -p "$pid" >/dev/null 2>&1; then
    echo "$name is already running (pid=$pid). Run ./scripts/release-stop.sh first."
    exit 1
  fi
  rm -f "$pid_file"
}

ensure_command curl
ensure_command ffmpeg
ensure_command python3
ensure_command streamlink

if [[ ! -x "$SERVER_BIN" ]]; then
  echo "server binary not found or not executable: $SERVER_BIN"
  exit 1
fi

if [[ ! -f "$WEB_DIST_DIR/index.html" ]]; then
  echo "web static files not found: $WEB_DIST_DIR"
  exit 1
fi

if [[ ! -f "$WEB_PROXY_SCRIPT" ]]; then
  echo "web proxy script not found: $WEB_PROXY_SCRIPT"
  exit 1
fi

ensure_not_running "OmniStream server" "$SERVER_PID_FILE"
ensure_not_running "OmniStream web" "$WEB_PID_FILE"

echo "Starting OmniStream server..."
RUST_LOG="${RUST_LOG:-info}" \
BILIUP_DB_PATH="$DB_FILE" \
BILIUP_COOKIES_DIR="$COOKIES_DIR" \
BILIUP_RECORDINGS_DIR="$RECORDINGS_DIR" \
nohup "$SERVER_BIN" > server.log 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" > "$SERVER_PID_FILE"

echo "Waiting for API on http://127.0.0.1:${API_PORT}/api/tasks ..."
for _ in {1..60}; do
  if curl -fsS "http://127.0.0.1:${API_PORT}/api/tasks" >/dev/null 2>&1; then
    break
  fi
  if ! ps -p "$SERVER_PID" >/dev/null 2>&1; then
    echo "server exited unexpectedly"
    tail -n 80 server.log || true
    exit 1
  fi
  sleep 1
done

if ! curl -fsS "http://127.0.0.1:${API_PORT}/api/tasks" >/dev/null 2>&1; then
  echo "server did not become healthy in time"
  tail -n 80 server.log || true
  exit 1
fi

echo "Starting OmniStream web..."
nohup python3 "$WEB_PROXY_SCRIPT" \
  --web-dir "$WEB_DIST_DIR" \
  --web-port "$WEB_PORT" \
  --api-host "127.0.0.1" \
  --api-port "$API_PORT" \
  > web.log 2>&1 &
WEB_PID=$!
echo "$WEB_PID" > "$WEB_PID_FILE"

echo "OmniStream started."
echo "Web:  http://127.0.0.1:${WEB_PORT}"
echo "API:  http://127.0.0.1:${API_PORT}"
echo "Logs: tail -f server.log web.log"
