# CLAUDE.md
*Working context for Claude Code on the Pledge & Crown project.*

## Project identity

You are working on **Pledge & Crown**, a pixel-art RPG that teaches Rust. The owner is Matt Gates (Ridge Cell Repair LLC, Chico CA). The project is Rust-first by mandate — the game eats its own dog food.

## The team (lean roster, locked 2026-05-02)

The parent agent (you, when this file is loaded as the system prompt) drives the project. Three standing subagents and two dormant ones are defined under `.claude/agents/`. Triage by trigger:

**Active — invoke routinely:**
- **`rust-lead`** — runs after every merge to `main`, after each parallel-agent batch, before any release tag. Reviews architecture, idiomatic Rust, test coverage, hard-rule compliance. Files findings to `.claude/reviews/`. Does not block. (`.claude/agents/rust-lead.md`)
- **`art-lead`** — drives `claude.ai/design` via the Wraith browser MCP to generate sprite assets, runs the JSX→PNG render pipeline, stages deliverables. Matt is the approver. Invoke when the curriculum adds NPCs that need real art (currently 10 of 21 NPCs are SPRITE_PLAYER placeholders). (`.claude/agents/art-lead.md`)
- **`voice-editor`** — owns NPC dialogue + tutorial voice consistency across the cast. Invoke after each curriculum batch lands. Edits in place; commits a single cohesive voice pass. (`.claude/agents/voice-editor.md`)

**Dormant — DO NOT invoke unless their activation conditions hold:**
- **`security-reviewer`** (`.claude/agents/dormant/security-reviewer.md`) — activates only in autopilot mode AND when `compile-api/` changes. Threat-models against design/05 §2.
- **`audit-extender`** (`.claude/agents/dormant/audit-extender.md`) — activates only when a bug slips past the audit harness. Writes the test that would have caught it.

**Roles deliberately NOT staffed** (skip these when the topic comes up):
- "Art review team" — Matt is the single approver. Multiple agent reviewers create contradiction without judgment.
- "Story review team" — voice-editor self-edits; Matt approves. Two-tier review is bureaucracy at this scale.
- "Code reviewer that blocks merges" — too slow against parallel-agent work; the audit harness already gates correctness. Rust-lead reviews for taste, async.

**State + outputs:**
- Per-agent cursors: `.claude/state/<agent>-cursor.txt` (last sha reviewed)
- Findings: `.claude/reviews/<agent>-<date>-<sha>.md`
- Audit gap log: `.claude/audit-gaps.md` (institutional memory of bugs the harness almost shipped)


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

- Rust source SPDX headers depend on which crate the file lives in (decision Q3, 2026-05-01):
  - Files under `game/` → `// SPDX-License-Identifier: MIT`
  - Files under `compile-api/` → `// SPDX-License-Identifier: AGPL-3.0-or-later`
  - Rationale: MIT on the game binary keeps Steam/itch distribution unencumbered (Steam's TOS restricts copyleft on the binary itself). AGPL on the compile-API server is the moat against API competitors and matches the Wraith pattern. Player code submitted to the API is the player's; the AGPL only governs the server.
- All shell scripts go in `scripts/` as `.ps1` (Windows) or `.sh` (Linux server). Never inline.
- Asset files: `assets/sprites/{category}/{name}_{frame}.png`, e.g. `assets/sprites/npc/borrow_checker_idle_0.png`.

## When in doubt

Read the relevant `design/0X-*.md` doc. If still ambiguous, default to: less code, more tests, more logs.
