#!/bin/bash
set -e
echo "Building WASM..."
wasm-pack build --target web --out-dir ../grid-app/src/app/wasm --release
echo "Done! WASM output in grid-app/src/app/wasm/"
