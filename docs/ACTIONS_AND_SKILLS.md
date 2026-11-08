# EnzoBallZ — Action Slots and Modular Skills

> **Status (audited):** This document is the target contract/plan for action
> slots, modular skills, and mobile controls. Parts of it are now implemented.
> `docs/ARCHITECTURE.md` and `docs/SPEC.md` have been updated to reflect the
> current codebase; this file remains the detailed design/reference and tracks
> what is done vs. pending.
>
> **Goal of this document:** define the vocabulary, contracts, and implementation
> order for the controls refactor (slot emulation) and skill modularization.

## 0. Implementation status audit

| Area | Status | Notes |
| --- | --- | --- |
| `InputAction` / `ActionSlot` / `SlotInput` / `PlayerCommand` (core) | **Done** | Core input contract uses 4 generic slots; no fixed `jump`/`dash`/`melee`/`mogging` fields in `PlayerCommand`. |
| `SkillId` / skill registry / `SkillDef` / requirements / loadout (core) | **Done** | Skills live in `crates/enzo-core/src/skills/`; static registry + `match` dispatch; `Loadout` enforces uniqueness (`assign` swaps, `sanitize` on load). |
| `Locomotion` + `Skills` replacing old `Movement`/`Combat` flow (core) | **Done** | `mechanics/` holds walk/SixSeven helpers; `skills/` holds slotted abilities; `World` dispatches generically. |
| Loadout persistence + snapshot/catalog exposure | **Done** | `SAVE_VERSION = 2` (v1 migrated to default loadout), `SNAPSHOT_VERSION = 3` with loadout/slot/catalog data. |
| Client keyboard → slot commands | **Done (basic)** | `enzo-wasm/src/input.rs` maps defaults (move/arrows, slots `Space/Shift/J/K`, SixSeven `6/7`). **QoL addition:** `W`/`↑` also triggers slot 1 (jump) on press while preserving `move_y=-1` for fly intent. |
| Render modularization (client) | **Done** | `render/world.rs` orchestrates; projectile/attack visuals in `render/skills/*`; entity/character animations in `render/entities/*`. |
| Mogging rename | **Done** | `Mogar` → `Mogging` across core/docs/scripts where migrated. |
| Skill assets (`assets/skills/<id>/`) | **Not done** | Icons/entity/particle assets not generated/wired as skill-owned files; render/HUD use placeholders/shared handling in places. |
| Functional loadout-editing UI | **Partial** | DOM HUD/Skills panel can select a slot and assign/clear **unlocked** skills via `loadoutAssign`/`loadoutClear` (core-authoritative). Full UX remains pending: requirement lock reasons/toasts in editor, swap/move clarity in all flows, polish, and pause/loadout-edit flow integration. |
| `KeyBindings` persistence + rebind UI | **Not done** | Defaults are hardcoded; no `enzoballz.bindings.v1` model/UI yet. |
| HTML/mobile/DOM bridge (`UiState`, exports, touch) | **Partial** | Implemented in parts: Rust `ui_bridge`/`ui_controller` export `UiState` JSON and semantic actions (`enzo_ui_action`, `enzo_loadout_assign/clear`, `enzo_action_down/up`); `web/ui/*` renders DOM menus, HUD tabs/panels (Status point distribution, Skills, Equipment, Items), and mobile controls. Remaining/planned: full HTML-bridge packaging as originally scoped, broader touch UX polish, and eventual migration of DOM UI implementation to Vue/Vite if adopted. |
| Placeholder skills finalized (`NutellaShot`, `Shout`) | **Partial** | Modules/placeholders exist; balance, visuals, and gameplay integration need refinement. |
| Expanded slot/loadout/save/determinism scenario tests | **Partial** | Core tests pass; targeted slot/loadout/cooldown/lock/save migration scenario coverage should be expanded (see plan). |

## 1. Why refactor

Previously, input was coupled to specific abilities:

- `PlayerCommand` had `jump`, `jump_held`, `dash`, `melee`, and `mogging`.
- Movement/combat code translated each field to a fixed component.
- `World::step` called melee/Mogging logic directly.
- The skills HUD listed move IDs, but there was no notion of an “equipped slot”.
- Changing which ability a button used required touching input, core, and HUD.

The target separates four things that were mixed together:

1. **Physical key** (client): `A`, `Space`, touch button, etc.
2. **Semantic action** (shared contract): move left, use slot 2.
3. **Equipped slot** (core rule): which `SkillId` is in slot 1–4.
4. **Skill** (core module): requirements, state, activation, cost, and assets.

This lets the player swap jump, dash, punch, Mogging, fly, Nutella shot, or
shout between slots without the keyboard, HUD, or network knowing the skill.

## 2. Decisions

| Topic | Decision |
| --- | --- |
| Slots | 4 **generic** slots; any skill in any slot |
| Category | `Movement`/`Attack`/`Utility` is UI metadata, not a slot restriction |
| Loadout owner | Core rule in `enzo-core`; persists in save and appears in snapshot |
| Duplicates | Impossible by invariant; assigning an already-equipped skill **swaps** slots |
| Names | `InputAction`, `ActionSlot`, `SkillId`, `SkillDef`, `Loadout` |
| Representation | Enum + static registry (`SkillDef`), typed state, `match` dispatch |
| Traits | No `Box<dyn Skill>`; no per-skill allocation |
| Base movement | Walking and SixSeven remain mechanics outside slots |
| SixSeven | Still `6`/`7`, outside slots and loadout |
| Defaults | Slot 1 = Jump (`Space`), 2 = Dash (`Shift`), 3 = Melee/Punch (`J`), 4 = Mogging (`K`) |
| Movement | `A`/`D` or `←`/`→`; `W`/`S` or `↑`/`↓` for up/down (fly intent) |
| Attributes | `1`–`4` still distribute points on the level-up screen |
| Assets | `assets/skills/<id>/` with required icon and optional entity/particle art |
| Swap UX | Open skills menu → click slot → click desired skill |
| Pause | Proposal: pause simulation while the skills menu is open |
| Current reality note | Core/client migration is implemented in large part; loadout UI, keybindings, assets, and HTML bridge remain pending/partial |

