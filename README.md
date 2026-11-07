# EnzoBallZ

A 2D platformer in the style of Mario, written in Rust and compiled to
WebAssembly. Enzo runs, jumps, performs a physical combo, farms aura by
alternating `6`/`7`, and spends aura on the **Mogging** special.

## Status

In development. The simulation core is pure Rust and runs in the browser, in
headless tests, and, in the future, on an authoritative server.

## Documentation

- [`docs/SPEC.md`](docs/SPEC.md) — game rules, controls, and balancing.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — harness, contracts, and build.
- [`docs/ACTIONS_AND_SKILLS.md`](docs/ACTIONS_AND_SKILLS.md) — action slots,
  modular skills, and mobile controls (plan and implementation status).
- [`AGENTS.md`](AGENTS.md) — conventions for coding agents.

## Development

```bash
# Core tests (no browser)
cargo test -p enzo-core

# Headless simulation
cargo run -p enzo-sim -- --ticks 600

# WASM build + local server
rustup target add wasm32-unknown-unknown
npm run dev
```

The game opens at `http://localhost:8080`.

## Static hosting (GitHub Pages)

The build produces a complete static site in `dist/`: wasm, HTML, JS plugins, and
assets. It uses relative paths, so it works both at a domain root and in a
subdirectory such as `https://user.github.io/enzoballz/`.

The [`.github/workflows/pages.yml`](.github/workflows/pages.yml) workflow
compiles and publishes that bundle on each push to `main` (or `master`). To
enable it:

1. On GitHub, open **Settings → Pages**.
2. Under **Build and deployment → Source**, choose **GitHub Actions**.
3. Push to the main branch or run the workflow manually in **Actions**.

The build creates `dist/.nojekyll` to prevent Jekyll processing.

## Multiplayer in the future

Static hosting does not prevent multiplayer. The client remains only input +
rendering; when an authoritative server exists, it will run the same `enzo-core`
and the client will replace local `World::step` with sending commands and
receiving snapshots over WebSocket.

The same static bundle can be used in production: the server URL should come
from a runtime configuration file (for example, `config.json`), not be fixed in
the build. That way the client hosted on Pages can point to a local, staging, or
production server without recompiling. The server can run on any host with
WebSocket support (VPS, Fly.io, Render, etc.) and, if needed, serve `dist/`
itself in the future to keep everything on the same domain.

The current scope is still single-player and offline; adding WebSocket requires
updating `docs/SPEC.md` first.

## Structure

```text
crates/enzo-core/   pure simulation: physics, map, entities, mechanics, skills, aura, progression
crates/enzo-wasm/   macroquad client: input, render, HUD, and save
crates/enzo-sim/    headless harness for replays and CI
assets/             maps and original sprites
web/                HTML and miniquad/macroquad JS plugins
scripts/            build, local server, and sprite generator
```
