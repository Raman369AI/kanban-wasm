#!/usr/bin/env bash
set -e

echo "==> Building WASM app..."
trunk build --release

echo "==> Building native launcher..."
cargo build --release --manifest-path launcher/Cargo.toml

echo ""
echo "Done! Run with:"
echo "  ./launcher/target/release/kanban"
