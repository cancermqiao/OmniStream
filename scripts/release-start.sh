#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

API_PORT="${API_PORT:-3000}"
PID_DIR=".run"
SERVER_PID_FILE="$PID_DIR/server.pid"
SERVER_BIN="$ROOT_DIR/bin/server"
DB_FILE="${BILIUP_DB_PATH:-$ROOT_DIR/data/omnistream.db}"
COOKIES_DIR="${BILIUP_COOKIES_DIR:-$ROOT_DIR/data/cookies}"
RECORDINGS_DIR="${BILIUP_RECORDINGS_DIR:-$ROOT_DIR/data/recordings}"

mkdir -p "$PID_DIR" "$(dirname "$DB_FILE")" "$COOKIES_DIR" "$RECORDINGS_DIR" logs

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
ensure_command streamlink

if [[ ! -x "$SERVER_BIN" ]]; then
  echo "server binary not found or not executable: $SERVER_BIN"
  exit 1
fi

ensure_not_running "OmniStream server" "$SERVER_PID_FILE"

echo "Starting OmniStream server with Dioxus Fullstack SSR..."
RUST_LOG="${RUST_LOG:-info}" \
API_PORT="$API_PORT" \
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

echo "OmniStream started."
echo "Web/API: http://127.0.0.1:${API_PORT}"
echo "Logs:    tail -f server.log"
echo "Level logs: tail -f logs/info.log logs/warn.log logs/error.log"
