#!/usr/bin/env bash
set -euo pipefail

# Simple e2e smoke script: starts backend, waits for health, runs frontend smoke test, then stops backend.

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
FRONTEND_DIR="$ROOT_DIR/worktrees/opencode/frontend"
LOG=/tmp/quack-e2e.log

echo "Starting backend..." | tee $LOG
cd "$BACKEND_DIR"
# Start background server
cargo run >$LOG 2>&1 &
PID=$!
sleep 1

# Wait for health
echo "Waiting for backend health..." | tee -a $LOG
for i in {1..20}; do
  if curl -sS http://127.0.0.1:3001/api/health >/dev/null 2>&1; then
    echo "Backend healthy" | tee -a $LOG
    break
  fi
  sleep 0.5
done

if ! curl -sS http://127.0.0.1:3001/api/health >/dev/null 2>&1; then
  echo "Backend did not become healthy in time" | tee -a $LOG
  kill $PID || true
  exit 2
fi

# Run frontend smoke test
echo "Running frontend smoke test..." | tee -a $LOG
cd "$FRONTEND_DIR"
# Ensure node deps
npm install --no-audit --no-fund >> $LOG 2>&1 || true
node scripts/smoke-test.js >> $LOG 2>&1
RET=$?

# Cleanup
echo "Stopping backend (pid $PID)" | tee -a $LOG
kill $PID || true

if [ $RET -ne 0 ]; then
  echo "Smoke test failed (exit $RET). See $LOG" | tee -a $LOG
  exit $RET
fi

echo "Smoke test succeeded" | tee -a $LOG
exit 0
