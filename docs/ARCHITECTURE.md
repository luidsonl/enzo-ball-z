# EnzoBallZ — Arquitetura e Harness

> Documento de arquitetura do MVP. Define as fronteiras do código, o contrato de
> simulação, o pipeline de build WASM e o harness de testes. O objetivo é que o
> `enzo-core` rode sem alterações no navegador (cliente), em testes headless e,
> no futuro, em um servidor autoritativo.

## 1. Princípios

1. **O core é a única fonte de verdade.** Regras de jogo, física, mapa, combate,
   aura e progressão vivem em `enzo-core`. O cliente desenha e coleta input; ele
   não decide regras.
2. **Core puro.** `enzo-core` não depende de `macroquad`, `wasm-bindgen`,
   `web-sys`, arquivos, relógio do sistema ou aleatoriedade global.
3. **Tick fixo.** A simulação avança em passos de `1/60 s`. O render pode rodar
   em qualquer taxa e interpolar entre ticks.
4. **Comandos como dados.** Cada jogador produz um `PlayerCommand` por tick. A
   origem pode ser teclado local, IA ou rede, sem mudar o core.
5. **Servidor autoritativo (futuro).** O multiplayer será por snapshots do
   servidor, não lockstep. O cliente envia comandos e recebe estado; predição e
   reconciliação entram depois. Isso evita exigir bit-exatidão entre arquiteturas.
6. **Sem assets de terceiros.** Sprites são originais, gerados por script.

## 2. Workspace

```text
enzoballz/
├── Cargo.toml                 # workspace
├── rust-toolchain.toml        # versão do Rust + target wasm32
├── package.json               # scripts de build/servidor (Node)
├── index.html                 # entrada do cliente web
├── crates/
│   ├── enzo-core/             # simulação pura, sem render/IO
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── math.rs
│   │   │   ├── rng.rs
│   │   │   ├── input.rs
│   │   │   ├── map.rs
│   │   │   ├── entities/         # Entity + componentes + entidades concretas
│   │   │   │   ├── mod.rs
│   │   │   │   ├── entity.rs     # Entity, EntityKind, EntityId
│   │   │   │   ├── controller.rs # Controller, PlayerController, PlayerInput
│   │   │   │   ├── spec.rs       # EntitySpec + builders genéricos
│   │   │   │   ├── store.rs      # EntityStore
│   │   │   │   ├── enzo.rs       # entidade jogável do MVP
│   │   │   │   ├── slime.rs      # inimigo do MVP
│   │   │   │   ├── mogar.rs      # projétil do especial
│   │   │   │   ├── player.rs     # PlayerData (progressão/aura)
│   │   │   │   ├── enemy.rs      # EnemyData
│   │   │   │   ├── projectile.rs # ProjectileData
│   │   │   │   └── item/         # item.rs + um arquivo por item
│   │   │   ├── moves.rs          # MoveId, MoveSet, Requirements, AbilityContext
│   │   │   ├── movements/        # movimentos inerentes
│   │   │   │   ├── mod.rs        # Movement + orquestração
│   │   │   │   ├── walk.rs
│   │   │   │   ├── jump.rs
│   │   │   │   ├── dash.rs
│   │   │   │   ├── fly.rs
│   │   │   │   └── sixseven.rs   # farming de aura (6/7)
│   │   │   ├── physics.rs
│   │   │   ├── aura.rs           # saldo e gasto de aura
│   │   │   ├── attacks/          # movimentos de ataque
│   │   │   │   ├── mod.rs        # Combat + dano de contato
│   │   │   │   ├── melee.rs
│   │   │   │   └── mogar.rs
│   │   │   ├── ai/               # comportamentos de IA
│   │   │   │   ├── mod.rs        # Ai + AiBehavior
│   │   │   │   └── slime.rs
│   │   │   ├── progression.rs
│   │   │   ├── world.rs
│   │   │   ├── snapshot.rs
│   │   │   └── events.rs
│   │   └── tests/             # testes de cenário e determinismo
│   ├── enzo-wasm/             # cliente: macroquad + wasm-bindgen
│   │   ├── src/
│   │   │   ├── lib.rs         # start()
│   │   │   ├── app.rs         # loop + acumulador de tick fixo
│   │   │   ├── input.rs       # teclado -> PlayerCommand
│   │   │   ├── render.rs      # desenha a partir do snapshot
│   │   │   ├── tiles.rs       # malhas de tiles em chunks sob demanda
│   │   │   ├── camera.rs
│   │   │   ├── hud.rs
│   │   │   ├── sprites.rs
│   │   │   └── save.rs        # localStorage
│   │   └── assets/
│   │       ├── sprites/       # PNGs gerados
│   │       └── levels/        # mapas ASCII
│   └── enzo-sim/              # harness headless (binário de desenvolvimento)
├── assets/                    # fontes dos assets + gerador
├── scripts/
│   ├── build-wasm.sh
│   ├── serve.mjs
│   └── gen-sprites.mjs
├── docs/
│   ├── ARCHITECTURE.md        # este documento
│   └── SPEC.md                # especificação do jogo
└── dist/                      # saída do build (gitignored)
```

