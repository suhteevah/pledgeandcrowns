# HANDOFF.md

## Last Updated
2026-05-02 (handoff — game is playable end-to-end; 5 missions with full tutorials; 46-test audit harness)

## Project Status
🟢 Playable MVP — title screen → village → 5 NPCs → tutorial-rich code editor → live compile-API round-trip → mission progression. Workspace passes 46-test audit on every commit. Real `cargo build`/wasmtime sandbox per design/05 §2 still pending; v0 grader is pattern-based.

## What Was Done This Session (2026-05-02, day 6→8 push)

End-to-end Rust-teaching loop is live. Run `cargo run -p pledge_compile_api` in one terminal and `cargo run -p pledge_and_crown` in another; title screen → Space → Hearthstone Village → walk to any of 5 NPCs → F → editor opens with full tutorial sidebar (Concept / Syntax / Task / Hint) + starter code → write the answer → Compile → server grades and pipes flavor verdict back. Cleared missions show `[done]`.

**Plumbing shipped (commits `8d5f199`..`a6ee7df`):**

- `WorldPlugin` (`game/src/plugins/world.rs`) — 30×20 `bevy_ecs_tilemap` grid sourced from `tiles_village.png` (REF-04 atlas, 16 tile types). Hand-laid: dirt road across middle, cobblestone plaza around a 2-tile stone well, signpost at east edge of road, deterministic flora sprinkles. `TilemapAnchor::Center` (deprecated `get_tilemap_center_transform` replaced).
- `PlayerPlugin` — REF-01 sprite at 2× scale, WASD/arrow movement, world-bounds clamp. Camera lerp-follows the player at 6.0/sec; clamped so the void beyond the tilemap can't enter the viewport.
- `setup_camera` in `mod.rs` uses `OrthographicProjection` + `ScalingMode::FixedVertical { viewport_height: 180.0 }` (the latter imported from `bevy::camera`, NOT in prelude).
- `NpcPlugin` (`game/src/plugins/npc.rs`) — `NPC_ROSTER` static spec table drives spawning. 5 NPCs: Ferris, Trait Mage, Borrow Checker, Smith, Cartographer. Proximity system maintains `NearbyNpc` resource within 28px.
- `EditorPlugin` (`game/src/plugins/editor.rs`) — `bevy_egui` window toggled with `E`. Hand-rolled Rust syntax highlighter on `egui::TextEdit::multiline` because of the `egui_code_editor` 0.2 ↔ `bevy_egui` 0.39 egui-version diamond (0.34 vs 0.33). Tutorial side panel renders `## Heading` and triple-backtick code fences from each mission's `tutorial: &'static str`.
- `MissionPlugin` (`game/src/plugins/mission.rs`) — `MissionRegistry` (5 missions), `ActiveMission`, F-press handler that loads starter code into `EditorState`, `EguiPrimaryContextPass` interaction prompt.
- `CompileClientPlugin` (`game/src/plugins/compile_client.rs`) — `IoTaskPool` + `reqwest` POST to `127.0.0.1:7878/compile`; results stream back through a `crossbeam-channel`. Successful verdicts call `MissionProgress::mark_cleared`.
- `compile-api` split into lib + bin. `lib.rs` exposes `make_router()` so integration tests can mount the same router. `grader.rs` is the per-encounter pattern grader (5 missions wired); the freeform fallthrough catches any encounter without a grader arm.
- `StatePlugin` + `TitlePlugin` + `ProgressPlugin` — `GameState::{Title,InGame}`, `init_state` requires `bevy::state::app::StatesPlugin` (`MinimalPlugins` doesn't include it). Title spawns REF-10 wordmark; Space/Enter transitions to InGame which gates all village/player/NPC/mission systems via `OnEnter` + `run_if(in_state(InGame))`.

**Audit harness (`46/46` green every commit):**
- `compile-api/src/grader.rs` — 22 unit tests (every mission × pass + multiple fail modes + cross-mission contamination check)
- `compile-api/tests/http.rs` — 7 HTTP integration tests on a real axum server bound to an OS-assigned port via reqwest
- `game/tests/assets.rs` — 3 tests: every sprite path resolves on disk, no duplicates, all PNG
- `game/tests/registry.rs` — 6 tests: mission ids unique, NPC mission_id refs resolve, sprite paths in registry, npc names unique, tutorial substance (≥200 chars + has `## ` headers + has ``` fence)
- `game/tests/contract.rs` — 4 cross-crate tests: every mission has a canonical solution test fixture, all canonical solutions pass the live grader, unmodified starter_code does NOT trivially win, no mission falls through to freeform
- `game/tests/bevy_smoke.rs` — 4 minimal-app boot tests for headless plugins