### 2.1 Default behavior change

The jump is no longer `Space` **or** `W`/`↑` as a fixed command field; it is
**slot 1** (`Space` by default). `W`/`↑` provide the semantic **up/fly intent**
action (`move_y`). As a quality-of-life keyboard mapping, the client currently
also triggers slot 1 jump on `W`/`↑` press while preserving `move_y=-1`. If the
future keybinding editor allows remapping, conflicts must be handled explicitly.

## 3. Vocabulary and layers

```text
┌─────────────────────────────────────────────────────────────────────┐
│ Client (enzo-wasm / HTML)                                           │
│  keyboard ─┐                                                        │
│  touch  ───┼─▶ InputAction ─▶ PlayerCommand ─────────────────┐    │
│  mouse  ───┘   (semantic)       (tick contract)              │    │
│                                                              ▼    │
│                                                       ┌──────────┐│
│                                                       │ enzo-core││
│  UI ◀── WorldSnapshot ◀── World::step ◀── TickInput ─┤  rules   ││
│                                                       └──────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

| Layer | Lives in | Responsibility |
| --- | --- | --- |
| Physical key | `enzo-wasm` / HTML | `KeyCode`, touch button, future gamepad |
| `InputAction` | `enzo-core::input` | Semantic, serializable vocabulary |
| `PlayerCommand` | `enzo-core::input` | Tick intent, independent of physical key |
| `ActionSlot` | `enzo-core::input` | Slot 1–4, stable order |
| `Loadout` | `enzo-core::skills` | `slot → SkillId`, with uniqueness invariant |
| `SkillId`/`SkillDef` | `enzo-core::skills` | Stable identity and static metadata |
| Skill state | `enzo-core::skills` | Timers, buffers, cooldowns, active state |
| Snapshot | `enzo-core::snapshot` | Loadout, slot status, and catalog for UI |

The client **does not** decide whether a skill can be equipped, whether the
player has enough aura, whether a skill is on cooldown, or whether an item was
lost. It only sends actions and renders the snapshot.

Do not confuse the three kinds of “slots”: `ActionSlot` is the ability button
(1–4), `EquipSlot` is the equipment slot (weapon/armor/accessory), and “save
slot” is the progression file (0–2).

## 4. Input abstraction

### 4.1 `InputAction`

`InputAction` is the smallest unit the player can trigger. It does not know
physical keys or game rules.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionSlot {
    Slot1,
    Slot2,
    Slot3,
    Slot4,
}

impl ActionSlot {
    pub const COUNT: usize = 4;
    pub const ALL: [ActionSlot; 4] = [
        ActionSlot::Slot1,
        ActionSlot::Slot2,
        ActionSlot::Slot3,
        ActionSlot::Slot4,
    ];

    /// 0-based index used in arrays and snapshots.
    pub const fn index(self) -> usize {
        match self {
            ActionSlot::Slot1 => 0,
            ActionSlot::Slot2 => 1,
            ActionSlot::Slot3 => 2,
            ActionSlot::Slot4 => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InputAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    Slot1,
    Slot2,
    Slot3,
    Slot4,

    Six,
    Seven,

    AllocateSpeed,
    AllocateStrength,
    AllocateHealth,
    AllocateAura,
    AllocateEgo,

    Confirm,
    Restart,
    OpenMenu,
    OpenSkills,
}
```

Rules:

- `InputAction` is **not** read by the core; the client translates it into
  `PlayerCommand`. The core still does not know the keyboard.
- `OpenMenu` and `OpenSkills` are UI actions consumed by the client; they do not
  enter `PlayerCommand`.
- Mouse/touch clicks on menu/HUD buttons still produce UI actions, not
  `InputAction`.
- `InputAction::code()` is append-only: reordering breaks the JS bridge and
  input replays.

