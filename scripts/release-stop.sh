#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

PID_DIR=".run"
SERVER_PID_FILE="$PID_DIR/server.pid"
WEB_PID_FILE="$PID_DIR/web.pid"

stop_pid_file() {
  local name="$1"
  local pid_file="$2"
  if [[ ! -f "$pid_file" ]]; then
    echo "$name is not running"
    return
  fi

  local pid
  pid="$(cat "$pid_file" 2>/dev/null || true)"
  rm -f "$pid_file"
  if [[ -z "$pid" ]]; then
    echo "$name pid file was empty"
    return
  fi

  if ps -p "$pid" >/dev/null 2>&1; then
    kill "$pid" 2>/dev/null || true
    echo "stopped $name (pid=$pid)"
  else
    echo "$name was not running (stale pid=$pid)"
  fi
}

stop_pid_file "OmniStream web" "$WEB_PID_FILE"
stop_pid_file "OmniStream server" "$SERVER_PID_FILE"