**Bugs the audit caught and fixed:**
1. All 3 original starter codes leaked grader needles (e.g. intro starter said `// Bind the value 42 to a variable named answer` → grader saw `let answer` + `42` and passed). Rewrote starters as neutral `_todo` templates.
2. Grader's `or_else` chain returned the second-attempt err message, so failure said `+=1` instead of canonical `+= 1`. Rewrote those arms with explicit conditions.

**Decisions / changes from earlier today:**
- 4 design questions locked: leaderboards deferred to v1.1, dual-license (game = MIT, compile-api = AGPL-3.0-or-later), itch.io→Steam ladder, Temple as surprise reveal.
- SPDX headers added to all `.rs` files (MIT in `game/`, AGPL in `compile-api/`); `LICENSE-MIT`, `LICENSE-AGPL`, `LICENSE.md` at repo root.
- TESS authoritative GREEN: zero PLEDGE+CROWN combinations in USPTO records. Cleared to file ITU and commission marketing.
- 10 reference PNGs rendered + committed at `design/art/refs/png/`. JSX→PNG harness lives at `scripts/render-refs-inline.html` + `scripts/render-refs.py`.
- Cargo target dir moved to `G:/cargo-target/pledgeandcrown` via `.cargo/config.toml` because J: drive was full.
- GitHub repo transferred from `suhteevah/pledgeandcrowns` → `pledgeandcrown/pledgeandcrown` (org owned).

## Earlier this calendar day (2026-05-01 evening)

- **10 reference images delivered** by Matt as a Claude-Project bundle, extracted into `design/art/`. (Earlier in the day: TESS GREEN, design decisions locked, art deliverables spec, dual-license implementation, repo transfer, JSX→PNG export pipeline. Full timeline in commit log `git log --oneline`.) Format is unconventional but excellent: each REF is an ASCII pixel grid (e.g. `'........XXXXXX................'`) with single-char palette codes (X=Coalblack outline, P=Pink quartz, etc.), wrapped in a renderable `.jsx` component. Palette compliance is mathematically guaranteed because every char must resolve through `palette.js`. Files:
  - `refs/ref-01-player.jsx` … `refs/ref-10-title.jsx` (10 references, REF-09 has v1 + v2; v2 is canonical)
  - `palette.js` (32-color Heraldic Code v2.0 with role names)
  - `pixel-art.jsx` + `design-canvas.jsx` + `refs/artboard-frame.jsx` + `refs/palette-legend.jsx` (rendering scaffold)
  - `Pledge & Crown - Reference Set.html` (in-browser preview using Babel-standalone)
  - `uploads/` (frozen copies of bible + handoff prompts that the Project was given as context)
- **Bible v2.0 reference-image-set milestone is COMPLETE.** Bulk asset gen is unblocked.
- **GitHub remote wired.** `origin = https://github.com/suhteevah/pledgeandcrowns.git`. *Note: this is Matt's personal namespace, not the planned `pledgeandcrown` org. Worth confirming whether the org was created and if the repo should be transferred, or if personal-namespace is the chosen path. Not a blocker.*

## Reference-art PNG export pipeline

PNG exports of all 10 references now live at `design/art/refs/png/REF01..REF10.png`. Contractor-deliverable contract per `04b-art-deliverables.md` §2 is satisfied. Native pixel sizes verified: 32×32 (player/Ferris/goblin), 64×64 (Borrow Checker / village tiles / tower tiles), 16×16 (potion), 320×180 (worldmap), 640×360 (editor frame, title screen).

Workflow is a 2-step manual flow because Babel-standalone can't load external `text/babel src=` files over `file://` (Chrome treats sibling .jsx as cross-origin under file: scheme). Bundled into `scripts/`:
- `render-refs-inline.html` — self-contained harness that inlines all .jsx + palette + the renderer; mounts every REFxx React component, captures each canvas as a PNG data URL into `window.__refs`.
- `render-refs.py` — zero-dep decoder; reads `scripts/.refs-snapshot.json` (gitignored), writes PNGs to `design/art/refs/png/`.
- `render-refs.md` — workflow docs.

CI does NOT run this. PNGs are committed artifacts. Re-run only when a `.jsx` ref source changes.

## What Was Done This Session (2026-05-01)

