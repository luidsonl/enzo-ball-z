# Animation and Character Customization Plan

> Planning document for entity animation states, layered character customization
> (paper-doll equipment/clothing overlays), and skill-driven visual effects.
> This complements `SPEC.md`, `ARCHITECTURE.md`, and `ACTIONS_AND_SKILLS.md`.
>
> Status: **planned** (runtime sprite loading has been decoupled from the wasm
> binary; layered customization and the formal animation-state pipeline are not
> implemented yet).

## 1. Goals

- Drive character visuals from **entity gameplay states** (idle, walk/run, jump,
  fall, dash, hurt, death, melee/cast, SixSeven farming, etc.).
- Support **character customization** with layered sprites (equipment/clothing),
  so equipped items can change appearance without exploding pre-baked variants.
- Let **skills** produce their own visuals (character action pose/cast, local
  VFX/particles, and gameplay entities such as projectiles) while keeping core
  rules authoritative.
- Keep image assets **outside the wasm binary** (runtime loading) so they can be
  reused by the web UI/tooling and so the wasm stays smaller as assets grow.
- Use a **reproducible, original** asset pipeline (see tooling section).

## 2. Architecture boundaries

| Concern | Owner | Notes |
| --- | --- | --- |
| Gameplay states, equipment data, damage/cooldown/cost | `enzo-core` | Authoritative. No PNG paths, no frames, no draw order. |
| Visual state mapping (`state -> clip/frame`) | `enzo-wasm` (render/animation) | Deterministic from snapshot/state/time; presentation only. |
| Layer composition and drawing | `enzo-wasm` render | Base + equipment overlays + weapon/fx layers. |
| Skill VFX/particles/projectile visuals | `enzo-wasm` `render/skills/*` + effects | Gameplay projectile/collision entities remain core-owned. |
| Asset files and manifests | `assets/`, build scripts | Runtime-loaded from `dist/assets/`; reusable outside wasm. |
| DOM/Vue UI | `web/src/*` | May reuse asset URLs; never computes game rules. |

## 3. Entity animation states

### 3.1 Logical visual state

Expose or derive a logical visual state for render (from `WorldSnapshot` /
player/entity snapshots and relevant timers/flags), not raw texture frames:

- Locomotion/pose: `Idle | Walk | Run | JumpRise | JumpFall | Dash | Fly` (as applicable)
- Action/combat: `None | MeleeStep(n) | Cast(skill_id) | Hurt | Dead`
- Context: `facing`, `on_ground`, velocity, SixSeven/farming visual flag, active
  skill/effect flags when relevant

The core remains the source of gameplay truth. The client resolves
`logical state -> animation clip -> frame`.

### 3.2 Animation driver (client)

Add a small client-side animation module (e.g. `render/animation.rs` or
`render/entities/animation.rs`):

- `AnimClipId` enum (Idle, Run, JumpRise, JumpFall, Dash, Hurt, Melee1..3,
  Cast, Dead, SixSeven, ...)
- Priority mapping: `Dead > Hurt > Dash/Cast/Attack > Airborne > Run/Walk > Idle`
  (adjust as needed, but keep deterministic/explicit)
- Frame driver: time-based (`time / frame_duration % len`) or tick/event-aware;
  support one-shot clips (attack/hurt) with fallback
- Initial compatibility: keep current `ACTOR_CELL = 32` and frame-index style
  (`sprites::enzo::*`) where practical; evolve to clip metadata/manifest later

### 3.3 Tests

- Unit-test state-to-clip/frame mapping in pure client logic where possible.
- Keep gameplay behavior tests in `enzo-core`; render/animation tests must not
  encode game rules.

## 4. Layered character customization (paper doll)

### 4.1 Composition model

Render each animated frame by drawing aligned layers in a documented order
(same destination rect, same cell/frame mapping, same `flip_x`, same pivot):

1. Back/cape (optional)
2. Base body/clothing
3. Legs/feet (if modularized)
4. Armor/torso
5. Head/hair/helmet
6. Weapon/hands/front (animation-dependent)
7. Front local FX (optional; skill gameplay VFX usually separate)

