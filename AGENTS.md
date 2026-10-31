# AGENTS.md — EnzoBallZ

Instruções para agentes de código (OpenCode, Claude Code, Codex, Cursor etc.)
que trabalharem neste repositório. Leia também:

- `docs/SPEC.md` — regras de jogo e balanceamento.
- `docs/ARCHITECTURE.md` — harness, contratos e fronteiras dos módulos.

## 1. O projeto

EnzoBallZ é um jogo de plataforma 2D estilo Mario, escrito em Rust e compilado
para WebAssembly. O Enzo corre, pula, faz combo físico, farma aura alternando
`6`/`7` e gasta aura no golpe **Mogar**. Inimigos dão XP; o level-up distribui
pontos entre Velocidade, Força, Vida e Aura.

O MVP é single-player, mas o core é desenhado para N jogadores e para um futuro
servidor autoritativo.

## 2. Stack e ambiente

- Rust 1.92.0 (edition 2021), workspace Cargo.
- Cliente: `macroquad` + `wasm-bindgen` (target `wasm32-unknown-unknown`).
- Servidor local de desenvolvimento: Node 24, sem dependências.
- Sem framework de teste externo no core; usar `#[test]` nativo.
- Sem assets de terceiros: sprites são gerados por script.

## 3. Comandos

```bash
# Qualidade e testes
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Simulação headless
cargo run -p enzo-sim -- --ticks 600

# Build WASM (quando o pipeline existir)
rustup target add wasm32-unknown-unknown
npm run build
npm run dev
```

Sempre rode `cargo test` e `cargo clippy` antes de commitar. Se o build WASM
falhar por falta de toolchain, registre o bloqueio no commit/PR em vez de
esconder o erro.

## 4. Estrutura

```text
crates/enzo-core/   simulação pura (física, mapa, entidades, aura, combate, progressão)
crates/enzo-wasm/   cliente: macroquad, input, render, HUD, save
crates/enzo-sim/    harness headless para replays e CI
assets/             sprites originais e mapas
docs/               spec e arquitetura
scripts/            build WASM, servidor local, gerador de sprites
```

## 5. Invariantes de arquitetura

Estas regras são obrigatórias. Quebrá-las exige atualizar `docs/ARCHITECTURE.md`
e justificar no commit.

1. **`enzo-core` é puro.** Não importa `macroquad`, `wasm-bindgen`, `web-sys`,
   `std::fs`, `std::time`, `std::env` ou `rand::thread_rng`.
2. **O core é a única fonte de verdade.** Dano, colisão, custo de aura, XP,
   level-up e vitória são decididos no core. O cliente só traduz input e pixels.
3. **Tick fixo de 60 Hz.** `DT = 1.0 / 60.0`; o render pode interpolar, mas
   nunca ditar a simulação.
4. **Comandos são dados.** `PlayerCommand` por jogador por tick. O core não lê
   teclado.
5. **Estado explícito e serializável.** `WorldSnapshot` é o contrato com render,
   save e rede. Não coloque lógica no snapshot.
6. **Determinismo.** Nada de relógio, `thread_rng`, `HashMap`/`HashSet` em
   lógica ou estado global mutável. Use `Rng` com semente e `Vec`/`BTreeMap`.
7. **Ordem dos sistemas é contrato.** Respeite a ordem de `World::step`
   documentada em `docs/ARCHITECTURE.md`.
8. **N jogadores desde o início.** `MAX_PLAYERS`, `PlayerId` estável e
   `commands` indexados por jogador. Nada de assumir um único jogador no core.
9. **Sem rede no MVP.** Não adicione WebSocket, servidor ou matchmaking sem
   atualizar o escopo em `docs/SPEC.md`.

## 6. Convenções de código

- Identificadores (tipos, funções, campos) em inglês; comentários e textos de UI
  em português (pt-BR).
- Prefira módulos pequenos e funções puras. Cada módulo tem uma
  responsabilidade (ver tabela em `docs/ARCHITECTURE.md`).