- **Locked all 4 outstanding design decisions** (Q2 leaderboards, Q3 licensing, Q4 distribution, Q5 Temple marketing). See "Open questions for Matt" below — all five are now resolved.
- **Implemented dual-license (Q3) end-to-end:**
  - `Cargo.toml` workspace `license = "TBD"` removed; per-crate licenses set: `game` = MIT, `compile-api` = AGPL-3.0-or-later.
  - SPDX headers added to all 4 .rs files (`game/src/{main,lib}.rs`, `game/src/plugins/mod.rs`, `compile-api/src/main.rs`).
  - Repo-root `LICENSE-MIT` written, `LICENSE-AGPL` downloaded from gnu.org, `LICENSE.md` written explaining the dual-license rationale + contributor terms + AGPL-only-applies-to-server clarification.
  - CLAUDE.md hard rule on SPDX headers updated from "all files use AGPL" to per-crate split with rationale.
- **Locked distribution + Temple marketing in `design/07-monetization.md`** under new "Marketing decisions (locked 2026-05-01)" section. Includes the leaderboards-deferred call so all three Phase-decisions live in one doc.
- **Domain + GH org availability verified.** RDAP confirmed all 9 candidate domains available; `pledgeandcrown` available on GitHub. Action items moved to Matt's queue.

## What Was Done This Session (2026-04-25 evening)

- **TESS sweep on "Pledge & Crown" — AUTHORITATIVE GREEN.** Matt ran the search in his real Chrome and downloaded the result xlsx (broad sweep, 500-record cap, search term `pledge & crown`). Hard findings: **zero marks anywhere in USPTO contain both "PLEDGE" and "CROWN"**; 5 LIVE PLEDGE-prefix marks in IC 009/041 (none gaming — Port Nexus enterprise software, Indonesian sports foundation, Texas fitness DBA, TN consulting); 52 LIVE CROWN-containing marks in IC 009/041 (23 are bare CROWN — Crown Battery, Crown Audio, Crown Equipment, Crown Financial Ministries, Crown Melbourne casino, Amazon CROWN, etc.; 29 are differently-prefixed compounds). Under 2(d) likelihood-of-confusion analysis the leading distinctive qualifier "PLEDGE &" gives strong commercial-impression separation from bare CROWN and from any other compound. RF Trademark Policy clean (neither word on protected-mark list). **Cleared to:** register `pledgeandcrown.dev`, create GitHub org, file 1(b) ITU in IC 009 + IC 041, commission logo lockup and marketing. Earlier note about "indirect/indexer triangulation" is superseded — authoritative source agrees with indexers. Source xlsx files: `C:\Users\Matt\Downloads\tmSearchResults2026-04-25.xlsx` (50-row narrow) + ` (1).xlsx` (500-row broad).
- **`design/04b-art-deliverables.md` written.** Companion to bible v2.0 + handoff-prompts v1.0 — covers the gaps identified for handing the visual art to a contractor: master asset manifest (~180 deliverables for MVP, prioritized A/B/C, mapped to Acts 1–2 + Borrow Checker), file format spec (PNG-32 / sRGB / single-frame / pivot rules), authoritative animation frame-count table, filename + folder convention promoted out of CLAUDE.md, approval workflow (2-round reference gate, batches of 20, palette-conformance auto-rejection), IP/licensing stance (work-for-hire default + portfolio carve-out), explicit out-of-scope (audio, marketing, 3D).

## Earlier This Session — Initial Audit + Scaffold

- **First-read design audit** — surfaced 8 issues that would have invalidated downstream work; all fixed in-doc:
  1. "Cargo & Crowns" disqualified by Rust Foundation Trademark Policy on the "Cargo" mark.
  2. CI plan (GitHub Actions) contradicted Matt's banned-on-account global rule.
  3. Compile-API spec did not address proc-macro / `[dependencies]` RCE — only `build.rs` and `[build-dependencies]`.
  4. AGPL-3.0 doesn't fit a binary-distributed game (network clause).
  5. Bevy 0.14 pin likely stale at session date.
  6. `wee_alloc` (Day 26 prescription) is unmaintained.
  7. Day 6 wasmtime sandbox + adversarial testing in one day was unrealistic.
  8. Compile-API latency story relied only on Redis source-hash cache; cold-miss `cargo build` on a CX22 is 5–30s.

