# CLAUDE.md
*Working context for Claude Code on the Cargo & Crowns project.*

## Project identity

You are working on **Pledge & Crown**, a pixel-art RPG that teaches Rust. The owner is Matt Gates (Ridge Cell Repair LLC, Chico CA). The project is Rust-first by mandate — the game eats its own dog food.

## Operating constraints (from owner's preferences)

- **Windows dev environment.** When invoking shell commands, write them to a `.ps1` or `.bat` script file and execute the file rather than inline command strings. This avoids quoting/escape hell.
- **Verbose logging always.** Every system gets `tracing` instrumentation at `debug` level minimum. No silent failures. Use `tracing_subscriber::fmt().with_env_filter(...).init()` at startup. When in doubt, log more, not less.
- **Admin PowerShell is fine** for anything system-level. UAC is disabled on Matt's box.
- **Revenue urgency: 30 days.** Scope creep is the enemy. Every PR should ask: does this ship the MVP, or can it wait for v1.1?
- **Rust idioms over cleverness.** Prefer `Result<T, E>` over panics, `?` over `.unwrap()` outside of tests, named errors via `thiserror`. The game's audience reads our code as a teaching artifact.

## Tech stack (locked for v1)

- **Engine:** Bevy — verify current stable when scaffolding (doc text was written against 0.14; pin to whatever's stable as of session date and re-check `bevy_egui` / `egui_code_editor` / `bevy_ecs_tilemap` / `bevy_kira_audio` compatibility).
- **UI in-game:** `bevy_egui` for the code editor and menus.
- **Code editor widget:** `egui_code_editor` with Rust syntax highlighting.
- **Player code execution:** Server-side compile-to-WASM API for v1. See `design/05-tech-architecture.md` §3 for the rationale and the eventual offline path via embedded `wasmtime`.
- **Web build:** `wasm32-unknown-unknown` target + `wasm-bindgen`.
- **Mobile:** Tauri 2.0 wrapper (Matt's tauri-frontend skill is installed and applies here).
- **Backend (compile API):** Axum on a $7/mo Hetzner VPS, deployed via Matt's `openclaw-deploy` skill.
- **Telemetry:** Prometheus + Grafana (Matt's `grafana-dashboard` skill applies).

## Hard rules

1. **No `unsafe` in shipped game code** outside of FFI boundaries (the irony if our Rust-teaching game leaks UB would be catastrophic).
2. **Player code runs in a WASM sandbox**, never in the host process. No exceptions. Server-side compiles never let player input touch `Cargo.toml` — server owns the manifest, deps are vendored, builds are `--offline`. See `design/05-tech-architecture.md` §2.
3. **The Borrow Checker NPC's dialogue must be technically correct.** If a player's code error-checks differently than the in-game NPC says it would, that's a P0 bug.
4. **Curriculum-faithful.** Every Rust concept introduced in `design/01-curriculum.md` must be reachable through normal play. No teaching gaps.
5. **Art style is locked.** Read `design/03-art-style-bible.md` before generating, prompting for, or accepting any visual asset. Style drift kills indie games.
6. **Naming gate.** Working title is **Pledge & Crown** (selected 2026-04-25 after "Cargo & Crowns" was disqualified by RF Trademark Policy). Before any public asset, domain, or marketing ships, run USPTO TESS on "Pledge & Crown" and a final RF policy sanity check.
7. **No GitHub Actions.** Local CI only (`scripts/ci.ps1` + pre-commit hook + optional self-hosted runner). Per global instructions.

## File creation conventions

- All Rust source uses `// SPDX-License-Identifier: AGPL-3.0-or-later` header (matches Wraith Browser / OpenClaw licensing pattern).
- All shell scripts go in `scripts/` as `.ps1` (Windows) or `.sh` (Linux server). Never inline.
- Asset files: `assets/sprites/{category}/{name}_{frame}.png`, e.g. `assets/sprites/npc/borrow_checker_idle_0.png`.

## When in doubt

Read the relevant `design/0X-*.md` doc. If still ambiguous, default to: less code, more tests, more logs.
