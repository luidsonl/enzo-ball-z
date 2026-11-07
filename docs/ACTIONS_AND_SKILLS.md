# EnzoBallZ — Action Slots e Skills Modulares

> **Status:** plano de arquitetura. Nada descrito aqui está implementado ainda.
> Este documento é a fonte de verdade do alvo; `docs/ARCHITECTURE.md` e
> `docs/SPEC.md` continuam descrevendo o código atual até a migração.
>
> **Objetivo desta etapa:** fechar o vocabulário, os contratos e a ordem de
> implementação da refatoração de controles (emulação de slots) e da
> modularização de habilidades.

## 1. Por que refatorar

Hoje o input é acoplado a habilidades específicas:

- `PlayerCommand` tem `jump`, `jump_held`, `dash`, `melee` e `mogar`.
- `Movement::apply_command` traduz cada campo para um componente fixo.
- `World::step` chama `melee::update` e `mogar::fire` diretamente.
- O HUD de habilidades lista `MoveId`, mas não existe noção de “slot equipado”.
- Trocar a habilidade de um botão exigiria mexer no input, no core e no HUD.

O alvo é separar quatro coisas que hoje estão misturadas:

1. **Tecla física** (cliente): `A`, `Espaço`, botão touch etc.
2. **Ação semântica** (contrato compartilhado): mover à esquerda, usar o slot 2.
3. **Slot equipado** (regra do core): qual `SkillId` está no slot 1–4.
4. **Habilidade** (módulo do core): requisitos, estado, ativação, custo e assets.

Assim, o jogador pode trocar pulo, dash, soco, Mogar, voo, arma de nutela ou
grito de slot sem que o teclado, o HUD ou a rede conheçam a habilidade.

## 2. Decisões fechadas

| Tema | Decisão |
| ---- | ------- |
| Slots | 4 slots **genéricos**; qualquer skill em qualquer slot |
| Categoria | `Movement`/`Attack`/`Utility` é metadado de UI, não restringe slot |
| Dono do loadout | Regra do `enzo-core`; persiste no save e aparece no snapshot |
| Duplicatas | Impossíveis por invariante; atribuir skill já equipada **troca** os slots |
| Nomes | `InputAction`, `ActionSlot`, `SkillId`, `SkillDef`, `Loadout` |
| Representação | Enum + registry estático (`SkillDef`), estado tipado, dispatch por `match` |
| Traits | Sem `Box<dyn Skill>`; sem alocação por skill |
| Movimento base | Andar e SixSeven continuam mecânicas fora dos slots |
| SixSeven | Continua em `6`/`7`, fora dos slots e do loadout |
| Defaults | Slot 1 = Pulo (`Espaço`), 2 = Dash (`Shift`), 3 = Soco (`J`), 4 = Mogar (`K`) |
| Movimento | `A`/`D` ou `←`/`→`; `W`/`S` ou `↑`/`↓` para subir/descer (voo) |
| Atributos | `1`–`4` continuam distribuindo pontos na tela de level-up |
| Assets | `assets/skills/<id>/` com ícone obrigatório e arte opcional de entidade/partículas |
| UX de troca | Abrir menu de skills → clicar no slot → clicar na skill desejada |
| Pausa | Proposta: pausar a simulação enquanto o menu de skills estiver aberto |
| Escopo agora | Somente documentação; nenhum código alterado |

### 2.1 O que muda no comportamento padrão

O pulo deixa de ser `Espaço` **ou** `W`/`↑` e passa a ser o **slot 1** (`Espaço`
por padrão). `W`/`↑` passam a ser a ação semântica **subir**, usada pelo voo e
por habilidades que aceitem movimento vertical. Para usar `W`/`↑` como slot, o
jogador precisa liberar a ação “subir” no editor: conflito de tecla é rejeitado
por padrão.

Essa mudança é intencional: se o slot 1 tiver Mogar em vez de Pulo, apertar
`W`/`↑` não pode disparar Mogar sem querer.

## 3. Vocabulário e camadas

```text
┌─────────────────────────────────────────────────────────────────────┐
│ Cliente (enzo-wasm / HTML)                                          │
│  teclado ─┐                                                         │
│  touch  ──┼─▶ InputAction ─▶ PlayerCommand ───────────────────┐     │
│  mouse  ──┘   (semântico)      (contrato do tick)             │     │
│                                                               ▼     │
│                                                        ┌──────────┐ │
│                                                        │ enzo-core│ │
│  UI ◀── WorldSnapshot ◀── World::step ◀── TickInput ───┤  regras  │ │
│                                                        └──────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

| Camada | Onde vive | Responsabilidade |
| ------ | --------- | ---------------- |
| Tecla física | `enzo-wasm` / HTML | `KeyCode`, botão touch, gamepad futuro |
| `InputAction` | `enzo-core::input` | Vocabulário semântico e serializável |
| `PlayerCommand` | `enzo-core::input` | Intenção do tick, independente da tecla |
| `ActionSlot` | `enzo-core::input` | Slot 1–4, ordem estável |
| `Loadout` | `enzo-core::skills` | `slot → SkillId`, com invariante de unicidade |
| `SkillId`/`SkillDef` | `enzo-core::skills` | Identidade e metadados estáticos da skill |
| Estado da skill | `enzo-core::skills` | Timers, buffers, cooldowns, estado ativo |
| Snapshot | `enzo-core::snapshot` | Loadout, status dos slots e catálogo para a UI |

O cliente **não** decide se uma skill pode ser equipada, se o jogador tem aura,
se está em cooldown ou se o item foi perdido. Ele só envia ações e desenha o
snapshot.

Não confundir os três “slots” do projeto: `ActionSlot` é o botão de habilidade
(1–4), `EquipSlot` é o slot de equipamento (arma/armadura/acessório) e “save
slot” é o arquivo de progressão (0–2).

## 4. Abstração de input

### 4.1 `InputAction`

`InputAction` é a menor unidade que o jogador pode acionar. Ela não conhece
tecla física nem regra de jogo.

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

    /// Índice 0-based usado em arrays e no snapshot.
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

    Confirm,
    Restart,
    OpenMenu,
    OpenSkills,
}

impl InputAction {
    pub const ALL: [InputAction; 18] = [/* ... */];
    pub const COUNT: usize = Self::ALL.len();

    pub const fn slot(self) -> Option<ActionSlot> {
        match self {
            InputAction::Slot1 => Some(ActionSlot::Slot1),
            InputAction::Slot2 => Some(ActionSlot::Slot2),
            InputAction::Slot3 => Some(ActionSlot::Slot3),
            InputAction::Slot4 => Some(ActionSlot::Slot4),
            _ => None,
        }
    }

    pub const fn sixseven(self) -> Option<SixSevenKey> {
        match self {
            InputAction::Six => Some(SixSevenKey::Six),
            InputAction::Seven => Some(SixSevenKey::Seven),
            _ => None,
        }
    }

    pub const fn attribute(self) -> Option<Attribute> {
        match self {
            InputAction::AllocateSpeed => Some(Attribute::Speed),
            InputAction::AllocateStrength => Some(Attribute::Strength),
            InputAction::AllocateHealth => Some(Attribute::Health),
            InputAction::AllocateAura => Some(Attribute::Aura),
            _ => None,
        }
    }

    /// Código estável para a ponte HTML/mobile. A ordem é ABI: só acrescente
    /// variantes no fim.
    pub const fn code(self) -> u8 {
        match self {
            InputAction::MoveLeft => 0,
            InputAction::MoveRight => 1,
            // ...
            InputAction::OpenSkills => 17,
        }
    }
}
```