- **Files updated:**
  - `HANDOFF.md` — RF policy gate, supply-chain RCE risk, dual-license recommendation, fallback-coverage clarified.
  - `CLAUDE.md` — Bevy version note, Hard Rule 2 extended (manifest-control), Hard Rule 6 (naming gate), Hard Rule 7 (no GH Actions).
  - `design/06-mvp-scope.md` — Day 1 swaps GH Actions for `scripts/ci.ps1`+pre-commit; Day 6 widened to Day 6–7; Day 26 swaps `wee_alloc` for `lol_alloc`.
  - `design/05-tech-architecture.md` — Bevy version preamble; three-layer cache (Redis + sccache + pre-warmed `target/` + reference-solution seeding); compile-time RCE defense rewritten as primary control (server-owned `Cargo.toml`, vendored deps, `--offline`, single player file).
  - `design/03-art-style-bible.md` — EDG32 license/credit note added.
  - `design/00-vision.md`, `README.md`, `CLAUDE.md` — title swapped to **Pledge & Crown** across all internal references; layout examples renamed `pledge-and-crown/`; crate name `pledge_and_crown`.

- **Naming decision:** working title locked as **Pledge & Crown** (medieval *pledge* = collateral on a loan, etymologically the Rust borrow-checker concept). The Borrow Checker remains the Act 2 boss's name. "Cargo & Crowns", "Rust & Ruin", and "Ferris's Trial" all rejected on RF Trademark Policy considerations.

## Current State

**Working end-to-end:**
- Title screen → InGame state machine
- Hearthstone Village rendered as 30×20 tilemap (REF-04 atlas)
- Player walks on tiles, WASD/arrows, world bounds clamped, camera follows with viewport clamp
- 5 NPCs spawned at distinct positions, proximity F-press handler
- In-game egui code editor with hand-rolled Heraldic-Code Rust syntax highlighting + tutorial side panel
- 5 missions wired: intro_let_binding (Ferris), double_function (Trait Mage), borrow_preview (Borrow Checker), mut_binding (Smith), if_else_sign (Cartographer). Each has a 100-200 word tutorial.
- Live HTTP round-trip game ↔ axum compile-API on `127.0.0.1:7878`
- Per-encounter pattern grader with 3 success/fail flavor strings each
- Mission progression tracking: cleared encounters show `[done]` in interaction prompt
- 46-test audit harness gates every commit (assets, registry, contract, bevy_smoke, grader unit, HTTP integration)

