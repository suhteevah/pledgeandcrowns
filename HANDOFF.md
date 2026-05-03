# HANDOFF.md

## Last Updated
2026-05-03 — wasm web build infrastructure live (game boots in browser, image-binding still broken); 22 NPCs with art (5 batch 1 + 5 batch 2 + 5 batch 3 + 3 legacy migrated + Borrow Checker + Trait Mage tilesheet refs); two input-bleed bugs fixed; Matt called out the buffet-vs-curriculum problem at session end.

## Project Status
🟡 In progress. Native game still playable end-to-end with 21 missions and the lean roster live. Web build boots in Chrome via `cargo run -p web-serve` but image-backed sprites don't bind to GPU. Curriculum-gating designed but not implemented (next session's first task).

## What Was Done This Session (2026-05-02 → 2026-05-03)

### Curriculum + cast (continued)
- **art-lead batch 2** (`6d99ef5`) — Oracle (`match_option`), Herald (`struct_basic`), Twin (`tuple_destructure`), Tinker (`while_loop`), Heraldic Sage (`enum_match`). Hand-authored ASCII grids → JSX → PNG → game asset → registry wiring → manifest extension. **Note:** these were LLM-imagined, not driven through claude.ai/design (see "Notes for Next Session").
- **art-lead batch 3** (`a8ba270` + merge `3b9f538`) — Forgewright (`borrow_mut`), Linguist (`string_vs_str`), Pilgrim (`option_unwrap_or`), Drillmaster (`for_in_range`), Reckoner (`closure_basic`). Same recipe; same caveat.
- **voice-editor batch 1 art alignment** (`4872b44` + merge `f6b1825`) — Smith, Cartographer, Trait Mage, Bellringer, Cooper. Tutorials and grader/stub flavor tightened to match the visual specs. Server↔stub byte-identical, byte-parity audit happy.
- **voice-editor batch 2 art alignment** (`a1e1bbe` + merge `03bfa71`) — Oracle, Herald, Twin, Tinker, Heraldic Sage. The agent caught a fact I had wrong (Heraldic Sage owns `enum_match`, not `iter_map_collect` — Alchemist owns that).

