#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRONTEND_DIR="$ROOT_DIR/frontend"
WASM_DIR="$ROOT_DIR/crates/calc-wasm"
PORT="${PORT:-5173}"
URL="http://localhost:${PORT}"
LOG_DIR="$ROOT_DIR/.run"
PID_FILE="$LOG_DIR/vite.pid"
LOG_FILE="$LOG_DIR/vite.log"

mkdir -p "$LOG_DIR"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

need_cmd npm
need_cmd wasm-pack

echo "Building WASM package..."
cd "$WASM_DIR"
wasm-pack build --target web --out-dir ../../frontend/public/pkg >/dev/null

echo "Installing frontend dependencies if needed..."
cd "$FRONTEND_DIR"
if [[ ! -d node_modules ]]; then
  npm install
fi

if [[ -f "$PID_FILE" ]]; then
  OLD_PID="$(cat "$PID_FILE")"
  if kill -0 "$OLD_PID" >/dev/null 2>&1; then
    echo "Stopping existing Vite server ($OLD_PID)..."
    kill "$OLD_PID" || true
    sleep 1
  fi
fi

echo "Starting Vite dev server on port $PORT..."
nohup npm run dev -- --host 127.0.0.1 --port "$PORT" >"$LOG_FILE" 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" >"$PID_FILE"

for _ in {1..40}; do
  if curl -fsS "$URL" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done

echo "Opening $URL"
if command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$URL" >/dev/null 2>&1 || true
elif command -v sensible-browser >/dev/null 2>&1; then
  sensible-browser "$URL" >/dev/null 2>&1 || true
else
  echo "Open this URL in your browser: $URL"
fi

echo "Vite log: $LOG_FILE"
echo "Vite pid: $SERVER_PID"