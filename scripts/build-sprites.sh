#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Export core-driven contracts (gitignored) so pixel-forge can build skill/item assets in CI.
cargo run -p enzo-export

# Generate sprite/module assets from pixel-forge contracts and generators.
node tools/pixel-forge/cli.mjs build blocks
node tools/pixel-forge/cli.mjs build skills --force
node tools/pixel-forge/cli.mjs build items --force
node tools/pixel-forge/cli.mjs build entities --force

echo "sprites ok"