Start simple: **base + equipment overlays per visual slot** (e.g. weapon,
armor, head/accessory). Move to fully modular body parts (head/hands/legs/feet)
only when art and requirements justify it.

### 4.2 Visual equipment contract

- **Core:** exposes equipped item/visual identifiers per equipment slot in
  snapshot/player data (stable IDs such as `ItemKind`/`visual_id`). No image
  paths.
- **Client:** resolves `visual_id + slot + clip/frame -> texture`. Use a static
  registry or generated manifest.
- **Fallback:** missing equipment visual -> draw base only (and log/debug
  placeholder if useful).

### 4.3 Asset alignment rules

All character overlay sheets for a shared rig must use:
- same cell size/grid (currently actor cells are 32x32),
- same frame count and order per clip (or explicit remap metadata),
- same pivot/origin conventions.

This is the main authoring constraint; tooling/validation should enforce it.

### 4.4 Suggested asset layout

```text
assets/sprites/characters/enzo/
  base.png
  manifest.json                # optional, generated/versioned clip metadata
assets/sprites/equipment/
  weapon/<visual_id>.png
  armor/<visual_id>.png
  head/<visual_id>.png
assets/sprites/skills/<skill_id>/
  icon.png
  projectile.png               # optional
  cast_fx.png                  # optional
  impact.png / particles.png   # optional
```

Paths are runtime-loaded from `dist/assets/...` (served statically). Keep names
stable and original.

### 4.5 Performance

- Cache textures by id/path in the client (`HashMap`/equivalent client-side
  cache, not in `enzo-core`).
- Preload common/base assets; lazy-load uncommon equipment visuals when needed.
- A few extra draws per character are acceptable initially; consider atlases if
  asset count grows.

## 5. Skills: character animation + VFX + gameplay entities

Combine three visual tracks:

1. **Character action animation** — driven by skill/cast/attack state or skill
   activation events (e.g. `Melee1-3`, `Cast`). May influence weapon/layer pose.
2. **Local VFX/particles** — client-only presentation (auras, slashes, sparks,
   dust), modularized per skill under `render/skills/*`/effects as appropriate.
3. **Gameplay entities** — projectiles/areas/summons that affect rules remain
   **core entities** in snapshots/events. The renderer draws them; it does not
   decide collision or damage.

This keeps the authoritative simulation intact while allowing rich skill visuals.

## 6. Incremental implementation phases

### Phase A — Animation foundation
- [ ] Define logical visual state / `AnimClipId` and deterministic mapping from
      current snapshots.
- [ ] Extract Enzo frame selection into an animation/render helper.
- [ ] Document clip priorities and one-shot behavior.
- [ ] Add focused client-side tests for mapping/frame selection where practical.

### Phase B — Asset pipeline and metadata
- [ ] Standardize spritesheet/clip layout (frame order/count, duration, loop or
      one-shot).
- [ ] Add optional generated manifest for clips/layers.
- [ ] Evolve `sprites.rs` into a generic texture/atlas cache/loader as needed.
- [ ] Define stable visual-id/path conventions.

### Phase C — Layered customization MVP
- [ ] Pick 1–2 visual equipment slots (e.g. weapon, armor).
- [ ] Ensure snapshot/core exposes the required equipped visual IDs.
- [ ] Implement layer composition in the Enzo renderer.
- [ ] Create aligned example assets.
- [ ] Add safe fallbacks for missing visuals.

### Phase D — Skill visuals
- [ ] Map skill events/states to character clips and VFX.
- [ ] Standardize per-skill assets (`icon`, projectile/cast/impact/particles as
      declared).
- [ ] Keep gameplay projectiles/entities snapshot-driven.

### Phase E — Scale and validation
- [ ] Atlas/manifest optimizations if needed.
- [ ] Build-time validation for rig dimensions/frame counts/alignment basics.
- [ ] Document the asset workflow.

## 7. Pixel art tooling recommendations

### Is code-generated pixel art a good fit?
**Yes, for this project.** The repo already uses generated/versioned assets
(`scripts/gen-sprites.mjs`) and requires original assets. Code is strong for:
- reproducible rigs, palettes, grids, frame counts, and pivots;
- recolors/variants for equipment tiers;
- simple particles/icons/placeholders;
- manifests and validation in CI/build scripts.