- Nada de `unwrap()`/`expect()` em código de biblioteca. Use `Result` ou
  `Option` e trate o erro. Em testes, `unwrap()` é permitido.
- Evite `panic!` no core; erros de parse de mapa devem virar `LevelError`.
- Use `f32` atrás do alias `Scalar` quando fizer sentido, para permitir migração
  futura para ponto fixo.
- Não adicione dependências sem necessidade clara. O core deve continuar leve;
  `serde` fica atrás da feature `serde`.
- Mantenha o código formatado com `cargo fmt`; não reformate arquivos não
  relacionados no mesmo commit.
- Comentários explicam **por quê**, não o óbvio. Documente contratos e
  invariantes, não cada linha.

## 7. Testes

Toda mudança de regra de jogo precisa de teste. O mínimo:

- **Unitário:** funções de `math`, `map`, `aura`, `progression`, `combat`.
- **Cenário:** pulo, colisão, double-tap, combo de 3, Mogar, morte, bandeira.
- **Determinismo:** mesma semente + mesmos comandos = mesmo snapshot.
- **Regressão:** bug corrigido ganha um teste que falharia antes.

O harness de cenários vive em `crates/enzo-core/tests/`. Use `enzo-sim` para
reproduzir bugs sem navegador.

Não use `sleep` em teste. Avance ticks explicitamente com `World::step`.

## 8. Commits

- Commits **incrementais** e com uma mudança lógica por vez.
- Formato Conventional Commits: `feat:`, `fix:`, `test:`, `docs:`, `refactor:`,
  `chore:`, `build:`.
- Mensagem no imperativo e curta; o corpo explica o porquê quando necessário.
- Inclua o teste no mesmo commit da mudança que ele cobre.
- Nunca commite `target/`, `dist/`, `node_modules/` ou saves locais.
- Exemplos:
  - `feat(core): add fixed-timestep world loop`
  - `test(core): cover aura alternation combo`
  - `fix(physics): resolve one-way platform landing`

## 9. Assets

- Sprites devem ser originais, pixel-art e gerados por
  `scripts/gen-sprites.mjs`.
- Não copie personagens, sprites, músicas ou nomes de obras existentes
  (Dragon Ball, Mario etc.). O nome "Mogar" é do projeto.
- Novos assets entram em `assets/` com o script que os gera versionado.
- Se trocar um PNG, mantenha o atlas e as dimensões documentados.

## 10. O que NÃO fazer

- Colocar regra de jogo no `enzo-wasm`.
- Ler teclado, relógio ou arquivo dentro do `enzo-core`.
- Usar `f32` para tomar decisões de rede que exijam bit-exatidão (o multiplayer
  é autoritativo por snapshot; se virar lockstep, migre para ponto fixo).
- Criar estado global, `lazy_static` ou singleton mutável no core.
- Adicionar multiplayer, áudio, menu ou novas fases sem atualizar o escopo.
- Fazer um commit gigante misturando refactor, feature e formatação.

## 11. Definition of Done

Antes de considerar uma tarefa concluída:

- [ ] `cargo fmt --all -- --check` passa.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passa.
- [ ] `cargo test --workspace` passa.
- [ ] A mudança tem teste (ou justificativa explícita de por que não).
- [ ] `docs/SPEC.md` e `docs/ARCHITECTURE.md` continuam verdadeiros.
- [ ] O core não ganhou dependência de render/wasm/IO.
- [ ] O commit é incremental e tem mensagem no padrão.

## 12. Estado atual

- [x] Spec e arquitetura documentados.
- [ ] Workspace Cargo e scaffold.
- [ ] `enzo-core`: math, rng, map, entity, physics.
- [ ] `enzo-core`: aura, combate, IA, progressão, world, snapshot.
- [ ] `enzo-sim` headless.
- [ ] `enzo-wasm`: loop, input, render, HUD, save.
- [ ] Sprites e pipeline de build WASM.
- [ ] Testes de cenário e determinismo verdes.
