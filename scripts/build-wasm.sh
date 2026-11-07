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

# GitHub Pages usa Jekyll por padrão; o arquivo vazio desliga o processamento
# e evita que o wasm/assets sejam ignorados ou reescritos.
touch dist/.nojekyll

cp "$WASM" dist/
cp web/index.html web/mq_js_bundle.js web/sapp_jsutils.js web/quad-storage.js web/ui.js web/ui.css dist/
cp -r assets/. dist/assets/

echo "build ok: dist/"
