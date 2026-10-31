#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET="wasm32-unknown-unknown"
WASM="target/$TARGET/release/enzo-wasm.wasm"

cargo build -p enzo-wasm --release --target "$TARGET"

if [[ ! -f "$WASM" ]]; then
  echo "erro: $WASM não foi gerado" >&2
  exit 1
fi

rm -rf dist
mkdir -p dist/assets

cp "$WASM" dist/
cp web/index.html web/mq_js_bundle.js web/sapp_jsutils.js web/quad-storage.js dist/
cp -r assets/. dist/assets/

echo "build ok: dist/"