### Responsabilidades por crate

| Crate        | Pode depender de                              | Nunca depende de                     |
| ------------ | --------------------------------------------- | ------------------------------------ |
| `enzo-core`  | `serde` (opcional, feature `serde`)           | render, wasm, arquivos, relógio, rede |
| `enzo-wasm`  | `enzo-core`, `macroquad`, `wasm-bindgen`, `web-sys` | regras de jogo              |
| `enzo-sim`   | `enzo-core`, `serde_json`                     | render, wasm                         |

`enzo-server` não é criado no MVP, mas o caminho está descrito na seção 9.

## 3. Fronteiras dos módulos (`enzo-core`)

| Módulo        | Responsabilidade | Não faz |
| ------------- | ---------------- | ------- |
| `math`        | `Vec2`, `Rect`, clamp, aproximação, AABB | conhecer entidades |
| `rng`         | PRNG determinístico com semente explícita | usar `rand::thread_rng` |
| `input`       | `PlayerCommand`, `SixSevenKey`, intenção por tick | ler teclado |
| `map`         | `TileMap`, `Tile`, parse de mapa ASCII, spawn, bandeira | carregar arquivo |
| `entities`    | `EntityId`, `Entity`, `EntityStore`, `EntitySpec`, componentes comuns e entidades concretas | desenhar |
| `entities::controller` | `Controller`, `PlayerController`/`PlayerInput`, `Ai`; quem decide a intenção | executar movimento |
| `moves`       | `MoveId`, `MoveSet`, `Requirements`, `AbilityContext`, validação de atribuição | decidir dano |
| `movements`   | componentes `Walk`, `Jump`, `Dash`, `Fly`, `SixSeven`; intenção vira velocidade | decidir dano |
| `physics`     | gravidade, integração, colisão AABB com tiles | decidir dano |
| `aura`        | saldo de aura: aplica o ganho do SixSeven e gasta no Mogar | desenhar barra |
| `attacks`     | `MeleeCombo`, `MogarAbility`, dano, i-frames, knockback | input direto |
| `ai`          | comportamentos de inimigos (patrulha etc.) | colisão |
| `progression` | XP, nível, pontos, atributos e stats derivados | salvar em disco |
| `world`       | orquestra o passo fixo e mantém o estado | render/IO |
| `snapshot`    | estado serializável para render/rede/save | conter lógica |
| `events`      | eventos do tick (dano, level-up, morte, vitória) | ser estado persistente |

### 3.1 Modelo padronizado de entidades

Jogador, inimigos e projéteis são o **mesmo tipo** `Entity`. O que muda é o
`EntityKind` e os dados específicos; os atributos compartilhados ficam sempre
nos mesmos componentes.

```rust
pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,      // Player | Enemy | Projectile | Item
    pub controller: Controller,// Player(PlayerController) | Ai(Ai) | None
    pub team: TeamId,          // PLAYERS | ENEMIES | NEUTRAL
    pub body: Body,            // posição, velocidade, tamanho, facing, on_ground
    pub health: Health,        // vida atual + i-frames
    pub stats: Stats,          // velocidade, força, vida máx., aura máx., dano do Mogar
    pub attributes: Attributes,// pontos do personagem
    pub equipment: Equipment,  // soma dos itens equipados
    pub movement: Movement,    // MoveSet + Walk | Jump | Dash | Fly | SixSeven
    pub combat: Combat,        // MoveSet + MeleeCombo | MogarAbility
}

pub enum EntityKind {
    Player(PlayerData),        // player_id, nível, XP, pontos, aura
    Enemy(EnemyData),          // espécie, XP, dano de contato
    Projectile(ProjectileData),// dono, dano, inimigos já atingidos
    Item(ItemData),            // categoria, requisitos, modificadores
}
```

