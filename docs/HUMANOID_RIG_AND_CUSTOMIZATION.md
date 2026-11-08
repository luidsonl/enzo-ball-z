# Humanoid Rig, Animation States, and Customization Layers

> Design and implementation contract for humanoid characters (Enzo, Valentina,
> future humanoids). This document adjusts the action/skills evolution plan to a
> **state-driven rig + layered overlay** model.
>
> **Principles:** `enzo-core` is authoritative and pure (logical states, stable
> IDs, contracts only; no image paths/IO/render). Assets/manifests are generated
> and validated by `enzo-export` + `pixel-forge`. Rendering (WASM/DOM/canvas)
> composes layers from snapshot + manifests and must never decide gameplay rules.

## 1. Goals

- Define a **humanoid animation rig**: a list of animation clips for each entity
  state (idle, walk/run, crouch, jump/fall, fly, swim, farm aura/SixSeven,
  attack/melee/kick/cast, hurt, dying/dead, etc.).
- Every clip uses the **same frame grid/alignment contract** for a given rig so
  customization layers can overlay the base animation perfectly.
- Support **layered customization/equipment**: skin color, face, hair style/color,
  outfit top/shirt, outfit bottom/pants, shoes/sneakers, headgear/cap, weapon/sword,
  accessories/items. Layers overlay the rig frames for the current state.
- Make the model reusable by all humanoid entities (Enzo, Valentina, and future
  humanoid characters/enemies where applicable), with character-specific rigs or
  shared rigs selected by stable IDs.
- Keep gameplay authoritative in core: core resolves logical `AnimationState` and
  exposes character/equipment/customization IDs; clients resolve visuals.

## 2. Logical animation states (core)

Core resolves deterministic gameplay state into `AnimationState` (`pose` +
`action` + facing/context). Renderers/assets map these to clip/frame lists.

### 2.1 Locomotion poses

| Pose | Meaning / typical states |
| --- | --- |
| `idle` | Standing still on ground |
| `walk` | Moving on ground (non-running) |
| `run` | Moving on ground at run speed (when applicable) |
| `crouch` | Crouching/ducking (`abaixar`), when supported by gameplay |
| `jump_rise` | Airborne, ascending |
| `jump_fall` | Airborne, descending/falling |
| `dash` | Dash locomotion override |
| `fly` | Flying/hover movement |
| `swim` | Swimming/water movement (gameplay may be added later) |

### 2.2 Action poses (overlay/priority)

| Action | Meaning |
| --- | --- |
| `none` | No priority action |
| `sixseven(hand)` | Aura farming pose/hand alternation (`6/7`) |
| `melee(step)` | Melee/punch combo step |
| `kick` | Kick attack (character/skill-specific) |
| `cast` | Ranged/magic/aura cast |
| `hurt` | Damage reaction / invulnerability flash context |
| `dying` | Death transition (optional; may map to `dead` clip) |
| `dead` | Dead/knocked-out pose |

Action priority follows gameplay semantics (e.g. dead/hurt/attacks/farming over
locomotion) and is defined in core's resolver.

### 2.3 Clip IDs (asset/rig contract)

Each rig publishes named clips. Clip IDs are stable `snake_case`. A state maps to
one clip (with fallbacks):

- `idle`, `walk`, `run`, `crouch`
- `jump` or `jump_rise`/`jump_fall` (rig may provide combined or split)
- `fly`, `swim`
- `dash`
- `sixseven_a`, `sixseven_b` (or `sixseven` with frame rules)
- `attack_1`, `attack_2`, `attack_3` (melee steps), `kick`, `cast`
- `hurt`, `dying`, `dead`

Fallback rules are explicit in the rig manifest (e.g. `run -> walk -> idle`,
`crouch -> idle`, `swim -> jump/fly/walk` depending on rig, `dying -> dead`).

## 3. Humanoid rig contract

A rig defines the frame geometry and clip catalog shared by all layers for that
humanoid model.

```json
{
  "id": "humanoid_enzo_v1",
  "cell": { "w": 32, "h": 32 },
  "anchor": { "x": 16, "y": 30 },
  "clips": {
    "idle": { "frames": ["idle_0", "idle_1"], "fps": 2.5, "loop": true },
    "walk": { "frames": ["walk_0", "walk_1", "walk_2", "walk_3"], "fps": 8, "loop": true },
    "crouch": { "frames": ["crouch_0"], "fps": 1, "loop": true },
    "jump_rise": { "frames": ["jump_rise_0"], "fps": 1, "loop": true },
    "jump_fall": { "frames": ["jump_fall_0"], "fps": 1, "loop": true },
    "sixseven_a": { "frames": ["sixseven_a_0"], "fps": 1, "loop": true },
    "sixseven_b": { "frames": ["sixseven_b_0"], "fps": 1, "loop": true },
    "attack_1": { "frames": ["attack_1_0"], "fps": 10, "loop": false },
    "hurt": { "frames": ["hurt_0"], "fps": 1, "loop": true },
    "dead": { "frames": ["dead_0"], "fps": 1, "loop": true }
  },
  "fallbacks": { "run": ["walk", "idle"], "dying": ["dead"], "swim": ["walk"] }
}
```

- `cell` is the sprite cell size.
- `anchor` is the foot/pivot point inside the cell for consistent composition.
- Every layer sheet for this rig provides the same frame IDs/rects (or is mapped
  through the same atlas layout). Missing layer frames fall back to transparent.

