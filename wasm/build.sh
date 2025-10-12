#!/bin/bash
# Build script for WASM target
# Requires wasm-pack: cargo install wasm-pack

set -e

echo "Building ButaButi for WebAssembly..."

# Build for web target
wasm-pack build --target web --features wasm --out-dir wasm/pkg ..

echo "Build complete! Output in wasm/pkg/"
echo ""
echo "To test locally:"
echo "  cd wasm"
echo "  python -m http.server 8000"
echo "  Open http://localhost:8000 in browser"