- `Stats` é derivado de `Attributes` no jogador e fixo no inimigo; os dois usam a
  mesma struct.
- `EntityStore` é o único ponto de criação/remoção:
  `spawn(EntitySpec)`, `despawn(id)`, `get`, `get_mut`, `iter`, `players`,
  `enemies`, `in_rect`.
- `EntitySpec` tem construtores explícitos: `EntitySpec::player`,
  `EntitySpec::slime`, `EntitySpec::mogar` e `EntitySpec::item`. Cada construtor
  vive no módulo da entidade concreta. Nada de montar `Entity` à mão em sistemas.
- O store usa `BTreeMap<EntityId, Entity>` para iteração determinística e IDs
  estáveis. IDs não são reutilizados no MVP.
- Sistemas nunca duplicam atributos: dano, vida, velocidade e time vêm dos
  componentes comuns. Novos tipos de entidade entram como nova variante de
  `EntityKind` + construtor em `EntitySpec`.
- `Controller` é o componente de controle: `Player(PlayerController)` guarda o
  `PlayerInput` do tick, `Ai(Ai)` guarda o comportamento e seu estado, e `None`
  cobre projéteis/itens. Não existe campo `ai` paralelo no `Entity`.
- `Movement` agrupa movimentos inerentes como componentes independentes
  (`Walk`, `Jump`, `Dash`, `Fly`, `SixSeven`) em `movements/`. Cada um tem estado
  próprio e declara `REQUIREMENTS`.
- `SixSeven` é o movimento que farma aura alternando `6`/`7`. O componente guarda
  combo, janela e lentidão; o sistema de aura consome o pedido e aplica o saldo.
- `Combat` agrupa movimentos de ataque (`MeleeCombo`, `MogarAbility`) em
  `attacks/`. Novos golpes entram como um módulo novo, nunca como lógica solta
  no `World`.
- `MoveSet` é o conjunto determinístico de movimentos atribuídos. É um bitset
  serializável, sem `HashSet`.
- Todo movimento declara `Requirements` (tipos de entidade, nível mínimo e
  atributos mínimos). `MoveSet::try_assign` valida o `AbilityContext` da
  entidade antes de inserir. Jogador nasce com andar/pular/dash/SixSeven, além
  de melee/mogar; inimigo nasce só com andar; projétil não tem movimentos.
- `Fly` exige nível 5 e 5 pontos em Aura; o jogador do MVP não nasce com ele.
- Para adicionar um movimento: crie o componente no diretório correto, declare
  `REQUIREMENTS`, adicione a variante em `MoveId`/`MoveId::requirements`, ligue
  no construtor do `EntitySpec` e adicione a etapa no sistema. O cliente só
  ganha uma tecla nova e um comando.

```rust
// Cada movimento declara para quem pode ser atribuído.
pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Player],
    5, // nível mínimo
    Attributes { aura: 5, ..Attributes::ZERO },
);

// A entidade só recebe o movimento se cumprir os requisitos.
let spec = EntitySpec::player(0, pos).with_move(MoveId::Fly)?;
let mut entity = store.spawn(spec);
entity.try_assign_move(MoveId::Fly)?; // revalida em runtime
```

### Ordem dos sistemas em `World::step`

A ordem é parte do contrato: mudá-la muda replay e rede.

1. `input`: aplica comandos aos controladores dos jogadores (intenção de
   movimento, edges de pulo/dash/ataque/SixSeven).
2. `aura`: consome o pedido do SixSeven e aplica o ganho no saldo.
3. `movements`: atualiza os componentes atribuídos (incluindo a janela do
   SixSeven) e a intenção vira velocidade.
4. `physics`: integra e resolve colisão com tiles (X, depois Y).
5. `attacks::melee`: avança o combo de 3 golpes e aplica hitbox.
6. `attacks::mogar`: dispara projétil se houver aura e input.
7. `projectiles`: move projéteis e resolve colisão com tiles/inimigos.
8. `ai`: patrulha e dano de contato.
9. `collisions`: dano em inimigos/jogador, i-frames, knockback.
10. `progression`: concede XP, sobe de nível, gera pontos.
11. `cleanup`: remove mortos e projéteis expirados.
12. `events`: devolve a lista de eventos do tick.

