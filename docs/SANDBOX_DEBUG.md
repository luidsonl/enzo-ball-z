# EnzoBallZ — Sandbox / Debug Mode Specification

> Status: **planned/specification** (not implemented yet).
>
> This document specifies the secret sandbox/debug area used for QA and
> development iteration. It defines unlock behavior, sandbox session rules, the
> debug UI, world-editing tools, and progression/debug mechanics.
>
> Related docs:
> - [`SPEC.md`](SPEC.md) — game rules and player-facing behavior.
> - [`ARCHITECTURE.md`](ARCHITECTURE.md) — core/client boundaries and contracts.
> - [`ACTIONS_AND_SKILLS.md`](ACTIONS_AND_SKILLS.md) — action/input/loadout model.

## 1. Purpose and scope

The sandbox/debug mode is a QA-focused environment for quickly testing combat,
progression, skills, entities, items, and map interactions without normal
campaign constraints.

### 1.1 Goals

- Provide a hidden entry point from the main menu unlocked by entering `67`.
- Load a dedicated, unique flat map optimized for testing.
- Expose a debug panel with 4 tabs: **Character**, **Blocks**, **Enemies**, and
  **Items**.
- Allow safe, fast iteration: free level/attribute changes, invulnerability
  (sigma mode), controlled death/reset behavior, and runtime placement/removal
  of blocks, enemies, and items.
- Keep core authoritative: all gameplay-affecting debug mutations are applied by
  `enzo-core` APIs/commands; UI and bridge only transport intents and render
  state.

### 1.2 Non-goals (initial scope)

- Shipping sandbox/debug tools in normal campaign gameplay UX by default.
- Replacing normal save/progression systems.
- Networked/multiplayer debug synchronization (future work follows the same core
  contracts).
- Arbitrary scripting, console commands, or unrestricted file/map import/export
  (unless explicitly added later).

## 2. Secret unlock and entry

### 2.1 Unlock sequence (`67`)

- **Secret code:** the digit/key sequence `6`, then `7` (same thematic inputs as
  the SixSeven mechanic).
- **Context:** sequence detection is active on the **main menu** (and may be
  ignored while text/input capture modals are active, e.g. keybinding capture).
- **Timing:** the second input must occur within a short window after the first
  (recommended: `1.0 s`); timeout or invalid intermediate input resets the
  sequence.
- **Keyboard/desktop:** pressing the `6` and `7` keys in order satisfies the
  sequence.
- **Touch/mobile and mouse (DOM UI):** the menu must provide an accessible way
  to enter the same sequence without relying on a hardware keyboard (e.g. hidden
  tappable targets, menu easter-egg controls, or DOM-emitted sequence actions
  mapped to the same semantic detection). The exact visual affordance may remain
  subtle, but touch users must be able to unlock it.

### 2.2 Unlock state and persistence

- **Default:** sandbox entry is hidden until unlocked.
- **Persistence (recommended):** store an `unlocked` preference locally (client
  storage, separate from campaign `SaveData`) so QA does not need to re-enter
  the code every session.
- **Reset:** clearing local app data/preferences resets the unlock state.
- **Feedback:** when unlocked, the UI should provide clear confirmation (toast,
  label, or newly visible menu item).

### 2.3 Menu entry

- After unlock, the main menu shows a **Sandbox / Debug** entry (or equivalent
  label in i18n).
- Selecting it starts a sandbox session in the dedicated sandbox map.
- Sandbox can be opened from the menu without selecting a normal campaign save
  slot by default. If a slot/profile context is used for testing, it must be
  explicit and isolated (see §3.3).

## 3. Sandbox session model

### 3.1 Dedicated flat map

- Sandbox uses a **unique, flat map** (dedicated level asset/config), distinct
  from campaign levels.
- **Map requirements:**
  - Continuous flat ground with generous horizontal space for movement, combat,
    and placement.
  - Safe player spawn with room for spawned enemies/items/blocks.
  - No mandatory progression blockers (flag/victory trigger optional; if present
    it must not interfere with QA workflows by default).
  - Simple, stable background metadata.
  - Deterministic, versioned map definition consistent with existing level/map
    format.

### 3.2 Session lifecycle

- Entering sandbox creates/loads a sandbox world session.
- Exiting sandbox returns to the menu (or previous safe UI state) and tears down
  sandbox-only runtime state.
- **Reset sandbox:** restores the flat map, removes spawned/runtime sandbox edits
  (as configured), and resets the sandbox player/debug state to sandbox defaults.
- A visible indicator should show when sandbox mode and/or sigma mode is active.

### 3.3 Save/profile isolation (safety)

