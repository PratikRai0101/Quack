#!/usr/bin/env bash
set -euo pipefail

echo "Building backend (release) and frontend (dist)"

# Build backend
cd backend
cargo build --release
cd ..

# Build frontend
cd worktrees/opencode/frontend
npm ci
npm run build
cd ../..

# Prepare dist/
mkdir -p dist
cp backend/target/release/quack-server dist/
cp -r worktrees/opencode/frontend/dist dist/frontend

echo "Build artifacts in dist/"
