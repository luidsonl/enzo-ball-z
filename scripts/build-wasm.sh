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

if [[ ! -x "node_modules/.bin/vite" ]]; then
  echo "erro: vite não encontrado. Execute: npm install" >&2
  exit 1
fi

rm -rf dist
node_modules/.bin/vite build

# GitHub Pages usa Jekyll por padrão; o arquivo vazio desliga o processamento.
touch dist/.nojekyll

cp "$WASM" dist/enzo-wasm.wasm
mkdir -p dist/assets
cp -r assets/. dist/assets/

echo "build ok: dist/"