- **Default policy:** sandbox sessions are isolated from normal campaign save
  slots (`SaveData` in slots 1–3). Sandbox debug mutations must not auto-save to
  campaign slots.
- **Progression context:** sandbox uses a dedicated debug profile/session state
  (or reserved debug-only profile) for level/attributes/loadout/customization
  testing.
- **Explicit persistence only:** if an implementation adds "save sandbox state"
  or "apply to slot/profile" actions, they must be explicit user actions and
  clearly labeled to avoid accidental campaign corruption.
- Existing campaign save format/versioning rules remain unchanged unless a
  documented, backward-compatible migration is introduced for debug-only fields.

## 4. Debug panel UI (4 tabs)

The sandbox exposes a debug panel with four tabs. The panel is available only in
sandbox sessions (and/or only when sandbox tools are enabled).

### 4.1 Common UI behavior

- Tabs: **Character**, **Blocks**, **Enemies**, **Items**.
- UI follows existing DOM/HUD interaction patterns where practical (focus,
  draggable/modal behavior as applicable, i18n labels, mobile-friendly controls).
- The panel emits semantic debug actions through the UI bridge; it does not
  mutate core state directly.
- Tool selection is explicit and visible (active tab/tool and selected
  block/enemy/item).

### 4.2 Character tab

Provides controls for progression and survival testing:

- **Level up (free):** increment level and/or grant progression resources for
  testing (e.g. grant points/XP/levels) without normal combat/farming.
- **Level down / adjust level:** optional but recommended for QA; must obey
  minimum level and redistribution rules.
- **Sigma mode toggle:** enables/disables invulnerability (see §5.2).
- **Die action:** triggers the sandbox death/debug rule (see §5.3).
- **Reset actions:** at minimum reset sandbox character/progression state;
  optionally separate "reset attributes/redistribute" action.
- **Status display:** current level, XP, points, attributes, and sigma mode
  state.

### 4.3 Blocks tab

Provides runtime map/block editing tools:

- Palette/list of placeable block types (from core/block definitions).
- **Place block** tool and **remove block** (erase/empty) tool.
- Selection of active block type and placement preview/cursor feedback.
- Operations apply at a targeted tile/world position (pointer/tap/click).

### 4.4 Enemies tab

Provides enemy spawn tools:

- Catalog/list of spawnable enemy types/species (initially Slime; extensible).
- **Place/spawn enemy** action at targeted position.
- Optional QA helpers (not required initially): remove selected/recent enemy,
  clear all sandbox-spawned enemies.

### 4.5 Items tab

Provides item spawn tools:

- Catalog/list of spawnable item kinds (weapons/equipment/consumables/passives
  as supported by item definitions).
- **Place/spawn item** action at targeted position.
- Optional QA helpers (not required initially): remove item entity, clear items.

## 5. Debug mechanics and rules

### 5.1 Free leveling and progression edits

- Sandbox may grant levels/points/XP directly for QA.
- Progression edits must keep state consistent:
  - `level >= 1`.
  - `xp` within valid bounds for the resulting level (or normalized by rule).
  - `points` reflects unspent points after edits/redistribution.
  - attributes and derived stats (`Stats`) are recomputed consistently.
  - health/aura are clamped to valid maxima; behavior on max changes must be
    deterministic and documented (e.g. heal to max on level gain, clamp on
    reductions).
- Loadout validity must be preserved (sanitize/revalidate against requirements
  when progression/attributes/equipment change).

### 5.2 Sigma mode (invulnerability)

- **Sigma mode** is a sandbox/debug toggle that makes the player invulnerable.
- While enabled, the player must not die or lose health from normal gameplay
  damage sources in sandbox, including at minimum:
  - enemy contact damage,
  - projectile/skill damage received from hostile sources (as applicable),
  - falling below the map / fall-death handling.
- Sigma mode must be enforced by core simulation rules, not by UI-only checks.
- Visual/HUD feedback should clearly indicate when sigma mode is active.
- Default state on sandbox start: **off** (unless a future explicit preset says
  otherwise).

### 5.3 Death, level loss, and redistribution ("all stages")

This rule is required for sandbox QA behavior.

- **Trigger:** when the sandbox player dies (including via explicit "Die" debug
  action) and sigma mode is not preventing death, apply sandbox death
  progression handling instead of (or in addition to) normal campaign death flow
  as defined for sandbox.
- **Level loss:** reduce player level by `1`, with a minimum of level `1`.
- **Redistribution:** allocated attribute/status points are redistributed by
  resetting allocated attributes to their base state and returning the
  appropriate total unspent points for the resulting level/progression state, so
  the user can reassign them.