## 4. Contrato de simulação

```rust
pub const TICK_HZ: u32 = 60;
pub const DT: f32 = 1.0 / 60.0;
pub const MAX_PLAYERS: usize = 4;

#[derive(Clone, Copy, Default)]
pub struct PlayerCommand {
    pub move_x: i8,          // -1, 0, 1
    pub jump: bool,          // pressionado neste tick (edge)
    pub jump_held: bool,     // estado contínuo, para pulo variável
    pub dash: bool,          // pressionado neste tick (edge)
    pub melee: bool,         // pressionado neste tick (edge)
    pub mogar: bool,         // pressionado neste tick (edge)
    pub sixseven: Option<SixSevenKey>, // Six | Seven, pressionado neste tick
    pub allocate: Option<Attribute>, // 1..4 na tela de level-up
}

pub struct TickInput {
    pub commands: [Option<PlayerCommand>; MAX_PLAYERS],
}

pub struct TickOutput {
    pub events: Vec<GameEvent>,
}
```

- `World::step(&mut self, input: TickInput) -> TickOutput` é a única porta de
  avanço do tempo.
- `World::snapshot() -> WorldSnapshot` é puro e barato o suficiente para 20 Hz.
- `World::write_snapshot(&mut WorldSnapshot)` preenche um buffer existente e
  reaproveita `entities`, para o cliente não alocar a cada tick.
- `PlayerId` é estável e indexa `commands`; jogadores ausentes são `None`.
- Inputs são **edge-triggered** para pulo/ataque/SixSeven. O cliente calcula o
  edge; o core não conhece key repeat do navegador.
- O core guarda `tick: u64` e um `Rng` com semente. Nada lê o relógio.

### Determinismo

- Proibido: `std::time`, `rand::thread_rng`, `HashMap`/`HashSet` em lógica,
  iterar `f32` com ordem não determinística, estado global mutável.
- Permitido: `Vec`, `BTreeMap`, `f32` atrás do alias `Scalar`.
- **Trade-off:** o MVP usa `f32` (`type Scalar = f32`). Testes e replays são
  determinísticos na mesma plataforma. Como o multiplayer é autoritativo por
  snapshot, não precisamos de bit-exatidão entre wasm e x86. Se no futuro
  quisermos lockstep, troca-se `Scalar` por ponto fixo (`i32` em 1/256) sem
  mudar as assinaturas.
- Teste de determinismo: dois `World` com a mesma semente e a mesma sequência de
  `TickInput` devem produzir snapshots idênticos (hash) após N ticks.

## 5. Snapshot e serialização

```rust
#[derive(Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub version: u16,
    pub tick: u64,
    pub phase: GamePhase,          // Playing | LevelUp | Dead | Victory
    pub entities: Vec<EntitySnapshot>,
    pub level_width: f32,
    pub level_height: f32,
    pub flag: Option<Rect>,
    pub camera_hint: Vec2,
}

#[derive(Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub kind: SnapshotKind,        // Player { player_id, level, xp, ... } | Enemy | Projectile
    pub pos: Vec2,
    pub size: Vec2,
    pub vel: Vec2,
    pub facing: i8,
    pub on_ground: bool,
    pub health: f32,
    pub max_health: f32,
    pub aura: f32,
    pub max_aura: f32,
    pub combo: u32,
    pub running: bool,
}
```

- `WorldSnapshot` não contém lógica; é o contrato entre core e render/rede.
- `serde` fica atrás da feature `serde` no `enzo-core` para manter o core leve.
- JSON para debug, save e testes; `postcard`/`bincode` para rede no futuro.
- `version` sobe a cada mudança incompatível; save e rede recusam versão antiga.
- O render **nunca** muta o snapshot nem o `World` diretamente.

## 6. Cliente WASM (`enzo-wasm`)

### Loop

```text
acumulador += frame_dt
ticks = min(floor(acumulador / DT), 5)
enquanto ticks > 0:
    swap(previous, current)
    world.step(input_do_tick())
    world.write_snapshot(&mut current)   # reutiliza o vetor de entidades
    acumulador -= DT
se acumulador >= DT: acumulador = 0      # descarta backlog de frame lento
alpha = acumulador / DT
render(previous, current, alpha)
```