Hand-authored pixel art is still valuable for expressive character animation.
A hybrid workflow works well: **code/script for structure and variants + pixel
editor for detailed keyframes/overlays**.

### Is AI a good fit?
**Only as a controlled assistant, not as unchecked final art.** Follow project
rules (`AGENTS.md`): assets must be original and not copied from existing works.
- Use AI, if at all, for references/concepts/palette/pose exploration, then
  redraw/clean manually into the project’s grid/frame rig.
- Do not commit raw AI output as final assets without rights/originality review.
- Do not imitate protected styles/franchises.

### Practical tools

| Need | Recommendation | Notes |
| --- | --- | --- |
| Reproducible game pipeline | **Node scripts** (evolve `scripts/gen-sprites.mjs`) + image libs/Sharp/Canvas as needed | Official, versioned, CI-friendly |
| Manual pixel animation/overlays | **Aseprite** (paid) or **LibreSprite** (open fork) | Spritesheets, tags, onion skin, export control |
| Quick/simple editing | **Piskel** | Lightweight for simple items/fx |
| Composition/automation | **Sharp/ImageMagick** or custom scripts | Combine layers, validate sizes, build atlases/manifests |
| AI assistance | Draft/reference only, then manual redraw | Ensure originality and license clarity |

## 8. Acceptance criteria (implementation targets)

- [ ] Logical visual state maps deterministically to clips/frames.
- [ ] Enzo renders as **base + equipment layers** aligned to the same rig.
- [ ] Changing equipment affects visuals without changing core rules.
- [ ] Skills produce coherent character action + VFX/projectile visuals driven by
      core state/events.
- [ ] Visual assets are runtime-loaded and cacheable; wasm does not embed them.
- [ ] Asset pipeline is reproducible, documented, and produces original assets.

## 9. Character customization rig (proposed contract)

This section defines a concrete, scalable contract for appearance customization:
skin color, hair, facial expressions/variations, clothing colors, and equipment
(hats, shirts/clothes, pants, shoes, weapons/accessories such as a Nutella gun,
fidget spinner, amoeba-themed item, etc.).

### 9.1 Viability

This is **viable** with a paper-doll/layered rig. The main requirement is
discipline: every customizable layer must follow the same animation contract
(grid, clips, frame order/count, pivot, and `flip_x` behavior). Runtime cost is
low (a handful of extra sprite draws per character plus cached textures).
Complexity comes from **asset authoring and validation**, not from rendering.

### 9.2 Visual slots (example)

Define stable visual slots. Slots can be cosmetic, equipment-backed, or both.

| Slot | Examples | Notes |
|---|---|---|
| `skin` | Skin tones | Prefer palette variants/pre-generated sheets; simple tint can break baked pixel shading. |
| `eyes` / `face_base` | Eye shape/base face | Optional split from head if needed. |
| `expression` | Idle, blink, happy, hurt, attack, cast | Driven by visual/game state or events; can be a face overlay or head variant. |
| `hair_back` | Long hair behind body/head | Optional; used when hair must render behind torso/headgear rules. |
| `pants` | Pants, shorts | Clothing layer. |
| `shoes` | Shoes, sneakers, boots | Clothing layer. |
| `torso` | Shirt, jacket, outfit top | Clothing layer. |
| `armor` | Protective gear | May overlap/replace torso presentation rules; define explicitly. |
| `head_base` | Base head if separated | Optional; some rigs keep head in base body. |
| `hair_front` | Hair visible in front | Can be hidden/replaced by hats/helmets depending on rules. |
| `headgear` | Cap, hat, helmet | May hide hair or select a `hair_under_hat` variant. |
| `weapon` | Nutella gun, fidget spinner, melee weapon, amoeba container/wand | Held/equipped visual; may be clip-dependent (idle vs attack/cast). |
| `back` | Cape, backpack | Optional. |
| `front_fx` | Local cosmetic glow | Presentation only; gameplay VFX/projectiles are separate. |