- **XP normalization:** XP is reset/normalized to a valid value for the
  resulting level (recommended: `0` for the new current level) to avoid invalid
  progression states.
- **Derived state:** recompute stats, clamp/restore HP and aura to valid values,
  and revalidate any progression-dependent availability (e.g. skill/loadout
  requirements) deterministically.
- **"In all stages" interpretation:** progression is canonical/global for the
  active character/profile context (not stored per stage). Therefore the
  level/attribute redistribution state applies everywhere that same progression
  state is used. In sandbox's isolated context, this means the sandbox profile's
  progression remains consistent across sandbox resets/reloads only as explicitly
  specified by session/reset policy.

> Implementation note: if campaign and sandbox share no progression state by
> default (recommended), sandbox death redistribution must not alter campaign
> save slots.

## 6. World editing rules

### 6.1 Coordinate targeting and input modes

- Placement/removal targets a map tile (and/or world position snapped to tile),
  derived from pointer/touch/controller cursor input via the client camera.
- Sandbox editing uses explicit tool modes to avoid accidental edits during
  normal movement/combat.
- Client provides preview feedback; core validates and applies the final
  mutation.

### 6.2 Block placement/removal

- **Place:** set target tile to selected block type if allowed.
- **Remove:** set target tile to empty (or equivalent removable state) if
  allowed.
- **Validation (core):** bounds checks, valid block ID, and sandbox-mode
  permission checks are required.
- **Safety constraints (recommended/default):**
  - Prevent edits outside map bounds.
  - Avoid placing solid blocks in a way that traps/invalidates critical sandbox
    setup unless explicitly allowed (QA may choose to allow with warnings).
  - Treat special tiles (spawn markers, flags, scripted/unique tiles) carefully;
    if not fully supported, reject with clear feedback.
- **Rendering/collision consistency:** map collision data and rendered tile
  visuals/caches must reflect runtime edits immediately.

### 6.3 Enemy placement

- Spawns an enemy entity of the selected species at the targeted position.
- **Validation (core):** valid species/type, in-bounds position, and sandbox-mode
  permission checks.
- **Recommended constraints:** avoid spawning inside solid tiles; snap/adjust to
  valid space when possible or reject with feedback.
- Spawned enemies follow normal core rules unless sandbox tooling specifies
  otherwise.

### 6.4 Item placement

- Spawns an item/pickup entity of the selected item kind at the targeted
  position.
- **Validation (core):** valid item kind, in-bounds position, and sandbox-mode
  permission checks.
- **Recommended constraints:** avoid invalid overlaps with solid tiles; allow
  multiple items only as supported by entity/item systems.

## 7. Architecture and implementation constraints

- `enzo-core` remains pure and authoritative (no render/IO dependencies).
- Debug/sandbox mutations that affect simulation (progression, sigma mode, death
  handling, spawning, map edits) must be implemented in core APIs/state and be
  testable headlessly.
- UI/DOM/bridge layers may select tools, show previews, and emit semantic
  actions, but must not compute authoritative outcomes.
- New UI actions/bridge payloads must be versioned/explicit and documented.
- Determinism: debug actions applied at defined tick/input boundaries must
  produce deterministic results for the same initial state and action sequence.

## 8. Telemetry, feedback, and QA ergonomics (recommended)

- Toast/log messages for unlock, sandbox start/reset, rejected edits, and debug
  actions.
- On-screen debug indicator (e.g. `SANDBOX`, `SIGMA`) when active.
- Optional counters/readouts: entity count, selected tile coords, current tool.
- Fast reset and clear actions to restore a known-good test state.

## 9. Testing and acceptance criteria

### 9.1 Core tests (required targets)

- Sandbox progression helpers: grant/set/level-down/redistribute keep valid,
  deterministic state.
- Sigma mode prevents configured damage/fall-death outcomes.
- Death in sandbox applies level loss + redistribution rule correctly and clamps
  to minimum level.
- Block place/remove mutates map correctly and preserves collision/query
  consistency.
- Enemy/item spawn APIs create valid entities at valid positions and reject
  invalid requests safely.

### 9.2 Client/UI validation (required targets)

- Secret `67` unlock flow works (keyboard and touch-accessible path).
- Sandbox menu entry appears only when unlocked and launches the flat map.
- All 4 debug tabs render and emit correct semantic actions.
- Placement tools target correct tiles/positions and provide user feedback on
  success/failure.
- Normal campaign flow and saves remain unaffected by sandbox usage.

### 9.3 Documentation

- `docs/SPEC.md` and `docs/ARCHITECTURE.md` reference this spec and summarize
  sandbox/debug behavior and boundaries.
- Any finalized payload/action codes and persistence keys are documented in code
  and relevant docs when implemented.
