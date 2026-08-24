#!/bin/bash
set -e

echo "========================================"
echo "  Building Linux (x86_64) Release"
echo "========================================"

echo "[1/1] Compiling..."
cargo build --release

echo ""
echo "========================================"
echo "[OK] Linux build complete: target/release/cube_3d"
echo "========================================"
