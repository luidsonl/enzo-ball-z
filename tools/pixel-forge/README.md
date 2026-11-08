# pixel-forge

Project-local asset framework for EnzoBallZ.

See the full framework documentation:
- [`../../docs/PIXEL_FORGE.md`](../../docs/PIXEL_FORGE.md)
- Related plan: [`../../docs/ANIMATION_AND_CUSTOMIZATION_PLAN.md`](../../docs/ANIMATION_AND_CUSTOMIZATION_PLAN.md)

## Current scope

Incremental implementation focused on **block/tile sprites** (including
destructible block states), manifest output, and validation. The framework uses
Node builtins only (no external npm dependencies required for the current
generator).

## Quick start

```bash
node tools/pixel-forge/cli.mjs build blocks
node tools/pixel-forge/cli.mjs validate blocks
node tools/pixel-forge/cli.mjs inspect block crate
```

Outputs are written under `assets/blocks/generated/`. Runtime game sheets may
also be synced to `assets/blocks/ (runtime tiles) / legacy sprites removed; ` when a command is intended to replace a
legacy generator output.