- `frame_dt` é limitado (ex.: 0,2 s) para evitar espiral da morte.
- No máximo 5 ticks por frame; o resto do tempo é descartado em vez de virar
  uma fila que trava ainda mais o próximo frame.
- Render interpola posição entre o snapshot anterior e o atual usando `alpha`;
  a câmera usa o mesmo foco interpolado.
- A simulação nunca depende do framerate.

### Input (mapeamento inicial)

| Ação             | Teclas                          |
| ---------------- | ------------------------------- |
| Mover            | `A`/`D` ou `←`/`→`              |
| Pular            | `Espaço` ou `W`/`↑`             |
| Correr           | double-tap na direção (core)    |
| Dash             | `Shift` (esquerdo ou direito)   |
| Combo físico     | `J`                             |
| Mogar            | `K`                             |
| SixSeven (aura)  | `6` e `7`, alternando           |
| Distribuir ponto | `1` Velocidade, `2` Força, `3` Vida, `4` Aura |
| Confirmar level  | `Enter`                         |

### Render, câmera e HUD

- `render.rs` desenha o mundo a partir do snapshot; o fundo é um gradiente em
  textura cacheada e as entidades são interpoladas.
- `tiles.rs` agrupa o mapa em chunks de 16×16 tiles (512×512 px). Cada chunk
  visível vira uma malha estática uma única vez; chunks fora da câmera nunca são
  construídos. Isso evita percorrer e emitir todos os tiles do mapa por frame.
- `camera.rs` segue o jogador com suavização e limita ao tamanho do mapa.
- `hud.rs` desenha barras de vida/aura/XP, nível, pontos e combo.
- `sprites.rs` carrega o atlas PNG e escolhe frames por estado/animacão.
- `save.rs` grava JSON versionado em `localStorage` (`enzoballz.save.v1`).

### Regra de ouro

Se uma decisão muda o resultado do jogo (dano, colisão, custo, XP, level-up),
ela pertence ao `enzo-core`. O cliente só traduz input e pixels.

## 7. Harness de build

### Toolchain

```bash
rustup target add wasm32-unknown-unknown
```

- `rust-toolchain.toml` fixa o canal (1.92.0) e o target `wasm32-unknown-unknown`.
- O cliente usa o bundle JS do macroquad; o build não precisa de
  `wasm-bindgen-cli`.

### Pipeline (`scripts/build-wasm.sh`)

```text
1. cargo build -p enzo-wasm --release --target wasm32-unknown-unknown
2. criar dist/.nojekyll (GitHub Pages não deve processar o bundle com Jekyll)
3. copiar wasm, index.html, plugins JS e assets/ para dist/
4. o resultado é um site estático autocontido
```

### Servidor local (`scripts/serve.mjs`)

- Node puro, sem dependências.
- Serve `dist/` com `Content-Type: application/wasm` para `.wasm`.
- Porta padrão `8080`, `--port` configurável.
- `npm run dev` = build + serve; `npm run build` = só build.

### Hospedagem estática (GitHub Pages)

- `.github/workflows/pages.yml` compila e publica `dist/` via GitHub Actions.
- O bundle usa caminhos relativos, então funciona na raiz ou em
  `https://usuario.github.io/repositorio/`.
- Pages não executa lógica de servidor nem WebSocket. O cliente estático é
  compatível com o futuro servidor autoritativo porque só envia comandos e
  desenha snapshots.
- Quando o multiplayer existir, a URL do servidor deve ser lida em runtime
  (arquivo de configuração), não fixada no wasm, para o mesmo artefato apontar
  para servidores diferentes.
- Adicionar rede exige atualizar o escopo em `docs/SPEC.md` antes.

### Alternativa

`trunk` pode substituir o pipeline se o time preferir, mas o caminho acima é o
suportado e testado no MVP.

### Critérios de aceite do build

- `cargo test -p enzo-core` verde.
- `cargo clippy -p enzo-core -- -D warnings` verde.
- `npm run build` gera `dist/` e o jogo abre no navegador via `npm run dev`.
- Recarregar a página restaura nível, XP, atributos e pontos do save.

## 8. Harness de testes

### Pirâmide

