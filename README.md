# EnzoBallZ

Plataforma 2D estilo Mario, em Rust compilado para WebAssembly. O Enzo corre,
pula, faz combo físico, farma aura alternando `6`/`7` e gasta aura no golpe
**Mogar**.

## Status

Em desenvolvimento. O core de simulação é puro Rust e roda no navegador, em
testes headless e, no futuro, em um servidor autoritativo.

## Documentação

- [`docs/SPEC.md`](docs/SPEC.md) — regras de jogo, controles e balanceamento.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — harness, contratos e build.
- [`AGENTS.md`](AGENTS.md) — convenções para agentes de código.

## Desenvolvimento

```bash
# Testes do core (sem navegador)
cargo test -p enzo-core

# Simulação headless
cargo run -p enzo-sim -- --ticks 600

# Build WASM + servidor local
rustup target add wasm32-unknown-unknown
npm run dev
```

O jogo abre em `http://localhost:8080`.

## Estrutura

```text
crates/enzo-core/   simulação pura: física, mapa, entidades, aura, combate, progressão
crates/enzo-wasm/   cliente macroquad: input, render, HUD e save
crates/enzo-sim/    harness headless para replays e CI
assets/             mapas e sprites originais
web/                HTML e plugins JS do miniquad/macroquad
scripts/            build, servidor local e gerador de sprites
```