Not every project needs all slots. Start with an MVP subset (e.g. `skin`,
`hair_front`, `expression`, `torso`, `pants`, `shoes`, `headgear`, `weapon`) and
expand once the rig/manifest is stable.

### 9.3 Draw order and composition rules

Use an explicit, documented draw order. Example:

1. `back`
2. `hair_back`
3. `body/skin` (or base body)
4. `pants`
5. `shoes`
6. `torso`
7. `armor`
8. `face_base` / head details
9. `expression`
10. `hair_front` (or hidden by `headgear` rule)
11. `headgear`
12. `weapon`
13. `front_fx`

All layers for the same character frame use the same `dest`, `flip_x`, and frame
index/clip mapping.

**Rule examples (must be data-driven/explicit):**
- `headgear.hides_hair_front = true` → skip `hair_front` or load an alternate
  hair variant.
- `expression = hurt` when hurt state/timer is active; fallback to `idle`.
- `weapon` overlay may change by clip (`Idle`, `Cast_Nutella`, `Melee`), or a
  skill may spawn a separate projectile entity instead of relying only on an
  overlay.

### 9.4 Held items vs gameplay entities (important)

| Visual/gameplay case | Representation |
|---|---|
| Nutella gun equipped in hand | `weapon`/equipment visual layer + cast/attack animation pose. |
| Nutella shot fired | **Gameplay projectile entity** from `enzo-core` snapshot/event; rendered by skill render module with its own sprite/VFX. |
| Fidget spinner equipped/used | Visual layer/animation + optional local VFX; if it has gameplay hitboxes/projectiles, those are core entities/events. |
| Amoeba | Depends on design: cosmetic accessory/layer, held item, summoned companion entity, or projectile. If it affects gameplay, model it in `enzo-core` and render from snapshot/state. |

This separation preserves the authoritative simulation.

### 9.5 Core/client data contract

`enzo-core` must not contain texture paths. It should expose stable visual data,
for example:

```text
VisualProfile / EquipmentVisualState
  skin_tone_id
  hair_style_id
  hair_color_id      (or palette id)
  eye/expression defaults (optional)
  slots:
    headgear -> item_visual_id | null
    torso    -> item_visual_id | null
    pants    -> item_visual_id | null
    shoes    -> item_visual_id | null
    weapon   -> item_visual_id | null
```

The client (`enzo-wasm`) resolves IDs through a registry/manifest:
`slot + visual_id (+ clip/frame context) -> asset path/texture`. Vue/UI can
reuse the same IDs/URLs for previews, but must not compute game rules.

Persist appearance/customization in save data only through explicit, versioned
fields (core/save contract), not through render-only cache.

### 9.6 Colors and variants (pixel art)

For pixel art with baked highlights/shadows, **naive runtime tinting often looks
wrong**. Preferred strategies:

- **Pre-generated variants** via build script (palette swap/recolor from a base
  sheet): reliable and cheap at runtime. Good for skin tones, hair colors, and
  clothing colors.
- **Authored palettes**: hand-tuned palettes per variant for important assets.
- **Tint/multiply only when art is designed for it** (e.g. grayscale base +
  controlled shading layers). Treat as an opt-in technique, not the default.

The asset pipeline should generate/validate variants and record them in the
manifest (e.g. `shirt_blue.png`, `shirt_red.png`, or `shirt.png?palette=blue`
resolved to a generated file).

### 9.7 Rig manifest (proposed shape)

Use a generated/versioned manifest to avoid scattered hardcoded paths.

```json
{
  "rig": "enzo_32x32",
  "cell": { "w": 32, "h": 32 },
  "clips": {
    "idle": { "frames": 2, "frameDurationMs": 240, "loop": true },
    "run":  { "frames": 4, "frameDurationMs": 90,  "loop": true },
    "hurt": { "frames": 1, "frameDurationMs": 120, "loop": false }
  },
  "slots": {
    "torso": { "order": 60, "requiredForBase": false },
    "headgear": { "order": 110, "hides": ["hair_front"] },
    "weapon": { "order": 120 }
  },
  "visuals": {
    "weapon.nutella_gun": "assets/sprites/equipment/weapon/nutella_gun.png",
    "headgear.cap_red": "assets/sprites/equipment/head/cap_red.png"
  }
}
```

