#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

is_truthy() {
  case "${1:-}" in
    1|true|TRUE|yes|YES|on|ON) return 0 ;;
    *) return 1 ;;
  esac
}

is_falsy() {
  case "${1:-}" in
    0|false|FALSE|no|NO|off|OFF|"") return 0 ;;
    *) return 1 ;;
  esac
}

FORCE=0
DEV_MODE=0
REASON=""

if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
  REASON="--force"
elif [[ "${1:-}" == "--dev" ]]; then
  DEV_MODE=1
  REASON="--dev"
fi

if [[ "${FORCE}" -eq 0 ]] && is_truthy "${FORCE_SPRITES:-}"; then
  FORCE=1
  REASON="FORCE_SPRITES=${FORCE_SPRITES}"
fi

if [[ "${DEV_MODE}" -eq 0 ]] && { [[ "${NODE_ENV:-}" == "development" ]] || [[ "${PIXEL_FORGE_MODE:-}" == "dev" ]] || is_truthy "${PIXEL_FORGE_DEV:-}"; }; then
  DEV_MODE=1
  [[ -z "${REASON}" ]] && REASON="dev mode"
fi

# Robust CI forcing: always regenerate in CI environments (cache only on dev, never in CI).
if [[ -n "${CI:-}" ]] && ! is_falsy "${CI}"; then
  FORCE=1
  DEV_MODE=0
  REASON="CI=${CI}"
fi

FORCE_FLAG=()
if [[ "${FORCE}" -eq 1 ]]; then
  FORCE_FLAG=(--force)
  echo "pixel-forge: forcing full sprite rebuild (${REASON:-forced})"
elif [[ "${DEV_MODE}" -eq 1 ]]; then
  echo "pixel-forge: incremental sprite build enabled (${REASON:-dev mode})"
else
  FORCE=1
  FORCE_FLAG=(--force)
  echo "pixel-forge: forcing full sprite rebuild (cache only on dev)"
fi

# Export core-driven contracts (gitignored) so pixel-forge can build skill/item assets in CI.
cargo run -p enzo-export

# Generate sprite/module assets from pixel-forge contracts and generators.
node tools/pixel-forge/cli.mjs build blocks "${FORCE_FLAG[@]}"
node tools/pixel-forge/cli.mjs build skills "${FORCE_FLAG[@]}"
node tools/pixel-forge/cli.mjs build items "${FORCE_FLAG[@]}"
node tools/pixel-forge/cli.mjs build entities "${FORCE_FLAG[@]}"

echo "sprites ok"