### Infra cleanup + dog-fooding
- **rust-lead P2** (`c8f1cd9` + merge `4d9bce8`) — wasmtime `Config` deny-list (9 wasm proposals explicitly disabled with dependency-chain comments) + cancellable cargo watchdog (mpsc `recv_timeout` instead of poll-loop sleep; cargo_grader slow suite went 1.05s → 0.40s).
- **per-worktree CARGO_TARGET_DIR isolation** (`76bbb54`) — `scripts/ci.ps1` detects worktree checkouts via `.git`-is-a-file (vs main where it's a directory) and pins `CARGO_TARGET_DIR=G:/cargo-target/pledgeandcrown-wt-<basename>`. Closed the deferred fix from `.claude/audit-gaps.md`. Pre-commit hooks invoke ci.ps1 so any worktree commit gets isolation for free.
- **legacy sprite migration** (`9452d17`) — `sprites/player.png`/`ferris.png`/`borrow_checker.png` → `sprites/{category}/{name}_{frame}.png` per the bible convention. `git mv` so history's preserved. Manifest table updated.
- **`scratch/.git-commit-msg.txt` gitignored** (`5d928f8`) after one accidentally got swept up by `git add -A`.

### Pure-Rust render pipeline (the React/Babel/Python kill)
- **`tools/render-refs/`** (`ffe4eaf`) — pure-Rust binary that bakes `design/art/palette.js` + every `design/art/refs/ref-*.jsx` into the binary at compile time via `build.rs` + `include_str!`. Parses ASCII grids, looks up palette codes, writes PNG-32 to `design/art/refs/png/`. Single command: `cargo run -p render-refs --release`. Renders all 18 grid-shaped refs in ~20ms. **Removed:** `scripts/render-refs-inline.html` (2717-line React+Babel CDN harness), `scripts/render-refs.py`, `scripts/render-refs.md`. Pixel data identical to the legacy pipeline; only PNG encoder metadata differs.
- **art-lead role file tightened** (`8be31ea`) — claude.ai/design via the `claude-in-chrome` MCP is now the ONLY upstream design source. Hard rule banning hand-authored grids from text descriptions, Wraith fallback for CDN issues, Google image search, or any side-renderer. The previous role-file wording let agents rationalize hand-authoring as a "fast path"; that loophole's closed.

### Wasm web build (partial — see Blocking Issues)
- **`scripts/web-build.ps1` fixed** (`42e6f7f`) — was hardcoded to `target/...` but `.cargo/config.toml` redirects to `G:/`. Now resolves the target dir via `cargo metadata --no-deps`. Writes a starter `web/index.html` if absent.
- **`tools/web-serve/`** — pure-Rust local dev server (axum + ServeDir, ~50 LOC, defaults to `127.0.0.1:8080`, env vars override). Replaces the previous `python -m http.server` hint. Same workspace; same `cargo run` path.
- **bevy features for wasm** (`42e6f7f` + `2d14af8`) — added `webgl2` (without it `wgpu::Instance::new` panics with `unreachable`), `web` (Bevy's wasm-helpful default-plugins toggle), `bevy_picking` (silences egui warning).
- **AssetPlugin override** (`2d14af8`) — `AssetMode::Unprocessed` forced because Bevy's `web` feature flips on `AssetMode::Processed` which expects `.meta` sidecars we don't ship.
- **ImagePlugin::default_nearest()** — pixel-art sampler, dodges some WebGL2 sampler quirks.
- **TITLE_SCALE fix** (`2d14af8`) — was 2.0 = 4× viewport. Corrected to 0.5 to fit the FixedVertical(180) camera. Bug existed on native too; was never visible because the parchment background filled the cropped middle 25%.
- **`web/`, `web/pkg/`, `web/assets/`** gitignored except for `web/index.html` (real source).

### Input-bleed bugs (Matt caught playing native)
- **`E` no longer toggles when editor is open** (`2d14af8`) — typing `e` in your code closed the window mid-edit. E now opens-only-when-closed, Esc closes-when-open.
- **WASD bails out of `move_player` when `editor.open`** — typing WASD walked the player around in the background. Same gate now applied to `handle_interact_key` so typing `f` doesn't fire NPC interactions either.

## Current State

### Working end-to-end (native)
- Title screen → InGame state machine, Esc-only editor close
- Hearthstone Village rendered as 30×20 tilemap (REF-04 atlas)
- Player walks, world bounds clamped, camera follows with viewport clamp
- 21 NPCs spawned at distinct positions; 17 with hand-authored art, 4 still on `SPRITE_PLAYER` placeholder (Quartermaster, Auditor, Chronicler, Alchemist)
- In-game egui editor with hand-rolled Rust syntax highlighting + 4-section tutorial side panel; input correctly isolated from world while open
- 21 missions live with full Acts 1 prelude + Act 2 ownership coverage:
  - Act 1: intro_let_binding, double_function, borrow_preview, mut_binding, if_else_sign, loop_break, match_option, struct_basic, vec_iter, tuple_destructure, while_loop
  - Act 2: borrow_mut, string_vs_str, option_unwrap_or, for_in_range, closure_basic, slice_basic, result_question_mark, derive_debug, iter_map_collect, enum_match
- Live HTTP round-trip game ↔ axum compile-API on `127.0.0.1:7878`
- Per-encounter pattern grader with flavor strings; offline stub fallback in `compile_client.rs` when API unreachable
- Mission completion dialogue + persistence via `bincode 2` with atomic tmp+rename writes

### Wasm web build
- ✅ Builds cleanly: `powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1`
- ✅ wasm-bindgen produces `web/pkg/pledge_and_crown.{js,wasm}` (~36 MB unoptimized)
- ✅ Local server: `cargo run -p web-serve --release`, default `127.0.0.1:8080`
- ✅ wgpu inits via WebGL2 in Chrome
- ✅ Bevy app boots; state machine runs; plugins all build
- ✅ HTTP fetch on assets resolves correctly (`/assets/sprites/title.png` returns 200)
- ✅ Plain colored sprites render (`Sprite::from_color(...)`)
- ❌ **Image-backed sprites don't bind to GPU.** title.png loads, asset is in registry, sprite is spawned with the right Handle, but no pixels draw. Likely cascade from `Feature TEXTURE_BINDING_ARRAY is not supported` (bevy_egui complaint) — or .meta-503 fallout, or an asset-format mismatch. Diagnosed via diagnostic clear-color (red shows) + magenta probe sprite (shows) — the sprite renderer works, the image-binding stage doesn't.

### Audit harness
- ~144 fast tests + 8 slow `#[ignore]`d (5 cargo_grader, 3 http_real)
- Server↔stub byte-parity test catches both missing-arm AND wording-drift failures
- Cargo grader environment is properly isolated: `.env_clear()` + per-sandbox `CARGO_TARGET_DIR`
- `cargo_grader` watchdog uses cancellable mpsc channel pattern; slow tests went 1.05s → 0.40s
- wasmtime `Config` explicitly denies 9 wasm proposals (defense-in-depth)

### Stubbed / known limitations (intentional)
- Compile-API grader is still pattern-matching for `/compile`. The real `cargo check`-based path lives at `/compile-real` (gated, slow, not yet wired into the in-game client).
- Wasm execution layer (`wasm_runner`) exists but isn't yet wired into `/compile-real`'s response — `/compile-real` returns cargo's verdict only, no per-encounter wasm-stdout flavor.
- 4 NPCs still use `SPRITE_PLAYER` placeholder.
- `MissionProgress::save_to` panics on wasm (no fs); needs `#[cfg(not(target_arch = "wasm32"))]` gate + localStorage path. Logged as warn each frame currently.

## Blocking Issues

1. **Wasm image-binding bug** — the immediate blocker for an itch.io demo. The wasm binary boots and renders the camera + colored sprites, but image-backed sprites don't draw despite assets fetching successfully. Needs Bevy 0.18 + WebGL2 + bevy_egui texture-pipeline expertise. Likely cascade from missing TEXTURE_BINDING_ARRAY feature support, or asset-format mismatch in the sprite render pipeline. Best handled by a focused subagent with web-search budget OR by checking the Bevy issue tracker / examples for a known wasm sprite-rendering pattern we're missing.

2. **Curriculum-vs-buffet (Matt's session-end realization)** — all 21 NPCs are simultaneously interactable in an open village. The player can talk to the Heraldic Sage about enums before they've ever bound a `let`. There is no required tutorial sequencing, no first-area gating, no "you have to talk to Ferris first." Design TBD; minimum viable is a per-mission `prereq: Option<&str>` field + `handle_interact_key` gate + visual cue for the next NPC. See "What's Next" item 1.

## What's Next (prioritized)

1. **Curriculum gating + linear progression for Act 1.** Add `prereq: Option<&'static str>` to `Mission`. Wire prereqs in `MissionRegistry::default()` matching the curriculum order. `handle_interact_key`: if prereq not cleared, show `[locked — clear <prereq npc> first]` instead of opening editor. Add registry tests: first mission has no prereq, every prereq references a valid id, no cycles, all missions reachable. Bonus: a small persistent UI element saying "next mission: [name] · find [npc name]" so players don't wander confused. Don't paint a whole NPC differently yet (art is placeholder); the locked dialogue is enough cue.
2. **Wasm image-binding bug.** Either dispatch a research subagent (web-search Bevy 0.18 wasm sprite issues, check Bevy examples repo for a known-working wasm sprite setup, try `bevy_image` features explicitly, try the WGPU `OES_texture_float_linear` extension dance, etc.) OR open a Bevy issue with our minimal repro. Rough hypothesis order: (a) bevy_egui's `TEXTURE_BINDING_ARRAY` failure cascading, (b) Image asset's texture format isn't supported by WebGL2 sprite pipeline, (c) sprite renderer needs an explicit WebGL2 feature flag we missed.
3. **`MissionProgress::save_to` wasm gate** — `#[cfg(target_arch = "wasm32")]` no-op or localStorage path so the autosave warn doesn't fire every frame on web.
4. **Real art for the 4 remaining placeholder NPCs.** This time through claude.ai/design via `claude-in-chrome` per the role file. The 17 existing placeholder sprites also need redesign through the same pipeline; flagged in `.claude/agents/art-lead.md` as "placeholders pending real design through the canonical path."
5. **Wire wasmtime exec into `/compile-real`.** Build for `wasm32-wasip1`, run via `wasm_runner`, capture stdout, return as the per-encounter flavor. Then cut the in-game compile_client over from `/compile` to `/compile-real`.

## Notes for Next Session

- **Read this file first.** Global Matt rule.
- **Cargo target dir on G:** (`G:/cargo-target/pledgeandcrown` per `.cargo/config.toml`). For worktrees, `scripts/ci.ps1` now isolates to `G:/cargo-target/pledgeandcrown-wt-<basename>` automatically. If you run raw `cargo test` outside the script in a worktree, set the env var manually or you'll re-trigger the stale-binary trap.
- **Run native game:** terminal 1 `cargo run -p pledge_compile_api`, terminal 2 `cargo run -p pledge_and_crown`. Title screen → Space → 21 NPCs to talk to via F.
- **Run wasm build + serve:** `powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1` then `cargo run -p web-serve --release`. Tab the dev URL (default `127.0.0.1:8080`). The game boots but title sprite won't draw — that's the open blocker.
- **Render PNG refs:** `cargo run -p render-refs --release`. Pure Rust, no browser, no Python. Recompiles automatically when any `.jsx` or palette source changes.
- **The lean team is live.** `.claude/agents/{rust-lead,art-lead,voice-editor}.md` are the standing roles; `dormant/{security-reviewer,audit-extender}.md` activate only on trigger. CLAUDE.md has the team section. Routine review/voice/art passes can be dispatched in parallel via worktrees.
- **art-lead is now strict.** claude-in-chrome MCP into Matt's logged-in claude.ai/design is the ONLY upstream design source. The 17 NPC sprites currently in the repo from batches 1-3 are LLM-imagined hand-authored grids and need redesign through the canonical path. The role file has explicit no-fallback wording with the batch-3 incident as the cautionary tale.
- **Curriculum is a buffet right now**, per Matt's session-end call. Item 1 of "What's Next" is the highest-leverage next move — without progression gating the game doesn't teach.
- **Input-bleed pattern:** any gameplay input that's also a Rust character (E, F, W, A, S, D) needs to be gated on `editor.open`. Three sites already gated; future systems should follow the pattern.
- **Egui version diamond** (still): `bevy_egui 0.39` ships egui 0.33; `egui_code_editor 0.2` pins egui 0.34. Hand-rolled syntax highlighter via `LayoutJob` lives in `game/src/plugins/editor.rs`. If a newer `bevy_egui` ships with egui 0.34, swap back.
- **Pre-commit hook is real CI** (`scripts/pre-commit.sh` → `scripts/ci.ps1`): fmt + check + clippy `-D warnings` + test. ~40s warm rebuild on G:.
- **Repo is at `pledgeandcrown/pledgeandcrown` on GitHub** (org-owned). Earlier in the session the remote drifted back to `suhteevah/pledgeandcrowns`; verified and re-pointed. `git remote -v` should show the org URL.
- **Compile API design hard rule:** player input never touches `Cargo.toml`. Server owns the manifest. The cargo_grader environment is now properly isolated (`.env_clear()` + sandbox-local CARGO_TARGET_DIR + 64 KiB source cap on both routes).
- **GitHub Actions is banned** (global rule). Local CI only.

## Action items for Matt (require Matt-only access)

- Look at the wasm canvas in Chrome (`http://127.0.0.1:8080/` after `web-build.ps1` + `web-serve`). Confirm the render-refs Rust binary's PNG output looks pixel-identical to the legacy pipeline's outputs (both are committed; `git diff` shows them as modified by the migration commit but pixels should match).
- Decide curriculum gating shape before next session if you have a preference: strict linear (mission N requires mission N-1) vs branching (e.g. struct_basic and enum_match both unlock after if_else_sign). Strict linear is simpler and matches Act 1 prelude well; branching is more flexible for later acts.

## Where to look next

- Curriculum → `design/01-curriculum.md`
- Art canon → `design/03-art-style-bible.md` and `design/04b-art-deliverables.md`
- Tech → `design/05-tech-architecture.md`
- Compile-API security → `design/05-tech-architecture.md` §2 (compile-time RCE defense)
- Team roles → `.claude/agents/`
- Audit gap log → `.claude/audit-gaps.md`
- Rust render pipeline → `tools/render-refs/`
- Web build → `scripts/web-build.ps1`, `tools/web-serve/`, `web/`
- "What should I build today" → this file, item 1 of "What's Next"
