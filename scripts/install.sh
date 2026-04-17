#!/usr/bin/env bash
set -euo pipefail

# Simple installer script for local development. Builds backend and frontend.
# For production, use prebuilt release artifacts.

echo "Installing Quack v2 (local build)..."

# Build everything
./scripts/build.sh

echo "Quack built. You can run the frontend in dev mode or copy dist artifacts as needed."

echo "To run backend (release): ./dist/quack-server"
echo "To run frontend (dev): cd worktrees/opencode/frontend && npm run dev -- --cmd 'ls /nonexistent'"