**Stubbed (intentional v0):**
- Compile-API grader is pattern-matching, not real `cargo build` + wasmtime sandbox. Real implementation per `design/05-tech-architecture.md` §2 (server-owned `Cargo.toml`, vendored deps, `--offline`, seccomp container) is the next big chunk.
- 2 NPCs (Smith, Cartographer) use `player.png` as placeholder art until their REFs land.
- The 8-encounter pre-compiled stub fallback (per design/05) doesn't exist; if API is offline, game shows `[client error]`.
- LogPlugin warning at startup (Bevy's logger collides with our pre-init tracing subscriber). Non-functional.

**Not yet done:**
- Acts 3–10 missions (current 5 cover the Act 1 prelude — variables/functions/refs/mut/branches)
- Combat system, party, persistence, audio
- Save/load
- Web build (wasm-bindgen unused)
- Mobile (Tauri not scaffolded)

## Blocking Issues

None blocking forward motion. Open Matt-only action items:

- Register `pledgeandcrown.dev` + `.com` at Cloudflare (~$22/yr).
- File 1(b) ITU at USPTO TEAS in IC 009 + IC 041 (~$700, optional).

## What's Next (prioritized)

1. **Real cargo-build + wasmtime sandbox** for compile-API per `design/05-tech-architecture.md` §2. The pattern grader is fine for the demo loop but not the long-term curriculum. Server owns `Cargo.toml`, vendored deps, `cargo --offline`, single player file in `src/lib.rs`, exec the result inside wasmtime with WASI. Big chunk; design is already locked.
2. **Mission completion dialogue** — when player talks to a cleared NPC, show a "thanks for solving it, here's what we just covered" follow-up before re-opening the editor. Currently the [done] tag is all that changes.
3. **More missions / Act 1 content** — extend curriculum past the prelude. Suggested next: `loop_break`, `match_option`, `vec_iter`, `struct_basic`, `tuple_destructure`. Each addition is: grader arm + 4-7 unit tests + 1 HTTP test + Mission entry + canonical_solution + NPC entry + tutorial. Audit harness catches drift instantly.
4. **Polish** — fix the LogPlugin/tracing-subscriber collision (drop our pre-init `fmt().init()`, let Bevy's LogPlugin own logging); add `bevy/bevy_picking` feature to silence the bevy_egui warning.
5. **8 pre-compiled stub fallback** — when the live API is unreachable, ship a hard-coded "for the canonical solution this would compile" verdict so the demo isn't blocked by a server hiccup.
6. **Save/load** — `MissionProgress` is in-memory; persist to `~/.local/share/pledge-and-crown/save.bincode` via `bincode 2` (already in deps).

## Notes for Next Session

- **Read this file first.** Global Matt rule.
- **Cargo target dir lives on G:** (`G:/cargo-target/pledgeandcrown` per `.cargo/config.toml`). Don't run cargo from anywhere assuming `target/` is in-tree — it isn't. J: drive was full at 0.1 GB; the move was forced. Do not delete the G: directory.
- **Run the game:** terminal 1 `cargo run -p pledge_compile_api`, terminal 2 `cargo run -p pledge_and_crown`. Title screen → Space → 5 NPCs to talk to via F.
- **Mission/grader contract is enforced by tests.** When adding a new mission: add the grader arm in `compile-api/src/grader.rs`, add a Mission entry in `game/src/plugins/mission.rs`, add an NPC to `NPC_ROSTER` in `game/src/plugins/npc.rs`, add a canonical_solution arm in `game/tests/contract.rs`. The audit suite will fail loudly if any of these drift.
- **Tutorial structure** is `## Concept` / `## Syntax` (with ``` fences) / `## Task` / `## Hint`. Keep each 100-200 words. The `every_tutorial_meets_minimum_substance` test enforces non-emptiness, headers, and code fences.
- **Starter codes must NOT contain grader needles.** The audit suite caught the original 3 missions trivially passing because their comments contained the solution tokens. Use `_todo` placeholders instead. The `unmodified_starter_code_does_not_trivially_win` test enforces this.
- **egui version diamond:** `bevy_egui 0.39` ships egui 0.33; `egui_code_editor 0.2` (latest as of 2026-05-02) pins egui 0.34. Unfixable upstream. We hand-rolled the syntax highlighter via `egui::TextEdit::layouter` + `LayoutJob`. If a newer `bevy_egui` ships with egui 0.34 support, swap back.
- **`ScalingMode` is in `bevy::camera`, NOT prelude.** `OrthographicProjection` and `Projection` are prelude. Camera setup pattern: `Projection::Orthographic(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: 180.0 }, ..default_2d() })`.
- **`init_state` requires `bevy::state::app::StatesPlugin`.** `MinimalPlugins` doesn't include it; `DefaultPlugins` does. Smoke tests using `MinimalPlugins` must explicitly add `StatesPlugin`.
- **`bevy_ecs_tilemap` 0.18 uses `TilemapAnchor`** for centring; the older `get_tilemap_center_transform` helper is `#[deprecated]` and clippy with `-D warnings` will reject it.
- **PS 5.1 ASCII gotcha:** all `.ps1` files in `scripts/` must be ASCII-only — no em-dashes, no smart quotes. PS 5.1 reads UTF-8-without-BOM as Windows-1252 and garbles non-ASCII.
- **Pre-commit hook is real CI** (`scripts/pre-commit.sh` → `scripts/ci.ps1`): fmt + check + clippy `-D warnings` + test. ~40s warm rebuild, ~3-7 min cold rebuild on G:. To bypass for WIP only: `git commit --no-verify` (and global rules say don't).
- **Commit messages with embedded double-quotes** → use `git commit -m "..." -m "..."` (multiple `-m` flags) instead of `-F` heredocs. PowerShell heredocs don't escape `"` cleanly through to git's argv parser.
- **Repo is at `pledgeandcrown/pledgeandcrown` on GitHub** (org-owned, transferred 2026-05-01). Do NOT push back to `suhteevah/pledgeandcrowns` — that URL 404s now.
- **Compile API design hard rule:** player input never touches `Cargo.toml`. Server owns the manifest. v0 grader is pattern-matching to defer the security work; when implementing the real `cargo build` path, follow `design/05-tech-architecture.md` §2 to the letter — vendored deps, `--offline`, single player file in `src/lib.rs`.
- **GitHub Actions is banned on Matt's account** (global rule). Local CI only.

## Critical path to MVP (30 days)

1. **Day 1–2: Trademark + naming.** Working title locked: **Pledge & Crown**. Action: USPTO TESS + final RF policy sanity check, register `pledgeandcrown.dev`, create GitHub org + repo, scaffold local CI (`scripts/ci.ps1` + pre-commit + optional self-hosted runner — **no GitHub Actions**).
2. **Day 3–5: Style bible reference pass.** Use `design/04-art-handoff-prompts.md` to generate the 10 reference images. Lock the look. Do not proceed to bulk asset generation until references are approved.
3. **Day 6–10: Bevy scaffold + server compile API.** Project boots, shows a town tile, opens a `bevy_egui` code editor, ships player code to the compile server, runs WASM in sandbox, returns result. Verbose tracing throughout.
4. **Day 11–18: Act 1 implementation.** Variables, mutability, basic functions. Tutorial village. First two enemies.
5. **Day 19–25: Act 2 implementation.** Ownership, references, lifetimes. Bridge zone. The Borrow Checker boss.
6. **Day 26–28: Polish, juice, sound, web build.**
7. **Day 29: Deploy demo to itch.io as free web build. Email capture.**
8. **Day 30: Launch.** r/rust, Hacker News, Bluesky, Twitter, Rust gamedev Discord.

## Open questions for Matt

1. ~~**Naming.**~~ Resolved 2026-04-25: **Pledge & Crown**.
2. ~~**Leaderboards in MVP?**~~ Resolved 2026-05-01: **deferred to v1.1**. Backend dep doesn't move the needle on day-30 itch launch; revisit after first $5K of revenue.
3. ~~**Licensing.**~~ Resolved 2026-05-01: **dual-license** — `game/` crate is MIT, `compile-api/` crate is AGPL-3.0-or-later. SPDX headers added to all .rs files; `LICENSE-MIT`, `LICENSE-AGPL`, and `LICENSE.md` (rationale + contributor terms) committed at repo root.
4. ~~**Distribution.**~~ Resolved 2026-05-01: **itch.io free demo (day 30) → itch.io paid early-access (day 90) → Steam paid (day 180)**. Locked in `design/07-monetization.md` §"Marketing decisions".
5. ~~**HolyC Temple in MVP marketing?**~~ Resolved 2026-05-01: **surprise reveal**, not day-1 tease. First public mention is the deluxe-edition pricing on the Phase-3 Steam page (~day 180). Locked in `design/07-monetization.md` §"Marketing decisions".

**No remaining design-decision blockers. All open items below are action items, not decisions.**

## Action items for Matt (require Matt-only access)

- **Register `pledgeandcrown.dev` and `pledgeandcrown.com`** at Cloudflare (~$22/yr total). Both confirmed available via RDAP 2026-05-01.
- ~~**Create GitHub org / repo / transfer**~~ — done 2026-05-02. Repo at `https://github.com/pledgeandcrown/pledgeandcrown` (transferred from personal namespace + renamed). Local `origin` re-pointed to the org URL. Note: org policy forbids fine-grained PATs with lifetime > 366 days from accessing org resources, so `gh api orgs/pledgeandcrown/...` returns 403 unless the token is reissued with a shorter lifetime.
- **File 1(b) intent-to-use trademark** for "Pledge & Crown" in IC 009 + IC 041 via USPTO TEAS (~$700, 2× $350). Optional but recommended now that TESS is GREEN.
- ~~**Generate the 10 reference art images**~~ — done 2026-05-01. Delivered as JSX/ASCII-grid bundle in `design/art/`. Bulk asset gen is now unblocked.

## Known risks

- **Server-side compile API is a single point of failure.** Mitigations: three-layer cache (Redis source-hash + `sccache` + per-encounter pre-warmed `target/`), pre-compiled stub fallback covering **all 8 MVP encounters**, AWS Lightsail $5 cold-standby with low DNS TTL.
- **Compile-API supply-chain RCE risk.** Hard-locked in `05-tech-architecture.md` §2: server owns `Cargo.toml`, vendored deps, `cargo --offline`, single player file. Container hardening is defense-in-depth, not the primary control.
- **Bevy version churn.** Pin at scaffold time, document upgrade path, budget a week per major bump.
- **Art consistency drift.** Reference-image gate, batch-of-20 review, style bible amendments per pass.
- **Curriculum scope creep.** Doc fixes the curriculum. Resist mid-MVP additions.

## Where to look next

- Curriculum → `design/01-curriculum.md`
- Art → `design/03-art-style-bible.md` and `prompts/`
- Tech → `design/05-tech-architecture.md`
- Compile-API security → `design/05-tech-architecture.md` §2 (compile-time RCE defense)
- "What should I build today" → this file, the critical path above