| Nível            | O que cobre | Onde |
| ---------------- | ----------- | ---- |
| Unitário         | math, mapa, aura, progressão, dano | `#[cfg(test)]` por módulo |
| Cenário          | pulo, colisão, combo, Mogar, morte, bandeira | `crates/enzo-core/tests/` |
| Determinismo     | mesma entrada + mesma semente = mesmo hash | `tests/determinism.rs` |
| Headless (manual)| rodar uma sequência de comandos e imprimir snapshots | `enzo-sim` |

### Cenários mínimos

1. **Física:** jogador cai, pousa no chão e não atravessa tile sólido.
2. **Pulo:** pulo só no chão; coyote time e jump buffer respeitados.
3. **Corrida:** double-tap dentro da janela entra em corrida; soltar sai.
4. **Aura:** `6,7,6,7` gera aura crescente; `6,6` zera o combo; sem farmar, o
   combo expira e a lentidão termina.
5. **Combo físico:** três `J` dentro da janela aplicam 3 hits; o terceiro
   empurra; fora da janela reinicia.
6. **Mogar:** sem aura não dispara; com aura consome o custo, acerta inimigo e
   desaparece no tile sólido.
7. **Progressão:** XP suficiente sobe de nível, concede 1 ponto e pausa.
8. **Atributos:** cada ponto altera o stat derivado correto.
9. **Morte:** HP 0 -> respawn no início, vida/aura cheias, XP/nível mantidos,
   inimigos renascem.
10. **Vitória:** tocar a bandeira -> fase vencida, save gravado.

### `enzo-sim`

```bash
cargo run -p enzo-sim -- --script tests/replays/basic.json --ticks 600
```

- Lê uma sequência de `TickInput` em JSON, roda o core e imprime snapshots
  (ou um hash final).
- Serve para depurar sem navegador e para gerar replays de bug.
- Não depende de macroquad, então roda em CI.

## 9. Caminho para multiplayer

O que já fica pronto no MVP:

- `World` aceita `MAX_PLAYERS` comandos por tick e tem `PlayerId` estável.
- Nenhuma regra lê teclado, relógio ou globals.
- `WorldSnapshot` é serializável e versionado.
- A ordem dos sistemas é fixa e documentada.
- `enzo-sim` prova que o core roda fora do navegador.

O que fica deliberadamente fora do MVP:

- Servidor, WebSocket, matchmaking, autenticação, anti-cheat.
- Predição de cliente, reconciliação, interpolação de remotos.
- Persistência de progressão no servidor (no MVP é `localStorage`).

Desenho futuro:

```text
Cliente A ──comandos 60 Hz──▶ Servidor autoritativo (enzo-core)
Cliente B ──comandos 60 Hz──▶   ├─ step a 60 Hz
                                └─ snapshots 20 Hz ──▶ clientes
```

- `enzo-server` reutiliza `enzo-core` e um `CommandSource` de rede.
- Cliente aplica snapshot e interpola; predição local entra só se a latência
  justificar.
- Save passa a ser do servidor; o `localStorage` vira cache/offline.

## 10. Riscos e decisões em aberto

| Risco | Mitigação |
| ----- | --------- |
| `f32` não é bit-exato entre wasm/x86 | Multiplayer autoritativo por snapshot; alias `Scalar` permite ponto fixo depois |
| Versão do `wasm-bindgen-cli` divergir | Pin no `rust-toolchain.toml` + checagem no script de build |
| Build wasm grande/lento | `opt-level = "s"`, `lto = true`, `codegen-units = 1` no perfil release |
| Sprites gerados ficarem feios | Gerador versionado; trocar PNGs depois sem tocar no core |
| Save quebrar entre versões | `version` no JSON + migração explícita ou reset seguro |
| Escopo crescer (mais poderes/fases) | `SPEC.md` lista não-objetivos; novos poderes entram como módulo em `attacks` ou `movements` |

## 11. Definition of Done do harness

- [ ] Workspace compila com `cargo build --workspace`.
- [ ] `cargo test -p enzo-core` cobre os 10 cenários da seção 8.
- [ ] Teste de determinismo passa com 600 ticks.
- [ ] `npm run build` produz `dist/` e o jogo roda em `npm run dev`.
- [ ] `enzo-sim` roda um replay e imprime o hash final.
- [ ] Nenhum arquivo do `enzo-core` importa `macroquad`, `wasm-bindgen` ou `web-sys`.
- [ ] Save/restore funciona após recarregar a página.