The exact schema can evolve, but it should be **generated/validated by scripts**
and consumed by the client loader/renderer.

### 9.8 Sprite generation/build pipeline (proposed)

Evolve the existing Node pipeline (`scripts/gen-sprites.mjs`) into a small
pixel-asset build workflow (same repo, reproducible):

```text
scripts/pixel-build/
  rig/                # clip/frame/slot definitions
  palettes/           # skin/hair/clothing palettes
  generate.mjs        # recolor, compose, sheet assembly, placeholders
  validate.mjs        # dimensions, frame counts, transparency, manifest checks
  manifest.mjs        # emits manifest.json
```

Useful commands (target):
- `npm run sprites` / `npm run sprites:build` — generate/update assets
- `npm run sprites:validate` — fail on rig/contract mismatches
- CI check: generated outputs or validation passes (choose a reproducibility
  policy)

### 9.9 Libraries/tools: what helps

There is no single off-the-shelf library that perfectly matches this game's
Rust/WASM + macroquad + custom rig needs. Use focused tools:

| Purpose | Practical choice | Notes |
|---|---|---|
| Runtime composition | Custom renderer in `enzo-wasm` | Simple, deterministic, full control. |
| Texture loading/cache | Current runtime loading + client cache | Already aligned with decoupled assets. |
| Pixel authoring | **Aseprite** or **LibreSprite** | Best for aligned frame-by-frame overlays/expressions. |
| Node image pipeline | **sharp** (fast) or `pngjs/jimp`/Node canvas | Recolor, compose, crop/sheet, validate. |
| Rust build tooling (optional) | `image` crate in scripts/tools crate | Useful if you want validation/generation in Rust outside wasm. |
| Modular character reference | LPC/Universal LPC *concepts* only | Do not copy external asset packs/styles; keep assets original and build your own rig. |
| AI assistance | Draft/reference only, then manual redraw | Must follow `AGENTS.md` originality/licensing/style rules. |

### 9.10 Recommendation

Build a **custom Enzo rig contract + manifest + Node build/validation scripts**,
and render layers in `enzo-wasm` from stable core visual IDs. Use Aseprite/
LibreSprite for expressive frames and code for variants/validation. This gives
you scalability (many clothes/colors/weapons/expressions) without coupling
assets to the wasm or duplicating gameplay logic in the client.

## 10. Modular blocks and map symbols (planned)

Blocks/tiles should become **modular definitions** so new terrain and special
elements can be added without hardcoding behavior in the map parser, physics,
and renderer.

### 10.1 Viability

Yes. The current map uses a simple `Tile` enum and fixed ASCII symbols in
`enzo-core/src/map.rs` (`#`, `B`, `=`, `.`, `P`, `E`, `F`). This can evolve into
a registry-driven model: a symbol resolves to a `BlockId`/`BlockDef`, and core
systems query block properties instead of matching ad hoc tile variants.

### 10.2 Block definition contract (core-facing)

Each block type should declare stable, deterministic properties:

| Property | Example | Notes |
|---|---|---|
| `id` | `ground`, `water`, `ladder`, `flag`, `breakable_crate` | Stable identifier; append-only evolution when persisted/referenced. |
| `symbol` | `#`, `~`, `H`, `F` | ASCII map symbol (or token) used in level source. Symbols must be unambiguous in a given parser version. |
| `size` | `1x1` tile (default) | Default is one grid cell (`TILE_SIZE`). Special/multi-cell blocks need explicit placement/anchor rules. |
| `collision/traversability` | `solid`, `platform`, `passable`, `fluid`, `climbable`, `trigger` | Gameplay behavior; queried by physics/movement. |
| `resistance/durability` | `None`, `HP=3`, `unbreakable` | Only when applicable (breakable blocks). Damage/destruction rules live in core. |
| `special behavior` | `water -> swim`, `ladder -> climb`, `flag -> goal trigger` | Implemented as explicit core behavior/data, not render logic. |
| `render/sprite id` | `tile.ground_top`, `block.water` | Visual key resolved by client; core may expose id/kind for rendering. |
| `spawn/marker kind` | player/enemy spawn, flag marker | Some symbols are markers that place entities/triggers and leave an underlying tile; keep semantics explicit. |

