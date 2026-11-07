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
- [`docs/ACTIONS_AND_SKILLS.md`](docs/ACTIONS_AND_SKILLS.md) — plano de action
  slots, skills modulares e controles mobile.
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

## Hospedagem estática (GitHub Pages)

O build gera um site estático completo em `dist/`: wasm, HTML, plugins JS e
assets. Ele usa caminhos relativos, então funciona tanto na raiz de um domínio
quanto em um subdiretório como `https://usuario.github.io/enzoballz/`.

O workflow [`.github/workflows/pages.yml`](.github/workflows/pages.yml) compila
e publica esse bundle a cada push em `main` (ou `master`). Para ativar:

1. No GitHub, abra **Settings → Pages**.
2. Em **Build and deployment → Source**, escolha **GitHub Actions**.
3. Faça push na branch principal ou rode o workflow manualmente em **Actions**.

O `dist/.nojekyll` é criado pelo build para impedir o processamento do Jekyll.

## Multiplayer no futuro

A hospedagem estática não impede o multiplayer. O cliente continua sendo só
input + render; quando o servidor autoritativo existir, ele rodará o mesmo
`enzo-core` e o cliente trocará `World::step` local por envio de comandos e
recebimento de snapshots via WebSocket.

O mesmo bundle estático poderá ser usado em produção: a URL do servidor deve
entrar em um arquivo de configuração em runtime (por exemplo `config.json`),
não fixa no build. Assim o cliente hospedado no Pages aponta para um servidor
local, de homologação ou de produção sem recompilar. O servidor pode ficar em
qualquer host com WebSocket (VPS, Fly.io, Render etc.) e, se necessário, servir
o próprio `dist/` no futuro para manter tudo no mesmo domínio.

O escopo atual ainda é single-player e sem rede; adicionar WebSocket exige
atualizar `docs/SPEC.md` antes.

## Estrutura

```text
crates/enzo-core/   simulação pura: física, mapa, entidades, aura, combate, progressão
crates/enzo-wasm/   cliente macroquad: input, render, HUD e save
crates/enzo-sim/    harness headless para replays e CI
assets/             mapas e sprites originais
web/                HTML e plugins JS do miniquad/macroquad
scripts/            build, servidor local e gerador de sprites
```