Regras:

- `InputAction` **não** é lida pelo core; ela é traduzida para `PlayerCommand`
  pelo cliente. O core continua sem conhecer teclado.
- `OpenMenu` e `OpenSkills` são ações de UI consumidas pelo cliente; não entram
  no `PlayerCommand`.
- Mouse/touch em botões de menu e do HUD continuam produzindo
  `HudAction`/`MenuAction`, não `InputAction`.
- `InputAction::code()` é append-only: reordenar quebra a ponte JS e os replays
  de input.

### 4.2 `SlotInput` e `PlayerCommand`

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotInput {
    /// Edge: true apenas no tick em que o botão foi pressionado.
    pub pressed: bool,
    /// Estado contínuo: true enquanto o botão estiver segurado.
    pub held: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerCommand {
    pub move_x: i8, // -1, 0, 1
    pub move_y: i8, // -1, 0, 1 (voo/vertical)
    pub slots: [SlotInput; ActionSlot::COUNT],
    pub sixseven: Option<SixSevenKey>,
    pub allocate: Option<Attribute>,
    pub confirm: bool,
    pub restart: bool,
}

impl PlayerCommand {
    pub const fn none() -> Self {
        Self {
            move_x: 0,
            move_y: 0,
            slots: [SlotInput { pressed: false, held: false }; ActionSlot::COUNT],
            sixseven: None,
            allocate: None,
            confirm: false,
            restart: false,
        }
    }

    /// Mantém apenas estados contínuos. Usado nos ticks extras de um frame.
    pub fn without_edges(mut self) -> Self {
        for slot in &mut self.slots {
            slot.pressed = false;
        }
        self.sixseven = None;
        self.allocate = None;
        self.confirm = false;
        self.restart = false;
        self
    }
}
```

`PlayerInput` (no controlador) espelha a mesma intenção:

```rust
pub struct PlayerInput {
    pub move_x: i8,
    pub move_y: i8,
    pub slots: [SlotInput; ActionSlot::COUNT],
}
```

`clear_edges` zera `pressed` de todos os slots, mas preserva `held`, `move_x` e
`move_y`.

### 4.3 Semântica de edge e held

- `pressed` é **edge**: o cliente acumula a pressão entre frames e a consome no
  primeiro tick do frame, como hoje.
- `held` é **estado**: continua valendo nos ticks extras e enquanto o botão
  estiver pressionado.
- O cliente não aplica o mesmo edge duas vezes: `without_edges()` limpa apenas
  `pressed`.
- Skills que usam os dois (pulo variável) leem `pressed` para iniciar e `held`
  para sustentar. Skills de toque (dash, soco, Mogar) leem só `pressed`. Skills
  contínuas (voo) leem `held` e `move_y`.
- O core nunca recebe “tecla repetida” do navegador. Auto-repeat é filtrado no
  cliente.

### 4.4 `KeyBindings` (cliente)

As teclas físicas ficam no cliente porque `KeyCode` é detalhe de plataforma e
não deve entrar no core.

```rust
// crates/enzo-wasm/src/input/bindings.rs
pub struct KeyBindings {
    /// Uma ação pode ter várias teclas (ex.: A e ←).
    keys: [Vec<KeyCode>; InputAction::COUNT],
}

impl KeyBindings {
    pub fn default_player() -> Self;
    pub fn keys_for(&self, action: InputAction) -> &[KeyCode];
    pub fn rebind(&mut self, action: InputAction, keys: Vec<KeyCode>) -> Result<(), BindError>;
    pub fn actions_for(&self, key: KeyCode) -> impl Iterator<Item = InputAction>;
    pub fn conflicts(&self) -> Vec<BindConflict>;
}
```

- Persistência: `enzoballz.bindings.v1` no `localStorage`, JSON com nomes
  estáveis de tecla (`"Space"`, `"LeftShift"`, `"ArrowLeft"`), independente do
  `KeyCode` do macroquad.
- Bindings são preferência de dispositivo: não entram no `SaveData`.
- Conflito (mesma tecla em duas ações) é rejeitado por padrão no editor. Os
  defaults não têm conflito.
- A primeira entrega da refatoração pode usar `KeyBindings::default_player()`
  + persistência, sem tela de rebind. A tela de remapeamento é uma fase
  separada; o modelo já nasce pronto para ela.
- O HUD mostra o rótulo da tecla do slot, não o nome da skill. Se a ação tiver
  várias teclas, mostra a primeira.

### 4.5 Defaults

| Ação | Teclas padrão | Observação |
| ---- | ------------- | ---------- |
| `MoveLeft` | `A`, `←` | contínuo |
| `MoveRight` | `D`, `→` | contínuo |
| `MoveUp` | `W`, `↑` | contínuo; usado por voo |
| `MoveDown` | `S`, `↓` | contínuo; usado por voo |
| `Slot1` | `Espaço` | skill padrão: Pulo |
| `Slot2` | `Shift` esquerdo/direito | skill padrão: Dash |
| `Slot3` | `J` | skill padrão: Soco |
| `Slot4` | `K` | skill padrão: Mogar |
| `Six` | `6` | SixSeven, fora dos slots |
| `Seven` | `7` | SixSeven, fora dos slots |
| `AllocateSpeed` | `1` | só na tela de level-up |
| `AllocateStrength` | `2` | idem |
| `AllocateHealth` | `3` | idem |
| `AllocateAura` | `4` | idem |
| `Confirm` | `Enter` | level-up / modais |
| `Restart` | `R` | morte/vitória |
| `OpenMenu` | `Esc` | salva e volta ao menu |
| `OpenSkills` | `Tab` | deve chamar `preventDefault` |

`Tab` é a sugestão inicial; se o loader do macroquad não permitir impedir o
foco do navegador de forma confiável, o default pode migrar para `Q` sem mudar
o contrato.

### 4.6 Mapeamento ação → comando

O cliente faz o mapeamento em um único lugar:

```text
MoveLeft/MoveRight  -> move_x
MoveUp/MoveDown     -> move_y
Slot1..Slot4        -> slots[0..3] = SlotInput { pressed, held }
Six/Seven           -> sixseven
Allocate*           -> allocate
Confirm             -> confirm
Restart             -> restart
OpenMenu/OpenSkills -> UI do cliente (não vão para o core)
```

`move_x` e `move_y` são digitais e cancelam opositores:

```rust
pub fn axis(negative: bool, positive: bool) -> i8 {
    match (negative, positive) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}
```

Corrida por double-tap continua sendo regra do core (`Walk`), não uma
`InputAction`. O cliente só envia `move_x` contínuo.

### 4.7 Mobile e HTML

Os controles touch **não** precisam de `KeyBindings`: eles emitem
`InputAction` diretamente.

- Joystick/D-pad esquerdo: `MoveLeft`/`MoveRight`/`MoveUp`/`MoveDown` (held).
- Quatro botões de skill: `Slot1`..`Slot4` (`pressed` no `pointerdown`, `held`
  até `pointerup`/`pointercancel`).
- Botões `6`/`7`: `Six`/`Seven`.
- Botões de menu/skills: `OpenMenu`/`OpenSkills`.

A ponte com o HTML é detalhada na seção 9.6. O importante é que o HTML emite
**ações semânticas**, nunca campos de `PlayerCommand` nem regras de jogo.

## 5. Loadout

### 5.1 Invariantes

1. Existem exatamente 4 slots (`ActionSlot::COUNT`).
2. Cada slot contém `Option<SkillId>`; slot vazio é válido.
3. **A mesma `SkillId` nunca aparece em dois slots.**
4. A ordem dos slots é estável: `Slot1` → índice 0, ..., `Slot4` → índice 3.
5. O loadout é serializável, `Copy` e determinístico.
6. O loadout pertence ao `Player`; inimigos e projéteis têm loadout vazio.
7. Trocar o loadout não custa aura, XP ou tempo de jogo.
8. Perder os requisitos de uma skill **não remove** a skill do loadout; ela
   fica **bloqueada** até os requisitos voltarem.

### 5.2 Representação

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Loadout {
    slots: [Option<SkillId>; ActionSlot::COUNT],
}

impl Loadout {
    pub const EMPTY: Self = Self {
        slots: [None; ActionSlot::COUNT],
    };

    /// Loadout inicial do Enzo.
    pub const fn player_default() -> Self {
        let mut slots = [None; ActionSlot::COUNT];
        slots[ActionSlot::Slot1.index()] = Some(SkillId::Jump);
        slots[ActionSlot::Slot2.index()] = Some(SkillId::Dash);
        slots[ActionSlot::Slot3.index()] = Some(SkillId::Melee);
        slots[ActionSlot::Slot4.index()] = Some(SkillId::Mogar);
        Self { slots }
    }

    pub const fn get(self, slot: ActionSlot) -> Option<SkillId>;
    pub const fn contains(self, skill: SkillId) -> bool;
    pub const fn slot_of(self, skill: SkillId) -> Option<ActionSlot>;
    pub fn iter(self) -> impl Iterator<Item = (ActionSlot, SkillId)>;
    pub fn is_empty(self) -> bool;

    /// Atribui ou troca, mantendo a unicidade.
    pub fn assign(&mut self, slot: ActionSlot, skill: SkillId) -> LoadoutChange;
    pub fn clear(&mut self, slot: ActionSlot) -> Option<SkillId>;

    /// Remove duplicatas de um save corrompido, mantendo a primeira ocorrência
    /// na ordem dos slots.
    pub fn sanitize(&mut self) -> bool;
}
```

JSON do save:

```json
"loadout": ["jump", "dash", "melee", "mogar"]
```

Slot vazio é `null`. `SkillId` usa `snake_case` estável.

### 5.3 Operações e duplicatas

`assign(slot, skill)` tem três casos:

1. **Já está no slot**: no-op (`LoadoutChange::None`).
2. **Está em outro slot**: troca os dois slots (`Swapped`). Se o slot de destino
   estiver vazio, isso equivale a mover a skill.
3. **Não está equipada**: substitui o conteúdo do slot (`Assigned`), guardando
   a skill anterior.

```rust
pub enum LoadoutChange {
    None,
    Assigned {
        slot: ActionSlot,
        skill: SkillId,
        previous: Option<SkillId>,
    },
    Swapped {
        a: ActionSlot,
        b: ActionSlot,
    },
    Cleared {
        slot: ActionSlot,
        skill: SkillId,
    },
}
```

Trocar em vez de rejeitar é melhor UX: o jogador clica no slot 2 e na skill que
já estava no slot 1; as duas trocam de lugar, nada se perde e a unicidade é
mantida por construção.

O save é uma fonte não confiável: `sanitize()` roda na migração/`apply_to` e
remove duplicatas mantendo a primeira ocorrência. Nenhuma outra parte do jogo
precisa verificar duplicata em runtime.

### 5.4 Disponibilidade

Estar no loadout não significa estar usável. A disponibilidade é calculada no
core a partir de `SkillDef.requirements` + contexto do jogador:

```rust
pub struct SkillContext {
    pub entity_type: EntityType,
    pub level: u32,
    pub attributes: AttributeSet, // personagem + equipamento
    pub equipment: EquipmentTags,
}

pub enum SkillLock {
    EntityType,
    Level { required: u32, current: u32 },
    Attribute {
        attribute: AttributeId,
        required: u16,
        current: u16,
    },
    Equipment {
        required: EquipmentTags,
        current: EquipmentTags,
    },
}
```

Regras:

- `SkillDef::requirements.check(context) -> Result<(), SkillLock>` devolve o
  **primeiro** requisito não cumprido, em ordem fixa: tipo de entidade, nível,
  atributos, equipamento.
- Ao ativar um slot, o core **revalida** a disponibilidade. Uma skill
  bloqueada não ativa, mesmo que o cliente peça.
- Level-up e mudança de equipamento podem liberar ou bloquear skills. O
  loadout não muda sozinho; o slot fica bloqueado e o menu mostra o motivo.
- Skill concedida por item (ex.: arma de nutela) usa
  `EquipmentTags::NUTELLA_GUN`. Enquanto o item estiver equipado, a skill fica
  disponível; ao desequipar, ela continua no slot, mas bloqueada.
- O menu de skills lista **todas** as `SkillId`, com cadeado e motivo para as
  indisponíveis. Não existe “compra” ou “desbloqueio” separado neste plano;
  requisito cumprido = disponível.

### 5.5 API do core

Trocar loadout é uma operação esparsa de meta-estado, não um botão de 60 Hz.
Por isso ela **não** entra em `PlayerCommand`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadoutRequest {
    Assign { slot: ActionSlot, skill: SkillId },
    Clear { slot: ActionSlot },
}

pub enum LoadoutError {
    NoPlayer,
    EmptySlot,
    Unavailable { skill: SkillId, lock: SkillLock },
}

impl World {
    pub fn apply_loadout(
        &mut self,
        player_id: PlayerId,
        request: LoadoutRequest,
    ) -> Result<LoadoutChange, LoadoutError>;
}
```

Por que não em `PlayerCommand`:

- A troca acontece com o jogo pausado; `PlayerCommand` é o contrato do tick de
  simulação.
- É esparsa e não deve ocupar espaço no caminho quente de 60 Hz.
- É análoga a `World::equip`, que já é uma API de core e não um comando de tick.
- No multiplayer futuro, `LoadoutRequest` vira uma mensagem confiável
  (sequenciada), não um comando de movimento.

Para replays/determinismo, `enzo-sim` aplica a mesma sequência de
`LoadoutRequest` nos mesmos limites de tick. Se no futuro for necessário
lockstep, o request pode ser embrulhado em um comando com número de sequência.

`apply_loadout`:

1. valida que o jogador existe e está vivo;
2. valida `SkillDef.requirements` contra o contexto atual;
3. aplica `Loadout::assign`/`clear`;
4. emite `GameEvent::LoadoutChanged`;
5. pede save (`GameEvent::SaveRequested`).

## 6. Skills

### 6.1 `SkillId`

`SkillId` é a identidade estável de uma skill equipável. Andar e SixSeven
**não** são `SkillId`.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SkillId {
    Jump,
    Dash,
    Fly,
    Melee,
    Mogar,
    NutellaShot,
    Shout,
}

impl SkillId {
    pub const ALL: [SkillId; 7] = [
        SkillId::Jump,
        SkillId::Dash,
        SkillId::Fly,
        SkillId::Melee,
        SkillId::Mogar,
        SkillId::NutellaShot,
        SkillId::Shout,
    ];

    pub const COUNT: usize = Self::ALL.len();

    /// Índice estável em arrays (cooldowns, snapshot).
    pub const fn index(self) -> usize { /* 0..COUNT */ }

    pub const fn def(self) -> &'static SkillDef { /* match nos módulos */ }
}
```

- A ordem de `ALL` é a ordem de iteração e de cooldown; um teste garante
  `ALL[i].index() == i`.
- `SkillId` é append-only no fim para não quebrar save/snapshot. Remover ou
  reordenar exige migração.
- `Walk` e `SixSeven` continuam em `mechanics/`, não em `SkillId`.

### 6.2 `SkillDef`

Metadados estáticos, sem alocação e sem `trait object`:

```rust
pub struct SkillDef {
    pub id: SkillId,
    pub category: SkillCategory,
    pub source: SkillSource,
    pub activation: Activation,
    pub requirements: Requirements,
    pub aura_cost: f32,
    pub cooldown: f32,
    pub timing: SkillTiming,
    pub assets: SkillAssets,
}

pub enum SkillCategory {
    Movement,
    Attack,
    Utility,
}

pub enum SkillSource {
    /// Sempre existe para o tipo de entidade (ex.: pulo).
    Innate,
    /// Liberada por nível/atributos (ex.: voo).
    Progression,
    /// Só existe enquanto um equipamento estiver presente (ex.: arma de nutela).
    Equipment,
}

pub enum Activation {
    /// Só o edge `pressed`.
    Press,
    /// Só o estado `held`.
    Hold,
    /// `pressed` inicia, `held` sustenta (pulo variável).
    PressHold,
}

pub enum SkillTiming {
    /// Antes da física: define velocidade/intenção (pulo, dash, voo).
    PrePhysics,
    /// Depois da física: hitbox/projétil na posição atual (soco, Mogar).
    PostPhysics,
}

pub struct SkillAssets {
    /// Obrigatório para toda SkillId.
    pub icon: &'static str,
    /// Entidade gerada pela skill (projétil, onda etc.).
    pub entity: Option<&'static str>,
    /// Partículas/efeito da skill.
    pub particles: Option<&'static str>,
}
```

Exemplo de módulo:

```rust
// crates/enzo-core/src/skills/mogar.rs
pub const COST: f32 = 25.0;
pub const COOLDOWN: f32 = 0.35;

pub const DEF: SkillDef = SkillDef {
    id: SkillId::Mogar,
    category: SkillCategory::Attack,
    source: SkillSource::Innate,
    activation: Activation::Press,
    requirements: Requirements::new(
        &[EntityType::Playable],
        1,
        AttributeSet::ZERO,
        EquipmentTags::NONE,
    ),
    aura_cost: COST,
    cooldown: COOLDOWN,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/mogar/icon.png",
        entity: Some("assets/skills/mogar/projectile.png"),
        particles: Some("assets/skills/mogar/impact.png"),
    },
};
```

Regras de módulo:

- Um arquivo por skill (`skills/mogar.rs`); se crescer, vira diretório
  (`skills/mogar/mod.rs`, `state.rs`, ...).
- O módulo declara `DEF`, o estado tipado, funções puras e testes.
- O módulo não importa `macroquad`, `wasm-bindgen`, arquivos, relógio ou rede.
- Entidades/partículas são criadas via `EntitySpec` + `EntityStore::spawn`.
- O módulo não formata texto de UI; o cliente traduz `SkillId`/`SkillLock`.
- O módulo declara os próprios assets; não existe lista paralela.

### 6.3 Catálogo inicial

| Skill | Categoria | Ativação | Requisitos | Custo | Cooldown | Fase | Status |
| ----- | --------- | -------- | ---------- | ----- | -------- | ---- | ------ |
| Pulo (`Jump`) | Movimento | PressHold | jogável, nível 1 | 0 | 0 | Pré-física | migrar |
| Dash (`Dash`) | Movimento | Press | jogável, nível 1 | 0 | 0,70 s | Pré-física | migrar |
| Voo (`Fly`) | Movimento | Hold | jogável, nível 5, Aura 5 | 0 | 0 | Pré-física | migrar |
| Soco (`Melee`) | Ataque | Press | jogável, nível 1 | 0 | janela de combo | Pós-física | migrar |
| Mogar (`Mogar`) | Ataque | Press | jogável, nível 1 | 25 | 0,35 s | Pós-física | migrar |
| Tiro de Nutela (`NutellaShot`) | Ataque | Press | jogável, nível 1, `NUTELLA_GUN` | 0 | 0,25 s | Pós-física | primeiro novo |
| Grito (`Shout`) | Utilidade | Press | jogável, nível 1 | 10 | 2,0 s | Pós-física | primeiro novo |

Observações:

- Os números de Nutella/Grito são placeholders; o balanceamento vem depois.
- `Melee` é o “soco” atual (combo de 3). O nome interno continua `Melee`; a UI
  pode chamar de “Soco”.
- `Fly` usa `move_y`; não precisa de ação própria além do slot.
- `SixSeven` não aparece na tabela de slots; continua em `mechanics/sixseven.rs`.
- `NutellaShot` é a prova de conceito de skill concedida por item:
  `EquipmentTags::NUTELLA_GUN` + `ItemKind::NutellaGun` +
  `assets/skills/nutella_shot/`.
- `Shout` é a prova de conceito de skill de utilidade (onda de choque,
  empurrão, buff etc.) sem entidade própria obrigatória.

### 6.4 Estado runtime

`Skills` unifica o antigo `Movement` (parte de skill) e `Combat`. `Locomotion`
guarda só o que não é slot.

```rust
pub struct Skills {
    pub loadout: Loadout,
    /// Cooldown por SkillId, indexado por `SkillId::index()`.
    pub cooldowns: [f32; SkillId::COUNT],

    pub jump: jump::State,
    pub dash: dash::State,
    pub fly: fly::State,
    pub melee: melee::State,
    pub mogar: mogar::State,
    pub nutella_shot: nutella_shot::State,
    pub shout: shout::State,
}

pub struct Locomotion {
    pub walk: walk::State,
    pub sixseven: sixseven::State,
}
```

- `Entity` passa a ter `pub locomotion: Locomotion` e `pub skills: Skills`;
  `movement`/`combat` deixam de existir.
- O estado de todas as skills existe sempre (sem alocação dinâmica), mas só as
  equipadas respondem a input.
- Cooldowns ficam no array central para não poderem ser burlados trocando de
  slot. Uma skill desequipada e reequipada continua em cooldown.
- Timers internos (janela do combo, buffer de pulo, dash ativo) ficam no estado
  do módulo.
- `Skills::tick(dt)` avança cooldowns e timers uma vez por tick.
- `Skills::available(skill, context)` reexporta a checagem de requisitos.
- `Skills::slot_skill(slot)` lê o loadout.
- `Skills::is_active(skill)` é usado pelo snapshot/HUD.

### 6.5 Ativação, aura e cooldown

Fluxo central para skills `PostPhysics`:

```text
para cada jogador:
  para cada slot:
    se slot vazio: continua
    se skill bloqueada: emite SkillRejected(Unavailable); continua
    se cooldown > 0: emite SkillRejected(Cooldown); continua
    se aura < def.aura_cost: emite SkillRejected(NoAura); continua
    se input não dispara a ativação: continua
    executa skill.activate(...)
    se Started:
      aura -= def.aura_cost
      cooldowns[skill] = def.cooldown
      emite SkillActivated
```

- A aura só é gasta quando a skill **começa** de fato. Skill que falha por
  condição interna devolve `Rejected` e não cobra.
- O cooldown é preenchido na ativação e `Skills::tick` roda depois, então no
  tick da ativação ele permanece cheio e só começa a cair no tick seguinte. Isso
  é determinístico e fácil de testar.
- Skills `PrePhysics` têm fluxo próprio: pulo bufferiza, dash consulta e
  preenche o cooldown central, voo é contínuo. O fluxo genérico de aura/cooldown
  acima vale para skills `PostPhysics`.
- Custo de aura é **one-shot** por ativação neste plano. Drenagem contínua
  (ex.: voo consumindo aura por segundo) é uma extensão futura do `SkillDef`
  (`aura_drain`), não um requisito agora.

### 6.6 Ordem dos sistemas em `World::step`

A ordem é contrato. A refatoração troca as etapas 2–6 atuais por um pipeline
de skills, mantendo o resto:

1. **input**: aplica `PlayerCommand` aos controladores (`move_x`, `move_y`,
   slots, SixSeven).
2. **sixseven**: consome o pedido de `6`/`7`, atualiza aura/combo/lentidão.
3. **skills::apply_intent** (pré-física): andar, buffer de pulo, pedido de
   dash, intenção vertical de voo.
4. **physics**: integra o corpo e resolve colisão com tiles. Dash sobrescreve
   a velocidade horizontal; voo sobrescreve a gravidade; SixSeven aplica a
   lentidão.
5. **skills::activate** (pós-física): soco, Mogar, tiro de nutela, grito e
   qualquer skill `PostPhysics`. Hitboxes/projéteis usam a posição já
   integrada.
6. **projectiles**: move projéteis e resolve tile/inimigos.
7. **ai**: patrulha e intenção de inimigos.
8. **collisions**: dano de contato, i-frames e knockback.
9. **skills::tick**: cooldowns, janelas e timers.
10. **progression**: XP, level-up, pontos.
11. **cleanup**: remove mortos e projéteis expirados.
12. **events**: devolve a lista de eventos do tick.

Mudar essa ordem muda replay/rede e exige atualizar este documento.

## 7. Assets por skill

### 7.1 Layout

```text
assets/
├── sprites/                  # atlas atuais (transição)
└── skills/
    ├── jump/
    │   ├── icon.png          # obrigatório
    │   └── dust.png          # opcional (partículas)
    ├── dash/
    │   ├── icon.png
    │   └── trail.png
    ├── fly/
    │   ├── icon.png
    │   └── aura.png
    ├── melee/
    │   ├── icon.png
    │   └── impact.png
    ├── mogar/
    │   ├── icon.png
    │   ├── projectile.png    # entidade gerada
    │   └── impact.png
    ├── nutella_shot/
    │   ├── icon.png
    │   ├── projectile.png
    │   └── splash.png
    └── shout/
        ├── icon.png
        └── wave.png
```

- `<id>` é o `SkillId` em `snake_case` estável (`nutella_shot`).
- `icon.png` é obrigatório para **toda** `SkillId`.
- Ações de movimento (`MoveLeft`/`MoveRight`/`MoveUp`/`MoveDown`) usam setas
  padrão da UI de controle, não ícones de skill.
- `entity`/`particles` são opcionais e declarados no `SkillDef`.
- Ícones: PNG RGBA, fundo transparente, pixel-art, 32×32 (ou 24×24 se o atlas
  atual exigir). A escala no HUD é inteira para não borrar.
- Cada ícone deve ser legível em 32 px e distinto dos demais:
  - Pulo: seta para cima + poeira.
  - Dash: linhas de velocidade horizontais.
  - Voo: asas/aura vertical.
  - Soco: punho + estrela de impacto.
  - Mogar: orbe/feixe de energia.
  - Tiro de Nutela: pote/arma com projétil marrom.
  - Grito: ondas concêntricas saindo da boca.
- Nada copiado de obras existentes; o gerador versionado é a fonte.

### 7.2 Contrato `SkillAssets`

```rust
pub struct SkillAssets {
    pub icon: &'static str,
    pub entity: Option<&'static str>,
    pub particles: Option<&'static str>,
}
```

Cada módulo declara seus caminhos completos:

```rust
pub const ASSETS: SkillAssets = SkillAssets {
    icon: "assets/skills/mogar/icon.png",
    entity: Some("assets/skills/mogar/projectile.png"),
    particles: Some("assets/skills/mogar/impact.png"),
};
```

- O caminho é relativo à raiz servida (`dist/`), então funciona na raiz e em
  subdiretório no GitHub Pages.
- O core não lê arquivo; só carrega as strings.
- O cliente HTML usa o caminho direto em `<img src>`.
- O cliente canvas pré-carrega os ícones/entidades via `load_texture` no
  início; se falhar, usa um placeholder colorido com a inicial da skill. Nunca
  entra em pânico por asset ausente.

### 7.3 Pipeline

- `scripts/gen-sprites.mjs` gera os PNGs de `assets/skills/<id>/` de forma
  determinística.
- `scripts/build-wasm.sh` já copia `assets/` para `dist/assets/`; nenhum passo
  novo de cópia é necessário.
- Um teste de host (em `enzo-wasm/tests/` ou script npm) verifica que toda
  `SkillId::ALL` tem `icon` não vazio e arquivo existente. O core puro não faz
  IO; esse teste não entra no `enzo-core`.
- O `SkillDef` é a única fonte de verdade do catálogo; não criar um
  `manifest.json` paralelo. A UI HTML recebe o catálogo pelo `UiState`.

## 8. Snapshot, save e rede

### 8.1 Snapshot (`SNAPSHOT_VERSION = 3`)

```rust
pub struct PlayerSnapshot {
    // ... level, xp, points, attributes, equipment, inventário ...
    pub loadout: Loadout,
    pub slots: [SkillSlotSnapshot; ActionSlot::COUNT],
    /// Todas as skills e a disponibilidade para o menu.
    pub catalog: Vec<SkillCatalogEntry>,
    pub farming: bool,
    pub combo_window: f32,
    pub melee_step: u8,
    pub melee_timer: f32,
}

pub struct SkillSlotSnapshot {
    pub skill: Option<SkillId>,
    pub available: bool,
    pub lock: Option<SkillLock>,
    pub cooldown: f32,
    pub cooldown_max: f32,
    pub aura_cost: f32,
    pub active: bool,
}

pub struct SkillCatalogEntry {
    pub skill: SkillId,
    pub available: bool,
    pub lock: Option<SkillLock>,
    pub equipped_slot: Option<ActionSlot>,
    pub aura_cost: f32,
    pub cooldown_max: f32,
}
```

- `loadout`, `slots` e `catalog` são somente leitura no cliente.
- `catalog` é por jogador porque disponibilidade depende de nível, atributos e
  equipamento.
- `lock` é `None` quando disponível; caso contrário, traz o motivo para a UI.
- `cooldown`/`cooldown_max` alimentam o overlay radial e o texto.
- `active` cobre skills de `Hold` (ex.: voo).
- `melee_step`/`melee_timer` continuam como estado visual específico do soco;
  quando existir um segundo golpe corpo a corpo, generalizar para estado por
  slot.
- `moves: Vec<MoveId>` e `mogar_cooldown` saem; o HUD passa a ler os slots.
- `SNAPSHOT_VERSION` sobe para 3. Snapshots não são persistidos, então não há
  migração; todos os consumidores devem compilar contra a versão nova.

### 8.2 Save (`SAVE_VERSION = 2`)

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

- `loadout` é `#[serde(default)]` para o JSON v1 (sem o campo) continuar
  desserializável.
- Ao ler `version == 1`, o cliente/core de migração preenche
  `Loadout::player_default()`, ajusta `version = 2` e grava no slot v2.
- Ao ler `version == 2`, o loadout é preservado exatamente como está (inclusive
  todo vazio, se o jogador limpou os 4 slots).
- `Loadout::sanitize()` roda após o parse para remover duplicatas de save
  corrompido.
- `SaveData::apply_to` só aceita `version == SAVE_VERSION`; a migração acontece
  antes.

Chaves:

| Chave | Uso |
| ----- | --- |
| `enzoballz.save.slot.{0..2}.v2` | save novo |
| `enzoballz.save.slot.{0..2}.v1` | fallback legado, migrado para v2 |
| `enzoballz.save.v1` | fallback do slot 0, migrado para v2 |
| `enzoballz.bindings.v1` | teclas, separado do save |
| `enzoballz.language` | idioma |
| `enzoballz.interface_scale` | escala da UI |

- O v1 não é apagado automaticamente; ele é lido e o v2 é gravado. A remoção
  do v1 pode ser uma limpeza manual futura.
- Equipamento ainda não é salvo no MVP. Uma skill concedida por item fica no
  loadout, mas **bloqueada** após carregar até o item ser equipado de novo.
  Isso é uma decisão em aberto (seção 12).

### 8.3 Eventos

Eventos novos/alterados:

```rust
pub enum GameEvent {
    // ...
    SkillActivated {
        player: EntityId,
        slot: ActionSlot,
        skill: SkillId,
    },
    SkillRejected {
        player: EntityId,
        slot: ActionSlot,
        skill: SkillId,
        reason: SkillRejectReason,
    },
    SkillProjectileSpawned {
        owner: EntityId,
        skill: SkillId,
        projectile: EntityId,
    },
    SkillHit {
        attacker: EntityId,
        target: EntityId,
        skill: SkillId,
        damage: f32,
    },
    LoadoutChanged {
        player: EntityId,
        change: LoadoutChange,
    },
    // ...
}

pub enum SkillRejectReason {
    Unavailable(SkillLock),
    Cooldown,
    NoAura,
    EmptySlot,
}
```

- O cliente mapeia `SkillId` para efeitos/partículas e para o toast.
- `MogarNoAura`/`MogarFired`/`MogarHit` podem ser substituídos pelos eventos
  genéricos; se a migração for incremental, os antigos ficam como alias
  temporário e saem no mesmo commit da limpeza.
- `SaveRequested` continua existindo e é emitido após `LoadoutChanged`.

### 8.4 Determinismo e replays

- `PlayerCommand`, `SlotInput`, `Loadout`, `SkillId` e `LoadoutRequest` são
  `Copy`, `Eq` e determinísticos.
- Nada de `HashMap`/`HashSet` na lógica; o loadout é array, o catálogo é
  `Vec`/array e a iteração segue `SkillId::ALL`.
- Replays de `TickInput` continuam válidos. Replays que mudam loadout precisam
  registrar `(tick, player, LoadoutRequest)` e aplicar antes do tick
  correspondente. O `enzo-sim` ganha esse campo no formato de replay.
- O teste de determinismo compara o hash do `WorldSnapshot` depois de:
  mesma semente + mesma sequência de `TickInput` + mesmos `LoadoutRequest`.

## 9. UI e UX

### 9.1 Barra de ações (HUD)

Substitui o indicador fixo de Mogar por uma barra com os 4 slots:

```text
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│ [1]  │ │ [2]  │ │ [3]  │ │ [4]  │
│  ⤒   │ │  »   │ │  ✊  │ │  ◉   │
│ Pulo │ │ Dash │ │ Soco │ │Mogar │
└──────┘ └──────┘ └──────┘ └──────┘
```

- Ordem fixa `Slot1` → `Slot4`, da esquerda para a direita.
- Cada slot mostra: ícone, rótulo da tecla, nome curto, custo de aura e
  cooldown.
- Estados visuais:
  - **pronto**: ícone colorido;
  - **cooldown**: overlay escuro + segundos restantes;
  - **aura insuficiente**: borda/ícone apagado;
  - **bloqueado**: cadeado + motivo no tooltip;
  - **vazio**: `+` e “Vazio”;
  - **ativo** (`Hold`): brilho/pulso.
- O slot vazio não ocupa tecla “morta” silenciosamente: o rótulo mostra a
  tecla e “Vazio”.
- A barra não cobre os controles mobile; no modo HTML/touch, ela pode ser
  desenhada pelo HTML e o canvas esconde a versão de teclado. O core não sabe
  qual modo está ativo.

### 9.2 Menu de skills

Overlay modal aberto por `OpenSkills` (ou pelo dot “Habilidades” do HUD):

```text
┌──────────────────────────────────────────────────────────────┐
│ Habilidades                                            [×]  │
├────────────────────────────┬─────────────────────────────────┤
│ SLOTS EQUIPADOS            │ HABILIDADES                     │
│                            │                                 │
│ [1] Espaço   [⤒] Pulo      │ MOVIMENTO                       │
│ [2] Shift    [»] Dash      │ [⤒] Pulo      [»] Dash          │
│ [3] J        [✊] Soco     │ [✈] Voo 🔒 Nível 5 / Aura 5     │
│ [4] K        [◉] Mogar     │ ATAQUE                          │
│                            │ [✊] Soco     [◉] Mogar         │
│ [Limpar slot]              │ [🥜] Tiro de Nutela 🔒 Arma      │
│                            │ UTILIDADE                       │
│                            │ [📣] Grito                      │
│                            │                                 │
│ Clique num slot e depois numa habilidade.                   │
└──────────────────────────────────────────────────────────────┘
```

Fluxo:

1. Jogador abre o menu; jogo pausa.
2. Clica no slot desejado (1–4). O slot selecionado ganha destaque.
3. Clica na skill desejada no catálogo.
4. O core valida disponibilidade e aplica `LoadoutRequest::Assign`.
5. A UI atualiza os ícones, mostra toast e salva.
6. Se a skill já estava em outro slot, os dois slots **trocam**; a UI avisa.
7. `Limpar slot` remove a skill; o slot fica vazio.
8. Fechar (`Esc`, `×` ou “Voltar”) retoma a simulação.

Detalhes:

- Skill indisponível fica com cadeado, opacidade reduzida e motivo:
  “Nível 5”, “Aura 5”, “Requer Arma de Nutela”.
- Skill já equipada mostra um selo “Slot 3”; clicar nela com outro slot
  selecionado troca/move.
- O catálogo é agrupado por `SkillCategory` (Movimento/Ataque/Utilidade), mas
  qualquer skill pode ir em qualquer slot.
- O menu é somente leitura sobre o snapshot; ele não calcula requisitos nem
  duplicatas.
- O estado de UI (`selected_slot`, `open`, scroll) vive no cliente, não no
  core/snapshot.
- O menu não abre durante `Dead`, `Victory` ou `LevelUp` (nessas fases há
  overlays próprios). Abrir durante `LevelUp` é uma decisão em aberto.

### 9.3 Pausa

Proposta padrão: **pausar a simulação** enquanto o menu de skills está aberto,
reutilizando o `paused` do `GameSession` (o mesmo usado pelo menu de Escape).

- O mundo para de avançar; o último snapshot continua renderizado atrás do
  overlay.
- A troca de loadout é aplicada via `World::apply_loadout` (API de core) sem
  precisar de tick.
- Ao fechar, o cliente atualiza o snapshot (`world.write_snapshot`) e retoma.
- Em multiplayer futuro, menus não pausam o servidor; cada cliente pode abrir
  o próprio menu, e a troca é uma mensagem validada pelo servidor. O
  `LoadoutRequest` já foi desenhado para isso.
- Trocar em tempo real (sem pausa) foi descartado no MVP: o jogador ficaria
  vulnerável e sujeito a troca acidental.

### 9.4 Feedback e erros

- Sucesso: toast “Mogar movido para o Slot 1”, “Soco trocado com Mogar”,
  “Slot 3 limpo”.
- Falha: toast curto com o motivo (`Nível insuficiente`, `Aura insuficiente`,
  `Em cooldown`) e vibração/realce no slot quando a tecla é apertada.
- O HUD também reage ao `SkillRejected` do core; a UI não tenta adivinhar.
- Nenhuma ação de UI altera aura, XP, vida ou posição.

### 9.5 Acessibilidade e i18n

- Ícones sempre acompanhados de nome textual no menu e no tooltip da barra.
- Cadeado + texto, não só cor.
- Alvos de toque ≥ 44 px no mobile; contraste alto; foco visível no teclado.
- Novas chaves de `Text` (cliente):
  - `SkillJump`, `SkillDash`, `SkillFly`, `SkillMelee`, `SkillMogar`,
    `SkillNutellaShot`, `SkillShout`;
  - `SkillCategoryMovement`, `SkillCategoryAttack`, `SkillCategoryUtility`;
  - `SkillSlotLabel`, `SkillSlotEmpty`, `SkillClearSlot`;
  - `SkillLockedLevel`, `SkillLockedAttribute`, `SkillLockedEquipment`;
  - `SkillMenuHint`, `SkillEquippedInSlot`, `SkillMoved`, `SkillSwapped`,
    `SkillCleared`, `SkillUnavailable`;
  - `ActionBarHint`, `OpenSkillsHint`.
- `Walk`/`SixSeven` deixam de usar `MoveWalk`/`MoveSixSeven`; viram
  `MechanicWalk`/`MechanicSixSeven` ou permanecem como estão se o custo de
  renomear não compensar. O importante é não chamar mecânica de skill.
- O core expõe `SkillId`/`SkillLock`; o cliente escolhe o texto.

### 9.6 Ponte HTML/mobile

O plano de UI HTML (canvas como fundo, HUD/menus/controles em DOM) usa a mesma
abstração:

- HTML/CSS desenha os controles e o menu; o canvas desenha o mundo.
- O HTML emite `InputAction` (código estável) para o wasm:
  - `enzo_action_down(code)`
  - `enzo_action_up(code)`
- O menu de skills em HTML emite `LoadoutRequest`:
  - `enzo_loadout_assign(slot, skill)`
  - `enzo_loadout_clear(slot)`
- O wasm → HTML envia `UiState` em JSON (10–20 Hz ou quando muda):
  - vida/aura/XP/nível/pontos;
  - loadout e status dos 4 slots;
  - catálogo com `SkillId`, nome localizado, ícone, custo, cooldown, cadeado;
  - fase e toasts.
- O HTML **não** lê/grava save, não calcula requisitos e não altera aura. Ele
  só renderiza estado e emite intenções.
- A ponte usa `miniquad_add_plugin` (`importObject.env`) e exports
  `#[no_mangle] extern "C"`, sem `wasm-bindgen`/`web-sys`.
- Códigos de `InputAction`/`SkillId`/`ActionSlot` são append-only; a ponte é
  versionada junto com o bundle.
- Cuidados de mobile:
  - `touch-action: none`, `user-select: none`, `-webkit-tap-highlight-color`;
  - `pointerdown`/`pointerup`/`pointercancel` + `setPointerCapture`;
  - `tabindex="-1"` nos botões e `preventDefault` para não roubar foco do
    canvas;
  - `100dvh` e `env(safe-area-inset-*)`;
  - landscape preferido; aviso de girar em portrait;
  - um pointer id por botão para multitouch;
  - esconder a barra de ações do canvas quando o HTML estiver no controle.
- Fases sugeridas da UI HTML:
  1. controles mobile (maior ganho, menor risco);
  2. barra de ações e dots;
  3. overlays simples (toast, level-up, morte/vitória);
  4. menus e modal de delete;
  5. menu de skills completo;
  6. janelas arrastáveis.

## 10. Estrutura de módulos alvo

```text
crates/enzo-core/src/
├── input.rs                 # InputAction, ActionSlot, SlotInput, PlayerCommand
├── mechanics/               # o que NÃO é slot
│   ├── mod.rs               # Locomotion
│   ├── walk.rs              # andar/correr/double-tap
│   └── sixseven.rs          # farm de aura 6/7
├── skills/
│   ├── mod.rs               # SkillId, SkillDef, Skills, dispatch
│   ├── registry.rs          # SkillId::ALL, def(), catálogo
│   ├── requirements.rs      # Requirements, SkillContext, SkillLock, EntityType
│   ├── loadout.rs           # Loadout, LoadoutChange, LoadoutRequest
│   ├── jump.rs
│   ├── dash.rs
│   ├── fly.rs
│   ├── melee.rs
│   ├── mogar.rs
│   ├── nutella_shot.rs
│   └── shout.rs
├── entities/
│   ├── entity.rs            # Entity { locomotion, skills, ... }
│   ├── spec.rs
│   └── ...
├── world.rs                 # orquestra o pipeline da seção 6.6
├── snapshot.rs              # v3
├── progression.rs           # SaveData v2
└── ...
```

O que sai:

- `moves.rs` (`MoveId`, `MoveSet`, `MoveCategory`, `AbilityContext`);
- `attacks/` (melee/mogar viram `skills/melee.rs` e `skills/mogar.rs`);
- `movements/jump.rs`, `dash.rs`, `fly.rs` (viram skills);
- `Movement` e `Combat` (substituídos por `Locomotion` + `Skills`);
- `moves`/`mogar_cooldown` do snapshot.

O que fica:

- `mechanics/walk.rs` e `mechanics/sixseven.rs` (não são slots);
- `physics`, `aura`, `ai`, `progression`, `map`, `entities`, `world`, `events`;
- `Requirements`/`EntityType` movidos para `skills/requirements.rs` (ou
  `entities` para uso compartilhado com itens).

Cliente:

```text
crates/enzo-wasm/src/
├── input/
│   ├── mod.rs               # InputState combinando teclado + virtual
│   ├── bindings.rs          # KeyBindings, defaults, persistência
│   └── virtual_input.rs     # ações vindas do HTML/mobile
├── hud/
│   ├── action_bar.rs        # 4 slots
│   ├── skills.rs            # editor de loadout
│   └── ...
├── ui_bridge/               # plugin miniquad + exports (fase HTML)
└── ...
```

## 11. Plano de migração incremental

Cada commit deve passar `cargo fmt`, `cargo clippy` e `cargo test`. O diff é
grande, então a ordem abaixo evita misturar refatoração estrutural com feature.

| # | Commit sugerido | Conteúdo | Teste no mesmo commit |
| - | --------------- | -------- | --------------------- |
| 1 | `docs: plan action slots and modular skills` | este documento + atualizações em ARCHITECTURE/SPEC/AGENTS/README | links e build de docs |
| 2 | `refactor(core): add action slots and slot input` | `ActionSlot`, `SlotInput`, novo `PlayerCommand`, `without_edges` | edge/held, slots |
| 3 | `refactor(core): introduce SkillId and SkillDef registry` | renomeia `MoveId` → `SkillId`, `SkillDef`/`Requirements`, catálogo | requisitos, unicidade do catálogo |
| 4 | `refactor(core): replace Movement/Combat with Skills/Locomotion` | move módulos, `Skills`, `Locomotion`, `Entity`, `EntitySpec`, dispatch | pulo/dash/voo/soco/Mogar |
| 5 | `feat(core): add validated skill loadout` | `Loadout`, `LoadoutRequest`, `World::apply_loadout`, eventos | swap/move/clear/sanitize |
| 6 | `feat(core): persist loadout and expose skill catalog` | `SaveData` v2 + migração; snapshot v3 | save v1→v2, snapshot slots |
| 7 | `feat(wasm): map keys to InputAction` | `KeyBindings`, defaults, persistência; sem tela de rebind | teclas default, conflitos |
| 8 | `feat(wasm): add action bar and skill menu` | ícones, editor de loadout, pausa | ação de UI → request |
| 9 | `feat(wasm): bridge HTML actions and loadout` | `UiState`, exports, virtual input | códigos ABI, roundtrip |
| 10 | `feat(assets): generate skill icons and effects` | `gen-sprites.mjs`, assets, teste de existência | ícone para toda SkillId |
| 11 | `test(core): cover slot swaps and requirement changes` | cenários de level-up/item/cooldown | determinismo |

Observações:

- O commit 4 é o mais arriscado; se necessário, divida em “adiciona `Skills`
  ao lado” e “remove `Movement`/`Combat`”.
- Não misture renomeação mecânica com mudança de comportamento no mesmo
  commit.
- O commit 7 pode manter a UI de rebind para depois; o modelo e os defaults já
  entram.
- O commit 10 pode vir antes do 8 se os ícones forem necessários para o HUD;
  nesse caso, o menu usa placeholders até o 10.

## 12. Riscos e decisões em aberto

| Risco/decisão | Situação | Mitigação |
| ------------- | -------- | --------- |
| `W`/`↑` deixa de ser pulo | Decidido | Documentado; preset legacy pode ser oferecido no rebind |
| Pausa no menu de skills | Proposto | Pausar como o menu de Escape; validar com o usuário |
| Skill bloqueada após perder item | Decidido | Continua no loadout, com cadeado; reequipar reativa |
| Equipamento não é salvo | Em aberto | Salvar equipamento no futuro ou aceitar skill bloqueada após load |
| Duplicata em save corrompido | Decidido | `sanitize()` mantém a primeira ocorrência |
| Cooldown ao trocar de slot | Decidido | Cooldown é por skill, não por slot |
| Custo de aura em skill que falha | Decidido | Só cobra quando `Started` |
| Voo com drenagem contínua | Em aberto | `aura_drain` futuro; agora custo one-shot |
| Rebinding de teclas | Planejado | Modelo agora; tela depois |
| Conflito de teclas | Decidido | Editor rejeita; defaults sem conflito |
| `Tab` vs foco do navegador | Em aberto | `preventDefault`; fallback `Q` |
| Menu aberto durante `LevelUp` | Em aberto | Proposta: desabilitar |
| Skill de utilidade (Grito) | Exemplo | Definir efeito antes de implementar |
| `SkillId` desconhecida em save futuro | Em aberto | Versão + migração; não usar fallback silencioso sem aviso |
| Snapshot com catálogo por jogador | Custo baixo | ~7–12 entradas por jogador; medir se crescer |
| UI HTML duplicar regra | Risco | HTML só emite ações e renderiza `UiState` |
| Assets não carregarem no canvas | Risco | Placeholder + log; teste de existência no host |

## 13. Critérios de aceite da refatoração

- [ ] Existem 4 slots genéricos; qualquer `SkillId` disponível pode ocupar
      qualquer slot.
- [ ] `PlayerCommand` não tem mais `jump`, `jump_held`, `dash`, `melee` ou
      `mogar`; tem `move_x`, `move_y`, `slots[4]`, `sixseven`, `allocate`,
      `confirm`, `restart`.
- [ ] `InputAction` é a única camada que conhece teclas; o core não lê teclado.
- [ ] O loadout é validado no core, persiste no save v2 e aparece no snapshot
      v3.
- [ ] É impossível ter a mesma skill em dois slots, inclusive após load de save
      corrompido.
- [ ] Trocar uma skill já equipada faz swap, não duplica e não perde a outra.
- [ ] SLOT 1–4 podem ser rebindados sem tocar no core ou no HUD.
- [ ] Skill bloqueada por nível/atributo/item mostra o motivo e não ativa.
- [ ] Toda `SkillId` tem ícone em `assets/skills/<id>/icon.png`; skills que
      geram entidade/partículas declaram os caminhos.
- [ ] Mogar, Soco, Pulo e Dash têm ícone próprio.
- [ ] SixSeven continua funcionando em `6`/`7`, fora dos slots.
- [ ] A ordem de `World::step` está documentada e coberta por teste de
      determinismo.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --
      -D warnings` e `cargo test --workspace` passam.
- [ ] `docs/ARCHITECTURE.md`, `docs/SPEC.md` e `AGENTS.md` apontam para este
      documento e não contradizem o estado atual.