### 10.3 Special block examples

- **Flag**: goal trigger/marker. Can remain a level marker that creates a goal
  region/entity, or become a special block definition with trigger behavior. The
  victory condition remains a core rule.
- **Ladder**: `climbable` block. Entities capable of climbing (based on state/
  capability) can move vertically while overlapping it; physics resolves this as
  explicit behavior, not a sprite trick.
- **Water**: `fluid` block. Allows swimming-capable entities to swim (reduced
  gravity/altered movement, oxygen/breath rules later if added). Entry/exit,
  buoyancy, and movement modifiers must be deterministic core behavior.
- **Breakable block**: has `resistance`; when reduced to zero, core emits events
  and replaces/removes the block deterministically.

### 10.4 Map format evolution

- Keep ASCII maps human-editable, but move symbol handling to a **block
  registry**.
- Reserve a strategy for symbol exhaustion (multi-char tokens, legend header, or
  layered map format) before adding many block types.
- Map parsing must remain pure/deterministic and produce clear errors for unknown
  symbols, invalid legend entries, or incompatible map versions.
- Special markers (spawn, enemy, flag) should have explicit precedence rules
  (e.g. marker consumes cell and records spawn/trigger; underlying fill tile is
  configurable or defaults to empty).

### 10.5 Client/render responsibilities

- The client maps `render/sprite id` (or block kind) to textures/animations.
- Special visual states (animated water, waving flag, damaged crate) are
  presentation driven by snapshot/block state; they must not affect gameplay.
- Block assets should be runtime-loaded and reusable outside wasm, following the
  same asset pipeline as characters/skills.

## 11. Animation rig detail: hands and SixSeven (67) aura farming

The **SixSeven** mechanic (alternating `6`/`7` to farm aura) is a gameplay
mechanic in core, but its animation is important for readability and character
personality.

- Add explicit arm/hand layering or hand anchors in the character rig so the
  SixSeven pose can be animated clearly (including when holding weapons/items).
- Define visual states/clips such as `SixSevenA`, `SixSevenB`, or a `SixSeven`
  clip driven by the farming/pose state exposed for rendering.
- **Weapon interaction rules must be explicit:** e.g. holster weapon, keep weapon
  in one hand, overlay gesture VFX, or disable SixSeven animation while specific
  items are equipped. Gameplay availability remains a core rule; visuals follow
  the contract.
- Facial expression/upper-body overlays (if used) must align with the same rig
  and not conflict with hair/headgear/weapon layers.

## 12. Unified development framework (assets, entities, animations, abilities)

To scale content, create a project-local framework (tooling + conventions), not
ad hoc scripts per asset.

### 12.1 Goals

- Single source of truth for **rigs, clips, layers, block defs, skill visuals,
  and manifests**.
- Reproducible generation/validation of pixel assets.
- Fast iteration: create a new block/entity animation/skill visual by adding a
  definition + assets that pass the contract.
- Reuse assets outside wasm (Vue UI previews, tooling, future clients).

### 12.2 Proposed structure

```text
tools/pixel-forge/                 # project-local framework/lib (name can change)
  README.md
  package.json                     # or workspace package
  src/
    core/                          # schemas, ids, validation primitives
    rigs/                          # character/prop rigs
    blocks/                        # block definitions, symbol registry, render ids
    animations/                    # clips, frame maps, priorities
    entities/                      # entity visual definitions (not gameplay rules)
    skills/                        # skill visual manifests (icon/cast/projectile/fx)
    generators/                    # recolor, compose, spritesheet, atlas, manifest
    validators/                    # dimensions, frame counts, pivots, symbol conflicts
  cli.mjs                          # `pixel-forge build|validate|inspect`
assets/
  sprites/...
  definitions/                     # JSON/JS definitions consumed by generators/validators
scripts/
  gen-sprites.mjs                  # can become a thin wrapper over pixel-forge
```

### 12.3 Framework responsibilities

