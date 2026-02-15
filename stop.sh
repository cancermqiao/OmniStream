#!/bin/bash

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== OmniStream Stop ===${NC}"

PID_DIR=".run"
SERVER_PID_FILE="${PID_DIR}/server.pid"
WEB_PID_FILE="${PID_DIR}/web.pid"

stop_by_pid_file() {
    local name="$1"
    local pid_file="$2"

    if [ ! -f "$pid_file" ]; then
        echo "${name}: pid file not found (${pid_file}), skip."
        return 0
    fi

    local pid
    pid="$(cat "$pid_file" 2>/dev/null || true)"
    if [ -z "$pid" ]; then
        echo "${name}: empty pid file, cleanup."
        rm -f "$pid_file"
        return 0
    fi

    if ps -p "$pid" > /dev/null 2>&1; then
        kill "$pid" 2>/dev/null || true
        echo -e "${GREEN}${name} stopped (PID: ${pid}).${NC}"
    else
        echo "${name}: process ${pid} not running, cleanup."
    fi

    rm -f "$pid_file"
}

stop_by_pid_file "Backend Server" "$SERVER_PID_FILE"
stop_by_pid_file "Frontend Web" "$WEB_PID_FILE"

if [ -d "$PID_DIR" ] && [ -z "$(ls -A "$PID_DIR" 2>/dev/null)" ]; then
    rmdir "$PID_DIR" 2>/dev/null || true
fi

echo -e "${BLUE}Done.${NC}"