## 4. Layer model (overlay order)

Layers are composed back-to-front in deterministic order. Higher layers draw on
top. Exact order may be rig-specific, but the default humanoid stack is:

1. **shadow/effects context** (renderer concern)
2. **base body / skin** (customization: skin tone/physical base)
3. **underwear/base clothing** (optional)
4. **face** (customization)
5. **hair back** (optional, for styles with back hair)
6. **outfit bottom / pants** (customization or equipment override)
7. **shoes / sneakers** (customization or equipment override)
8. **outfit top / shirt** (customization or equipment override)
9. **hair front** (customization)
10. **headgear / cap** (equipment)
11. **accessory** (equipment/cosmetic)
12. **weapon / sword / held item** (equipment; may need hand-layer variants)
13. **front effects** (attack auras/projectile spawn cues, renderer)

### 4.1 Customization layers (stable IDs, no paths in core)

`VisualProfile` (persisted in save/snapshot) carries stable IDs interpreted by
client/asset manifests:

- `skin_tone` — base skin/body palette/variant
- `hair_style` — hair shape/style
- `hair_color` — hair palette/variant
- `face` — face/eyes/expression variant
- `outfit_top` / `shirt` — default top clothing
- `outfit_bottom` / `pants` — default bottom clothing
- `shoes` — default footwear
- optional future: `body_type`, markings, accessories (all with `serde(default)`)

### 4.2 Equipment overlays

Equipped items can replace or overlay customization layers for their visual slot:

| Equip slot / item category | Typical layer effect |
| --- | --- |
| Weapon / Sword | held weapon layer (may vary by state: idle vs attack/cast) |
| Head / Cap / Headgear | head overlay (can hide or adapt hair) |
| Armor / Shirt | top/body overlay |
| Feet / Sneakers | footwear overlay |
| Accessory | cosmetic/aura overlay |

Item definitions declare visual metadata (`visual_slot`, layer behavior, rig
compatibility tags) in data/contracts. Runtime composition is client-side using
authoritative equipped item IDs from snapshot.

## 5. Asset layout

```text
assets/
  definitions/generated/
    humanoid.json          # rig/clip/layer contracts (exported)
    items.json             # includes visual layer metadata
    entities.json          # character -> rig/profile defaults
  entities/
    enzo/
      rig.json             # rig/atlas/clip manifest
      sheet.png            # legacy/base fallback (transition)
      layers/              # rig-aligned layer sheets (generated/hand-tuned)
        base.png
        skin_*.png
        hair_*/*.png
        face_*.png
        outfit_top_*.png
        outfit_bottom_*.png
        shoes_*.png
    valentina/
      rig.json
      sheet.png
      layers/
  items/
    sword_67/
      icon.png
      world.png
      equipped.png         # legacy overlay
      layers/              # rig-aligned overlays (weapon/slot-specific)
        weapon.png
```

All generated assets and manifests are produced/validated by `pixel-forge` and
must align to the rig's clip/frame contract.

## 6. Core, export, and client responsibilities

- **`enzo-core`**: resolves `AnimationState`; stores/persists `PlayableCharacter`,
  `VisualProfile`, loadout/equipment; validates equip/loadout requirements. No
  paths, no textures, no rendering.
- **`enzo-export`**: exports authoritative contracts (`humanoid`, `items`,
  `entities`, customization IDs/options) to `assets/definitions/generated/*.json`.
- **`pixel-forge`**: generates/validates rig manifests, base sheets, and
  rig-aligned layer assets/placeholders; enforces required files and clip/frame
  coverage.
- **`enzo-wasm` renderer**: selects rig by character, resolves clip+frame from
  `AnimationState` + rig manifest, draws layers in order using `VisualProfile`
  and equipped items, with fallbacks to legacy sheets.
- **DOM/Vue UI**: edits customization/loadout through authoritative actions and
  renders previews from state/contracts; no gameplay rules.

## 7. Implementation phases

1. **Contracts/docs**: this doc; add `crouch/dying`-ready logical states as
   compatible extensions; export humanoid/entity contracts.
2. **Rig manifests**: generate `rig.json` + expanded clip/frame lists for Enzo and
   Valentina (including idle/walk/run/crouch-ready/jump/fall/fly/swim
   fallbacks/attack/cast/kick/hurt/dead/sixseven).
3. **Layer asset pipeline**: generate rig-aligned layer sheets for
   customization/equipment; replace single `equipped.png` overlays with
   rig-aligned layer composition where available.
4. **Renderer composition**: manifest-driven paper-doll compositor with
   fallbacks to current rendering.
5. **Customization API/UI**: authoritative profile-edit actions, save/snapshot
   wiring, DOM HUD customization controls, previews.
6. **Validation**: core tests, `pixel-forge validate all`, `cargo fmt/clippy/test`.

## 8. Compatibility and safety

- Add new enum variants/fields with `serde(default)` / backward-compatible
  defaults. Snapshots are not persisted across incompatible consumers; save
  migration must remain safe.
- Missing assets/manifests must degrade gracefully (fallback to legacy sheet or
  placeholder), never panic.
- Keep visual IDs stable and append-only where persisted.
- Do not move gameplay rules into tooling or rendering.