### 4.2 `SlotInput` and `PlayerCommand`

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotInput {
    /// Edge: true only on the tick when the button was pressed.
    pub pressed: bool,
    /// Continuous state: true while the button is held.
    pub held: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerCommand {
    pub move_x: i8, // -1, 0, 1
    pub move_y: i8, // -1, 0, 1 (fly/vertical intent)
    pub slots: [SlotInput; ActionSlot::COUNT],
    pub sixseven: Option<SixSevenKey>,
    pub allocate: Option<Attribute>,
    pub confirm: bool,
    pub restart: bool,
}
```

`PlayerCommand::without_edges()` keeps only continuous state (including slot
`held`) and clears edges/meta triggers.

### 4.3 Edge and held semantics

- `pressed` is an **edge**: the client accumulates presses between frames and
  consumes them on the first tick of the frame.
- `held` is **state**: it remains valid on extra ticks and while the button is
  held.
- The client must not apply the same edge twice: `without_edges()` clears only
  `pressed`.
- Skills that use both (variable jump) read `pressed` to start and `held` to
  sustain. Tap skills (dash, melee, Mogging) read only `pressed`. Continuous
  skills (fly) read `held` and `move_y`.
- The core never receives browser key repeat. Auto-repeat is filtered by the
  client.

### 4.4 `KeyBindings` (client, planned)

Physical keys belong in the client because `KeyCode` is a platform detail and
must not enter the core.

```rust
// crates/enzo-wasm/src/input/bindings.rs (planned)
pub struct KeyBindings {
    /// One action may have multiple keys (e.g., A and Left Arrow).
    keys: [Vec<KeyCode>; InputAction::COUNT],
}
```

- Persistence: `enzoballz.bindings.v1` in `localStorage`, JSON with stable key
  names (`"Space"`, `"LeftShift"`, `"ArrowLeft"`), independent of macroquad
  `KeyCode`.
- Bindings are a device preference: they do not go into `SaveData`.
- Key conflicts are rejected by default in the editor. Defaults have no
  conflicts.
- The first refactor step may use `KeyBindings::default_player()` + persistence
  without a rebind UI.
- The HUD shows the slot key label, not the skill name.

### 4.5 Defaults

| Action | Default keys | Notes |
| --- | --- | --- |
| `MoveLeft` | `A`, `←` | continuous |
| `MoveRight` | `D`, `→` | continuous |
| `MoveUp` | `W`, `↑` | continuous; fly intent (`move_y=-1`); **client QoL:** also triggers slot 1 jump on press |
| `MoveDown` | `S`, `↓` | continuous; fly intent |
| `Slot1` | `Space` | default skill: Jump |
| `Slot2` | `Left/Right Shift` | default skill: Dash |
| `Slot3` | `J` | default skill: Melee |
| `Slot4` | `K` | default skill: Mogging |
| `Six` | `6` | SixSeven, outside slots |
| `Seven` | `7` | SixSeven, outside slots |
| `AllocateSpeed` | `1` | level-up screen only |
| `AllocateStrength` | `2` | level-up screen only |
| `AllocateHealth` | `3` | level-up screen only |
| `AllocateAura` | `4` | level-up screen only |
| `AllocateEgo` | `5` (suggested) | level-up screen only; Ego (intelligence) increases aura attack power |
| `Confirm` | `Enter` | level-up / modals |
| `Restart` | `R` | death/victory |
| `OpenMenu` | `Esc` | saves and returns to menu |
| `OpenSkills` | `Tab` (suggested) | should call `preventDefault`; fallback `Q` if needed |

### 4.6 Action → command mapping

```text
MoveLeft/MoveRight  -> move_x
MoveUp/MoveDown     -> move_y
Slot1..Slot4        -> slots[0..3] = SlotInput { pressed, held }
Six/Seven           -> sixseven
Allocate*           -> allocate
Confirm             -> confirm
Restart             -> restart
OpenMenu/OpenSkills -> client UI (not sent to core)
```

`move_x` and `move_y` are digital and cancel opposites:

```rust
pub fn axis(negative: bool, positive: bool) -> i8 {
    match (negative, positive) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}
```

Running via double-tap remains a core rule (`Walk`), not an `InputAction`. The
client only sends continuous `move_x`.

### 4.7 Mobile and HTML (planned)

Touch controls do **not** need `KeyBindings`: they emit `InputAction` directly.

- Left joystick/D-pad: `MoveLeft`/`MoveRight`/`MoveUp`/`MoveDown` (held).
- Four skill buttons: `Slot1`..`Slot4` (`pressed` on `pointerdown`, `held` until
  `pointerup`/`pointercancel`).
- `6`/`7` buttons: `Six`/`Seven`.
- Menu/skills buttons: `OpenMenu`/`OpenSkills`.

The HTML bridge is detailed in section 9.6. The important rule: HTML emits
**semantic actions**, never `PlayerCommand` fields or game rules.

## 5. Loadout

### 5.1 Invariants

1. There are exactly 4 slots (`ActionSlot::COUNT`).
2. Each slot contains `Option<SkillId>`; an empty slot is valid.
3. **The same `SkillId` never appears in two slots.**
4. Slot order is stable: `Slot1` → index 0, ..., `Slot4` → index 3.
5. Loadout is serializable, `Copy`, and deterministic.
6. Loadout belongs to the player; enemies and projectiles have empty loadouts.
7. Changing loadout does not cost aura, XP, or gameplay time.
8. Losing a skill's requirements does **not** remove the skill from the loadout;
   it becomes **locked** until requirements are met again.

### 5.2 Representation

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Loadout {
    slots: [Option<SkillId>; ActionSlot::COUNT],
}
```

JSON save shape:

```json
"loadout": ["jump", "dash", "melee", "mogging"]
```

An empty slot is `null`. `SkillId` uses stable `snake_case`.

### 5.3 Operations and duplicates

`assign(slot, skill)` has three cases:

1. **Already in the slot**: no-op (`LoadoutChange::None`).
2. **In another slot**: swaps the two slots (`Swapped`). If the target slot is
   empty, this effectively moves the skill.
3. **Not equipped**: replaces the slot contents (`Assigned`), keeping the previous
   skill.

Swapping instead of rejecting is better UX and preserves uniqueness by
construction.

The save is untrusted input: `sanitize()` runs on migration/load and removes
duplicates while keeping the first occurrence in slot order. No other part of the
game needs to check for duplicates at runtime.

### 5.4 Availability

Being in the loadout does not mean being usable. Availability is computed in the
core from `SkillDef.requirements` + player context.

```rust
pub struct SkillContext {
    pub entity_type: EntityType,
    pub level: u32,
    pub attributes: AttributeSet, // character + equipment
    pub equipment: EquipmentTags,
}
```

Rules:

- **Skill modules declare requirements** in `skills/<id>/mod.rs` via `REQUIREMENTS`
  / `SkillDef::requirements`: entity type (e.g. playable), minimum level, minimum
  attributes/status (including `Ego` when relevant), and required equipment tags.
  The `SkillContext` used for checks includes character attributes plus equipment
  modifiers.
- **Equippable items and weapons declare status requirements** in
  `entities/item/<id>/mod.rs` via `ItemRequirements` (`entity_types`,
  `min_attributes`). The core validates these requirements before equipping/using;
  items may also grant equipment tags/modifiers that unlock or empower skills.
- `SkillDef::requirements.check(context) -> Result<(), SkillLock>` returns the
  **first** unmet requirement in a fixed order: entity type, level, attributes,
  equipment.
- When activating a slot, the core **revalidates** availability. A locked skill
  does not activate, even if the client requests it.
- Level-up and equipment changes can unlock or lock skills. The loadout does not
  change by itself; the slot becomes locked and the menu shows the reason.
- Item-granted skills (e.g., Nutella Gun) use an equipment tag. While the item is
  equipped, the skill is available; when unequipped, it remains in the slot but
  locked.
- The skills menu should list all `SkillId`s, with lock/reason for unavailable
  ones. No separate “purchase/unlock” system in this plan; requirement met =
  available.

### 5.5 Core API

Changing loadout is a sparse meta-state operation, not a 60 Hz button. Therefore
it does **not** go into `PlayerCommand`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadoutRequest {
    Assign { slot: ActionSlot, skill: SkillId },
    Clear { slot: ActionSlot },
}

impl World {
    pub fn apply_loadout(
        &mut self,
        player_id: PlayerId,
        request: LoadoutRequest,
    ) -> Result<LoadoutChange, LoadoutError>;
}
```

Rationale:

- Loadout changes happen while the game is paused; `PlayerCommand` is the
  simulation tick contract.
- It is sparse and should not take space in the hot 60 Hz path.
- It is analogous to `World::equip`, already a core API rather than a tick
  command.
- In future multiplayer, `LoadoutRequest` becomes a reliable (sequenced)
  message, not a movement command.

For replays/determinism, `enzo-sim` should apply the same `LoadoutRequest`
sequence at the same tick boundaries.

`apply_loadout`:

1. validates that the player exists and is alive;
2. validates `SkillDef.requirements` against current context;
3. applies `Loadout::assign`/`clear`;
4. emits `GameEvent::LoadoutChanged`;
5. requests save (`GameEvent::SaveRequested`).

## 6. Skills

### 6.1 `SkillId`

`SkillId` is the stable identity of an equippable skill. Walking and SixSeven
are **not** `SkillId`s.

- `SkillId` uses stable `snake_case` in serialization.
- `ALL` order is iteration/cooldown order; a test should guarantee
  `ALL[i].index() == i`.
- `SkillId` is append-only at the end to avoid breaking saves/snapshots.
  Removing or reordering requires migration.

### 6.2 `SkillDef`

Static metadata, no allocation and no trait objects:

```rust
pub struct SkillDef {
    pub id: SkillId,
    pub category: SkillCategory,   // Movement/Attack/Utility (UI metadata)
    pub source: SkillSource,       // Innate/Progression/Equipment
    pub activation: Activation,     // Press/Hold/PressHold
    pub requirements: Requirements,
    pub aura_cost: f32,
    pub cooldown: f32,
    pub timing: SkillTiming,       // PrePhysics/PostPhysics
    pub assets: SkillAssets,
}

pub struct SkillAssets {
    /// Required for every SkillId.
    pub icon: &'static str,
    /// Entity spawned by the skill (projectile, wave, etc.).
    pub entity: Option<&'static str>,
    /// Particles/effect for the skill.
    pub particles: Option<&'static str>,
}
```

Module rules:

- One file per skill (`skills/mogging.rs`); if it grows, make it a directory.
- The module declares `DEF`, typed state, pure functions, and tests.
- The module must not import `macroquad`, `wasm-bindgen`, files, clock, or
  network.
- Entities/particles are created via `EntitySpec` + `EntityStore::spawn`.
- The module does not format UI text; the client translates `SkillId`/`SkillLock`.
- The module declares its own assets; there is no parallel asset list.

### 6.3 Initial catalog

| Skill | Category | Activation | Requirements | Cost | Cooldown | Phase | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Jump (`Jump`) | Movement | PressHold | playable, level 1 | 0 | 0 | Pre-physics | Implemented (core) |
| Dash (`Dash`) | Movement | Press | playable, level 1 | 0 | 0.70 s | Pre-physics | Implemented (core) |
| Fly (`Fly`) | Movement | Hold | playable, level 5, Aura 5 | 0 | 0 | Pre-physics | Implemented (core) |
| Melee (`Melee`) | Attack | Press | playable, level 1 | 0 | combo window | Post-physics | Implemented (core) |
| Mogging (`Mogging`) | Attack | Press | playable, level 1 | 25 | 0.35 s | Post-physics | Implemented (core) |
| Nutella Shot (`NutellaShot`) | Attack | Press | playable, level 1, `NUTELLA_GUN` | 0 | 0.25 s | Post-physics | Placeholder (core) |
| Shout (`Shout`) | Utility | Press | playable, level 1 | 10 | 2.0 s | Post-physics | Placeholder (core) |

Notes:

- Nutella Shot/Shout numbers are placeholders; balancing comes later.
- `Melee` is the current “punch” (3-hit combo). Internal name remains `Melee`;
  UI may display “Punch”.
- `Fly` uses `move_y`; it does not need a separate action beyond the slot.
- `SixSeven` is not a slot skill; it remains in `mechanics/sixseven.rs`.

### 6.4 Runtime state

`Skills` unifies the old skill-capable parts of movement/combat. `Locomotion`
holds only what is not a slot.

- `Entity` has `locomotion: Locomotion` and `skills: Skills`.
- State for all skills exists (no dynamic allocation), but only equipped skills
  respond to input.
- Cooldowns live in a central array so they cannot be bypassed by swapping
  slots. A skill unequipped and re-equipped remains on cooldown.
- Internal timers (combo window, jump buffer, active dash) live in the module
  state.
- `Skills::tick(dt)` advances cooldowns/timers once per tick.

### 6.5 Activation, aura, and cooldown

Generic flow for `PostPhysics` skills:

```text
for each player:
  for each slot:
    if slot empty: continue
    if skill locked: emit SkillRejected(Unavailable); continue
    if cooldown > 0: emit SkillRejected(Cooldown); continue
    if aura < def.aura_cost: emit SkillRejected(NoAura); continue
    if input does not trigger activation: continue
    execute skill activation
    if Started:
      aura -= def.aura_cost
      cooldowns[skill] = def.cooldown
      emit SkillActivated
```

- Aura is spent only when the skill actually **starts**. A skill that fails due
  to an internal condition returns `Rejected` and does not charge.
- The cooldown is set on activation and ticked after activation, so it remains
  full on the activation tick and starts decreasing on the next tick
  (deterministic and testable).
- `PrePhysics` skills have their own flow: jump buffers, dash consults/fills the
  central cooldown, fly is continuous. The generic aura/cooldown flow above
  applies to `PostPhysics` skills.
- Aura cost is **one-shot** per activation in this plan. Continuous drain (e.g.,
  fly consuming aura per second) is a future `SkillDef` extension, not a current
  requirement.

### 6.6 `World::step` system order

Order is a contract. The refactor replaces the old movement/combat stages with a
skill pipeline while keeping the rest:

1. **input**: apply `PlayerCommand` to controllers (`move_x`, `move_y`, slots,
   SixSeven).
2. **sixseven**: consume `6`/`7` input, update aura/combo/slowdown.
3. **skills pre-physics**: walking, jump buffer, dash request, fly vertical
   intent.
4. **physics**: integrate body and resolve tile collisions. Dash overrides
   horizontal velocity; fly overrides gravity behavior as specified; SixSeven
   applies slowdown.
5. **skills post-physics**: melee, Mogging, Nutella shot, shout, and any
   `PostPhysics` skill. Hitboxes/projectiles use the integrated position.
6. **projectiles**: move projectiles and resolve tile/enemy collisions.
7. **ai**: patrol and enemy intent.
8. **collisions**: contact damage, i-frames, knockback.
9. **skills tick**: cooldowns, windows, timers.
10. **progression**: XP, level-up, points.
11. **cleanup**: remove dead entities and expired projectiles.
12. **events**: return tick events.

Changing this order changes replay/network behavior and requires updating this
document and determinism tests.

## 7. Skill assets

### 7.1 Layout

```text
assets/
├── sprites/                  # current atlases (transition)
└── skills/
    ├── jump/
    │   ├── icon.png          # required
    │   └── dust.png          # optional (particles)
    ├── dash/
    │   ├── icon.png
    │   └── trail.png
    ├── fly/
    │   ├── icon.png
    │   └── aura.png
    ├── melee/
    │   ├── icon.png
    │   └── impact.png
    ├── mogging/
    │   ├── icon.png
    │   ├── projectile.png    # spawned entity
    │   └── impact.png
    ├── nutella_shot/
    │   ├── icon.png
    │   ├── projectile.png
    │   └── splash.png
    └── shout/
        ├── icon.png
        └── wave.png
```

- `<id>` is the stable `snake_case` `SkillId`.
- `icon.png` is required for **every** `SkillId`.
- Movement actions (`MoveLeft`/`MoveRight`/`MoveUp`/`MoveDown`) use standard UI
  arrows, not skill icons.
- `entity`/`particles` are optional and declared in `SkillDef`.
- Icons: RGBA PNG, transparent background, pixel-art, 32×32 (or 24×24 if the
  atlas requires). HUD scaling should be integer to avoid blur.
- Each icon must be readable at 32 px and visually distinct.
- No assets copied from existing works; the versioned generator is the source.

### 7.2 `SkillAssets` contract

```rust
pub struct SkillAssets {
    pub icon: &'static str,
    pub entity: Option<&'static str>,
    pub particles: Option<&'static str>,
}
```

- Paths are relative to the served root (`dist/`), so they work at domain root
  and in subdirectories (e.g., GitHub Pages).
- The core does not read files; it only carries the strings.
- HTML UI can use paths directly in `<img src>`.
- Canvas client preloads icons/entities via `load_texture` at startup; on
  failure it uses a colored placeholder with the skill initial. Never panic due
  to missing assets.

### 7.3 Pipeline

- `scripts/gen-sprites.mjs` generates PNGs for `assets/skills/<id>/`
  deterministically.
- `scripts/build-wasm.sh` copies `assets/` to `dist/assets/`; no extra copy step
  is required.
- A host test (in `enzo-wasm/tests/` or an npm script) should verify that every
  `SkillId::ALL` has a non-empty icon and an existing file. The pure core does
  not do IO; that test does not belong in `enzo-core`.
- `SkillDef` is the single source of truth for the catalog; do not create a
  parallel `manifest.json`. HTML UI should receive the catalog through `UiState`.

## 8. Snapshot, save, and networking

### 8.1 Snapshot (`SNAPSHOT_VERSION = 3`) — **implemented**

The snapshot exposes loadout, slot status, and catalog data for the UI (read-only
on the client). Key concepts:

- `loadout`, `slots`, and `catalog` are read-only in the client.
- `catalog` is per player because availability depends on level, attributes, and
  equipment.
- `lock` is `None` when available; otherwise it carries the reason for the UI.
- `cooldown`/`cooldown_max` drive radial overlays and text.
- `active` covers `Hold` skills (e.g., fly).
- `melee_step`/`melee_timer` remain melee-specific visual state; generalize to
  per-slot state if a second melee skill is added.
- Old `moves`/fixed Mogging cooldown fields are replaced by slot/catalog data.
- Snapshots are not persisted, so no snapshot migration is needed; all consumers
  must compile against the new version.

### 8.2 Save (`SAVE_VERSION = 2`) — **implemented**

```rust
pub struct SaveData {
    pub version: u16, // 2
    pub level: u32,
    pub xp: f32,
    pub points: u32,
    pub attributes: Attributes,
    #[serde(default)]
    pub loadout: Loadout,
}
```

- `loadout` is `#[serde(default)]` so v1 JSON (without the field) remains
  deserializable.
- When reading `version == 1`, migration fills `Loadout::player_default()`,
  sets `version = 2`, and saves to v2.
- When reading `version == 2`, the loadout is preserved exactly (including all
  empty, if the player cleared all 4 slots).
- `Loadout::sanitize()` runs after parsing to remove duplicates from corrupted
  saves.
- `SaveData::apply_to` accepts only the current save version; migration happens
  before apply.

Keys (target/current intent):

| Key | Use |
| --- | --- |
| `enzoballz.save.slot.{0..2}.v2` | new save |
| `enzoballz.save.slot.{0..2}.v1` | legacy fallback, migrated to v2 |
| `enzoballz.save.v1` | slot 0 fallback, migrated to v2 |
| `enzoballz.bindings.v1` | keys, separate from save (planned) |
| `enzoballz.language` | language |
| `enzoballz.interface_scale` | UI scale |

- v1 is not deleted automatically; it is read and v2 is written.
- Equipment is not fully persisted in the MVP. An item-granted skill remains in
  the loadout but may be **locked** after load until the item is equipped again
  (open decision/behavior to confirm as implementation evolves).

### 8.3 Events — **implemented (generic skill events)**

Events include generic skill activation/rejection/hit/projectile/loadout changes
(e.g., `SkillActivated`, `SkillRejected`, `SkillProjectileSpawned`, `SkillHit`,
`LoadoutChanged`) with reasons like unavailable/cooldown/no aura/empty slot.

- The client maps `SkillId` to effects/particles and toasts.
- Old Mogging-specific events may be removed or aliased during incremental
  migration; cleanup should happen in the same commit as behavior-preserving
  replacement.
- `SaveRequested` is emitted after `LoadoutChanged`.

### 8.4 Determinism and replays

- `PlayerCommand`, `SlotInput`, `Loadout`, `SkillId`, and `LoadoutRequest` are
  `Copy`, `Eq`/deterministic-friendly.
- No `HashMap`/`HashSet` in logic; loadout is an array, catalog iteration
  follows `SkillId::ALL`.
- `TickInput` replays remain valid. Replays that change loadout must record
  `(tick, player, LoadoutRequest)` and apply before the corresponding tick.
  `enzo-sim` should support that field in replay format.
- Determinism test: compare `WorldSnapshot` hash after the same seed + same
  `TickInput` sequence + same `LoadoutRequest` sequence.

## 9. UI and UX

### 9.1 Action bar (HUD) — **partial**

Replace the fixed Mogging indicator with a 4-slot bar:

```text
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│ [1]  │ │ [2]  │ │ [3]  │ │ [4]  │
│ icon │ │ icon │ │ icon │ │ icon │
│ Jump │ │ Dash │ │Melee │ │Mogging│
└──────┘ └──────┘ └──────┘ └──────┘
```

- Fixed order `Slot1` → `Slot4`, left to right.
- Each slot shows: icon, key label, short name, aura cost, and cooldown.
- Visual states: ready, cooldown (overlay + remaining time), insufficient aura,
  locked (lock + reason tooltip), empty (`+` / “Empty”), active (`Hold` glow).
- An empty slot should not be a silent dead key: show the key label and “Empty”.
- The bar must not cover mobile controls; in HTML/touch mode it can be drawn by
  HTML while the canvas hides the keyboard version.

### 9.2 Skills menu — **not done (functional editor pending)**

Modal overlay opened by `OpenSkills` (or the HUD skills dot):

1. Player opens the menu; game pauses.
2. Click the desired slot (1–4). Selected slot is highlighted.
3. Click the desired skill in the catalog.
4. Core validates availability and applies `LoadoutRequest::Assign`.
5. UI updates icons, shows a toast, and saves.
6. If the skill was already in another slot, the two slots **swap**; UI warns.
7. `Clear slot` removes the skill; the slot becomes empty.
8. Close (`Esc`, `×`, or Back) resumes simulation.

Details:

- Unavailable skills show a lock, reduced opacity, and reason (e.g., “Level 5”,
  “Aura 5”, “Requires Nutella Gun”).
- Already-equipped skills show an “Slot N” badge; clicking with another slot
  selected swaps/moves.
- Catalog is grouped by `SkillCategory` (Movement/Attack/Utility), but any skill
  can go in any slot.
- The menu is read-only over the snapshot; it does not compute requirements or
  duplicates.
- UI state (`selected_slot`, open state, scroll) lives in the client, not in
  core/snapshot.
- Do not open during `Dead`, `Victory`, or `LevelUp` unless explicitly decided.

### 9.3 Pause — **proposal/partial**

Default proposal: **pause simulation** while the skills menu is open, reusing the
session pause state (like the Escape menu).

- The world stops advancing; the last snapshot remains rendered behind the
  overlay.
- Loadout changes are applied via `World::apply_loadout` (core API) without
  needing a tick.
- On close, the client refreshes the snapshot (`world.write_snapshot`) and
  resumes.
- In future multiplayer, menus do not pause the server; each client may open its
  own menu, and loadout changes are server-validated messages.
  `LoadoutRequest` is designed for that.

### 9.4 Feedback and errors

- Success: toasts like “Mogging moved to Slot 1”, “Melee swapped with Mogging”,
  “Slot 3 cleared”.
- Failure: short toast with reason (`Insufficient level`, `Insufficient aura`,
  `On cooldown`) and slot highlight/vibration when the key is pressed.
- HUD should also react to core `SkillRejected` events; UI must not guess.
- No UI action changes aura, XP, health, or position.

### 9.5 Accessibility and i18n

- Icons always accompanied by text labels in the menu and action bar tooltip.
- Lock + text, not color only.
- Touch targets ≥ 44 px on mobile; high contrast; visible keyboard focus.
- Add `Text` keys for skill names/categories, slot labels/empty/clear, lock
  reasons, menu hints, equipped/swap/clear/unavailable messages, and action bar
  hints.
- Keep `Walk`/`SixSeven` terminology as mechanics (not skills) in UI text.
- Core exposes `SkillId`/`SkillLock`; the client chooses localized text.

### 9.6 HTML/mobile bridge (planned)

- HTML/CSS draws controls and menus; canvas draws the world.
- HTML emits `InputAction` (stable code) to wasm:
  - `enzo_action_down(code)`
  - `enzo_action_up(code)`
- Skills menu in HTML emits `LoadoutRequest`:
  - `enzo_loadout_assign(slot, skill)`
  - `enzo_loadout_clear(slot)`
- wasm → HTML sends `UiState` JSON (10–20 Hz or on change): health/aura/XP/
  level/points, loadout + 4 slot status, catalog (`SkillId`, localized name,
  icon, cost, cooldown, lock), phase, and toasts.
- HTML must **not** read/write saves, compute requirements, or change aura. It
  only renders state and emits intents.
- Bridge uses `miniquad_add_plugin` (`importObject.env`) and
  `#[no_mangle] extern "C"` exports, without `wasm-bindgen`/`web-sys`.
- `InputAction`/`SkillId`/`ActionSlot` codes are append-only; bridge is
  versioned with the bundle.

Mobile care:
- `touch-action: none`, `user-select: none`, `-webkit-tap-highlight-color`
- `pointerdown`/`pointerup`/`pointercancel` + `setPointerCapture`
- `tabindex="-1"` on buttons and `preventDefault` to avoid stealing canvas focus
- `100dvh` and `env(safe-area-inset-*)`
- landscape preferred; rotate warning in portrait
- one pointer id per button for multitouch
- hide canvas action bar when HTML controls are active

Suggested HTML UI phases:
1. mobile controls (highest value, lowest risk)
2. action bar and dots
3. simple overlays (toast, level-up, death/victory)
4. menus and delete modal
5. full skills menu
6. draggable windows

## 10. Target module structure

```text
crates/enzo-core/src/
├── input.rs                 # InputAction, ActionSlot, SlotInput, PlayerCommand
├── mechanics/               # not slots
│   ├── mod.rs               # Locomotion
│   ├── walk.rs              # walk/run/double-tap
│   ├── sixseven.rs          # 6/7 aura farming
│   └── combat.rs            # shared combat helpers (as implemented)
├── skills/
│   ├── mod.rs               # SkillId, SkillDef, Skills, dispatch
│   ├── registry.rs          # SkillId::ALL, def(), catalog
│   ├── requirements.rs      # Requirements, SkillContext, SkillLock, EntityType
│   ├── loadout.rs           # Loadout, LoadoutChange, LoadoutRequest
│   ├── jump.rs
│   ├── dash.rs
│   ├── fly.rs
│   ├── melee.rs
│   ├── mogging.rs
│   ├── nutella_shot.rs
│   └── shout.rs
├── entities/
│   ├── entity.rs            # Entity { locomotion, skills, ... }
│   ├── spec.rs
│   └── ...
├── world.rs                 # orchestrates the pipeline in section 6.6
├── snapshot.rs              # v3
├── progression.rs           # SaveData v2
└── ...
```

Removed (migration complete/target): `moves.rs`; `attacks/`; `movements/jump.rs`,
`dash.rs`, `fly.rs`; `Movement` and `Combat`; snapshot `moves`/fixed Mogging
cooldown fields.

Kept: `mechanics/walk.rs` and `mechanics/sixseven.rs` (not slots); `physics`,
`aura`, `ai`, `progression`, `map`, `entities`, `world`, `events`.

Client (current + target):

```text
crates/enzo-wasm/src/
├── input.rs                 # current keyboard input (slot mapping)
├── input/                   # planned: bindings.rs, virtual_input.rs
├── render/
│   ├── world.rs             # orchestration
│   ├── entities/            # character/entity rendering
│   └── skills/              # projectile/attack visuals
├── hud/
│   ├── action_bar.rs        # planned
│   ├── skills.rs            # loadout editor (partial/simplified now)
│   └── ...
├── ui_bridge/               # planned miniquad plugin + exports
└── ...
```

## 11. Incremental migration plan (audit)

| # | Suggested commit | Content | Status |
| --- | --- | --- | --- |
| 1 | `docs: plan action slots and modular skills` | this document + ARCHITECTURE/SPEC/AGENTS/README updates | **Done** (plan committed; docs further translated/audited now) |
| 2 | `refactor(core): add action slots and slot input` | `ActionSlot`, `SlotInput`, new `PlayerCommand`, `without_edges` | **Done** (as part of core migration) |
| 3 | `refactor(core): introduce SkillId and SkillDef registry` | `SkillId`/`SkillDef`/requirements/catalog | **Done** |
| 4 | `refactor(core): replace Movement/Combat with Skills/Locomotion` | move modules, `Skills`, `Locomotion`, `Entity`, dispatch | **Done** |
| 5 | `feat(core): add validated skill loadout` | `Loadout`, `LoadoutRequest`, `World::apply_loadout`, events | **Done** |
| 6 | `feat(core): persist loadout and expose skill catalog` | `SaveData` v2 + migration; snapshot v3 | **Done** |
| 7 | `feat(wasm): map keys to InputAction` | `KeyBindings`, defaults, persistence; rebind UI optional later | **Partial** (slot-based keyboard mapping done; `KeyBindings` model/persistence/UI pending) |
| 8 | `feat(wasm): add action bar and skill menu` | icons, loadout editor, pause | **Partial** (DOM HUD tabs/panels and basic skills assignment exist; full action bar/icons/locks/toasts/pause UX pending) |
| 9 | `feat(wasm): bridge HTML actions and loadout` | `UiState`, exports, virtual input | **Partial** (Rust bridge + DOM UI modules exist; Vue/Vite migration and remaining bridge/UX polish pending) |
| 10 | `feat(assets): generate skill icons and effects` | `gen-sprites.mjs`, assets, existence test | **Not done** |
| 11 | `test(core): cover slot swaps and requirement changes` | level-up/item/cooldown scenarios + determinism | **Partial** |

## 12. Risks and open decisions

| Risk/decision | State | Mitigation |
| --- | --- | --- |
| `W`/`↑` no longer fixed as jump-only | Decided/adjusted | `W`/`↑` carries `move_y` (fly intent); client also triggers slot 1 jump on press as QoL. A legacy preset can be offered in rebind UI. |
| Pause in skills menu | Proposed | Pause like Escape menu; validate with user/product choice. |
| Skill locked after losing item | Decided | Remains in loadout with lock; re-equipping reactivates. |
| Equipment not saved | Open | Save equipment in the future or accept locked skill after load. |
| Duplicate in corrupted save | Decided | `sanitize()` keeps first occurrence. |
| Cooldown when swapping slots | Decided | Cooldown is per skill, not per slot. |
| Aura cost on failed skill | Decided | Charge only when `Started`. |
| Continuous fly aura drain | Open | Future `aura_drain`; currently one-shot/hold behavior as implemented. |
| Key rebinding | Planned | Model now; UI later. |
| Key conflicts | Decided | Editor rejects; defaults have no conflicts. |
| `Tab` vs browser focus | Open | `preventDefault`; fallback `Q`. |
| Menu open during `LevelUp` | Open | Proposal: disable. |
| Utility skill (Shout) effect | Example/placeholder | Define effect before final implementation. |
| Unknown `SkillId` in future save | Open | Version + migration; avoid silent fallback without warning. |
| Catalog size in snapshot | Low cost now | ~7–12 entries per player; measure if it grows. |
| HTML UI duplicating rules | Risk | HTML only emits actions and renders `UiState`. |
| Canvas assets fail to load | Risk | Placeholder + log; host existence test. |

## 13. Acceptance criteria (refactor target, audited)

- [x] 4 generic slots exist; any available `SkillId` can occupy any slot (core).
- [x] `PlayerCommand` no longer has fixed `jump`, `jump_held`, `dash`, `melee`,
      or `mogging`; it has `move_x`, `move_y`, `slots[4]`, `sixseven`,
      `allocate`, `confirm`, `restart`.
- [x] `InputAction` is the semantic layer; core does not read the keyboard.
- [x] Loadout is validated in core, persists in save v2, and appears in snapshot
      v3.
- [x] The same skill cannot occupy two slots, including after loading a corrupted
      save (sanitize).
- [x] Assigning an already-equipped skill swaps, does not duplicate, and does not
      lose the other skill.
- [ ] Slots 1–4 can be rebound without touching core/HUD logic (bindings system
      pending).
- [x/partial] Skill locked by level/attribute/item is blocked by core checks;
      full UI reason display is partial.
- [ ] Every `SkillId` has an icon at `assets/skills/<id>/icon.png`; skills that
      spawn entities/particles declare paths and assets exist.
- [ ] Jump, Dash, Melee, and Mogging have dedicated skill-owned icons wired in
      UI/render.
- [x] SixSeven still works on `6`/`7`, outside slots.
- [x/partial] `World::step` order is documented; determinism/loadout scenario
      coverage should be expanded for all new behaviors.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
      -D warnings`, and `cargo test --workspace` pass in the current state.
- [x] `docs/ARCHITECTURE.md`, `docs/SPEC.md`, and `AGENTS.md` point to this
      document and reflect the current audited state (no contradictions about
      implemented slot/skill core migration).