- **Schemas/contracts**: validate rig manifests, block defs, symbol maps, skill
  visual defs, and entity visual defs.
- **Generators**: palette recolor, layer composition, spritesheet assembly,
  placeholder generation, manifest output.
- **Validators**: fail on missing frames, wrong dimensions, unknown IDs, symbol
  collisions, misaligned overlay rigs, or missing required skill icons/assets.
- **Inspectors**: debug a composed frame/layer stack and print resolved asset
  paths.
- **No gameplay logic**: the framework generates/validates presentation data and
  assets; `enzo-core` remains authoritative for blocks/entities/skills behavior.

### 12.4 Libraries to use in the framework

Use focused libraries; avoid a heavy game-engine-specific art dependency:

- **Node + `sharp`** (preferred for performance) or `pngjs/jimp`/Node canvas for
  image generation/composition/recolor/validation.
- **Aseprite/LibreSprite** for manual frame authoring/export; framework scripts
  can normalize/validate exported sheets.
- **JSON Schema/Zod-like validation** (or lightweight custom validators) for
  manifests/definitions.
- Optional **Rust `image`** tooling only if a separate build tool crate is
  justified; keep the main pipeline simple and reproducible.

### 12.5 How sprites, animations, contracts, and layers will be created

Creation is **contract-first** and uses a **hybrid workflow**:

1. **Contract/definition** — define a rig (cell, pivot, clips, slots), visual
   definitions (id, rig, slot, clips covered, fallback, source/palette), block
   definitions (id, symbol, properties, render id), and skill visual manifests.
2. **Authoring** — either:
   - **Procedural (code)**: deterministic generators (evolved from
     `scripts/gen-sprites.mjs`) draw primitives/pixels, build spritesheets, and
     apply **palette swaps** for variants (skin/hair/clothing colors). Best for
     placeholders, tiles, icons, FX, and scalable color variants.
   - **Manual (Aseprite/LibreSprite)**: frame-by-frame art exported as PNG
     spritesheets aligned to the rig. Best for expressive character animation,
     expressions, hair, weapons, and SixSeven hand poses.
3. **Normalize/compose** — `pixel-forge` crops/orders frames to the rig, applies
   palette variants, and can compose layers for `inspect`/preview.
4. **Validate** — fail on wrong dimensions, missing clip frames, unknown
   IDs/symbol collisions, missing required assets (e.g. skill icon), invalid
   slot/`hiddenBy` rules, or incomplete coverage without fallback.
5. **Output** — deterministic spritesheets + `manifest.json` in
   `assets/sprites/generated/` (runtime-loaded into `dist/assets/`). `enzo-wasm`
   composes `base + layers` per frame at runtime; gameplay projectiles/special
   block behaviors remain core-driven.

### 12.6 Execution order (agreed)

1. **Modularize blocks first (current work):** introduce a core block
   definition/registry, symbol mapping, and property queries
   (size/collision-traversability/resistance/special hooks, render id). Preserve
   existing gameplay behavior and tests; add extension points for water,
   ladder, flag/trigger, and breakable blocks.
2. **Scaffold `tools/pixel-forge`:** CLI, schema/validation, manifest output,
   and a wrapper over `scripts/gen-sprites.mjs` without changing current sprite
   output.
3. **Create block sprites with `pixel-forge`:** define block visual contracts and
   generate/validate tile sprites (including placeholders for special blocks)
   through the framework.
4. **Expand to rigs/characters/skills:** Enzo rig, layered customization,
   expressions/hands/SixSeven, equipment visuals, and skill visual manifests.

### 12.7 Initial implementation recommendation

1. Modularize blocks in `enzo-core`/map+physics/render contracts (preserve
   behavior; add tests for registry/symbol/properties).
2. Scaffold `tools/pixel-forge/` with schema/validation stubs and a CLI wrapper.
3. Migrate existing `scripts/gen-sprites.mjs` behavior behind the framework.
4. Add block visual definitions and generate/validate block sprites with
   `pixel-forge`.
5. Add rig/clip/overlay validation before generating large customization sets.
6. Integrate `npm run sprites:build` and `npm run sprites:validate` into CI when
   stable.
