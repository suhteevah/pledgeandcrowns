# HANDOFF.md

## Last Updated
2026-05-03 (session 2) — autonomous v1 roadmap complete: curriculum gating live, wasm rendering bug diagnosed + fixed, wasm save no-panic, wasm-canonical stub grader, win-condition epilogue, title-screen prompt + progress indicator. Remaining v1 work is Matt-blocked (NPC art via claude.ai/design) or explicitly post-MVP (wasmtime/compile-real wiring).

## Project Status
🟢 v1 functionally complete pending visual verification of the wasm rendering fix. Native: 21 missions, full progression, epilogue. Web: builds cleanly, gameplay loop wired (stub-canonical grader, in-memory progress).

## What Was Done This Session (2026-05-03 — session 2, autonomous mode)

Six commits, end-to-end on the "What's Next" roadmap from session 1's handoff plus completeness wins.

### Curriculum gating + linear progression (item 1) — `d679886`
- `Mission.prereq: Option<&'static str>` + `is_unlocked()` helper.
- `MissionRegistry::default` post-build chain: each mission requires the previous one cleared. Strict-linear, branching deferred until later acts have parallel paths.
- `MissionRegistry::next_locked()` / `find()` helpers.
- `handle_interact_key` no-ops F-press on locked NPCs (debug-logged).
- `draw_interaction_prompt` renders `[locked — clear <prev npc> first]` when prereq isn't met.
- Persistent top-left HUD pin: `next: <id> · find <npc name>`.
- 4 new registry tests: first_mission_has_no_prereq, every_prereq_resolves, prereq_chain_is_acyclic, every_mission_is_reachable_from_a_root. All pass.

### Wasm save gate (item 3) — `a69cf03`
- `MissionProgress::save_to` and `load_from` are now `cfg(not(target_arch = "wasm32"))` with no-op shims on wasm. SaveFile/SAVE_VERSION/SAVE_FILENAME/validated_save_path also gated; serde import gated.
- Eliminates the per-frame fs error path on web. In-memory progress works for the tab's lifetime.
- localStorage persistence is the planned next step but blocked behind the wasm rendering fix landing visually.

### Wasm rendering bug diagnosed + fixed (item 2) — `f0bd47b`
- Root cause (from research subagent + `bevy_egui` 0.39 source inspection): `EguiPlugin::default()` sets `bindless_mode_array_size: NonZero::new(16)`, which requests `TEXTURE_BINDING_ARRAY` from wgpu. WebGL2 cannot provide that feature, so wgpu refuses to create the bind-group layout. The failure cascaded into the wider Image GPU upload path, killing image-backed sprite rendering on wasm. Color sprites (1×1 white texture, no asset binding) kept working — exactly the observed symptom.
- Fix: `EguiPlugin { bindless_mode_array_size: None, ..default() }` in `editor.rs`. Falls back to per-texture bind groups, which WebGL2 supports. Native works either way; doing this unconditionally keeps the build target-agnostic.
- Belt-and-suspenders: switched `AssetPlugin.meta_check` to `AssetMetaCheck::Never` (Bevy issue #18002 / PR #10623) — suppresses the HEAD probes the asset server otherwise issues for every `<asset>.meta`, cleaning the network panel and removing a class of first-frame asset binding stalls.
- Confidence: high on the cause (exact error string match against bevy_egui source). Visual verification deferred to Matt — the wasm build compiles and runs, but the title sprite either appearing or not appearing in Chrome is the actual confirmation.

### Win-condition epilogue — `293aad4`
- `EpilogueView` resource + `draw_epilogue_panel` system. Auto-shows center-screen when `cleared_count == registry.missions.len()`. Esc dismisses it for the rest of the session.
- Gated against the editor and per-mission completion panels so the three flows don't visually collide.
- Voice is the realm itself addressing the player: "The realm pledges its crown." Tonally heraldic, no spoiler for future acts.
- Closes the obvious gap: previously the game just stopped having new content with no acknowledgement.

### Title-screen prompt + progress indicator — `c124fb9`
- Egui overlay at bottom-center of the title state: "Press [Space] to begin" + a tonally tiered progress line:
  - 0 cleared: "a Rust kingdom awaits its pledge"
  - mid-game: "your pledge so far: N / 21 encounters cleared"
  - all done: "the realm has crowned you - N / N cleared"
- Returning players now have both a clear way in and a visible sign of where they left off.

### Wasm-canonical stub grader — `e72cd7a`
- The wasm build was structurally broken: reqwest from a browser can't reach `localhost:7878` (CORS + on a hosted demo no server exists), so every Compile click went through the stub fallback, which intentionally did NOT mark cleared on native to make players re-run against the real grader. On wasm this meant: missions graded correctly, but the persist step never fired, so progression was impossible.
- Cfg-gated the network path off on wasm. wasm `dispatch_pending_compile` runs `stub_verdict` synchronously and treats passes as canonical clears.
- Renamed `CompileOutcome.from_server` → `persist_clear` since the real distinction is "should this update MissionProgress", not "did this come over the wire".
- Added `format_stub_wasm` (no "[offline mode]" banner — the stub IS the grader on web, dressing it up as a fallback would mislead).
- Native behavior unchanged: live API still authoritative, fallback stub still doesn't mark cleared.
- Together with the bindless fix, this completes the wasm demo loop end-to-end.

## Current State

### Working end-to-end (native)
- Title screen with Space-to-start prompt + progress indicator → InGame state machine, Esc-only editor close.
- Hearthstone Village rendered as 30×20 tilemap (REF-04 atlas).
- Player walks, world bounds clamped, camera follows with viewport clamp.
- 21 NPCs spawned at distinct positions; 17 with hand-authored art, 4 still on `SPRITE_PLAYER` placeholder (Quartermaster, Auditor, Chronicler, Alchemist).
- In-game egui editor with hand-rolled Rust syntax highlighting + 4-section tutorial side panel.
- 21 missions live with full Acts 1 prelude + Act 2 ownership coverage.
- **Curriculum-gated:** strict-linear Act 1+2 progression; locked NPCs show "[locked — clear X first]"; "next mission" HUD pin always visible.
- Live HTTP round-trip game ↔ axum compile-API on `127.0.0.1:7878`.
- Per-encounter pattern grader with flavor strings; offline stub fallback when API unreachable (does NOT mark cleared on native — by design).
- Mission completion dialogue + persistence via `bincode 2` with atomic tmp+rename writes.
- **Win condition:** epilogue panel auto-shows after the 21st mission clears.

### Working end-to-end (wasm) — pending Matt's visual confirmation
- Builds cleanly: `powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1`.
- wasm-bindgen produces `web/pkg/pledge_and_crown.{js,wasm}`.
- Local server: `cargo run -p web-serve --release`, default `127.0.0.1:8080`.
- wgpu inits via WebGL2 in Chrome.
- Image binding **should now work** (bevy_egui bindless disabled). Visual verification: load `127.0.0.1:8080/`, expect title art (REF-10) visible, then Space → village + sprites.
- Compile button uses the in-process stub as canonical grader (no network). Passes mark cleared. Progression works for the tab's lifetime.
- Save persistence: in-memory only (localStorage path is the next-step enhancement).

### Audit harness (unchanged from session 1)
- ~144 fast tests + 8 slow `#[ignore]`d (5 cargo_grader, 3 http_real). All passing.
- Plus 4 new registry tests covering the prereq invariants.
- Server↔stub byte-parity test still happy.

### Stubbed / known limitations (intentional)
- Compile-API still pattern-matches at `/compile`. The real `cargo check`-based path lives at `/compile-real` (gated, slow, not yet wired into the in-game client). See "What's Next" item 1 below — out of v1 scope.
- Wasm execution layer (`wasm_runner`) exists but isn't wired into `/compile-real` — no per-encounter wasm-stdout flavor yet.
- 4 NPCs use `SPRITE_PLAYER` placeholder; 17 are LLM-imagined hand-authored grids that need redesign through the canonical claude.ai/design path. Matt-blocked.
- Wasm progress is in-memory only (cleared on tab close). localStorage path is post-v1.

## Blocking Issues

1. **Visual verification of the wasm rendering fix** — Matt-only. Run `scripts/web-build.ps1`, then `cargo run -p web-serve --release`, then Chrome → `http://127.0.0.1:8080/`. If the title art is visible: high-confidence diagnosis was right and v1 is browser-shippable. If still blank: the secondary candidate (Image asset texture format unsupported by WebGL2 sprite pipeline) is on deck — disambiguation experiment is to temporarily remove `EguiPlugin` from the wasm build entirely and re-run; if sprites then appear, the bindless fix is the right path and just needs deeper plumbing.

2. **NPC art (4 placeholders + 17 redesigns)** — Matt-only. The 4 remaining placeholders (Quartermaster, Auditor, Chronicler, Alchemist) and the 17 hand-authored sprites from sessions 1-2 all need to be redesigned through `claude.ai/design` via the `claude-in-chrome` MCP per the locked art-lead role file. This is the canonical path and the current art-lead role explicitly bans hand-authored grids from text descriptions.

## What's Next (post-v1, prioritized)

1. **`compile-real` + wasmtime exec wiring (was item 5 last session).** Compile player source to `wasm32-wasip1`, run via `wasm_runner`, capture stdout as the per-encounter flavor. Cut native compile_client over from `/compile` → `/compile-real`. Substantial: server needs `rustup target add wasm32-wasip1`, cargo skeleton changes from `[lib]` to `[[bin]]`, and the per-request cargo build adds 5-10s latency vs the current ~100ms pattern grading. Honest grading > pattern grading but not MVP-blocking.

2. **localStorage save persistence on wasm.** With the rendering fix in, the tab-lifetime in-memory progress is the obvious next gap. Use `web_sys::window().local_storage()` + base64-encoded bincode bytes. Should be a small diff against `progress.rs`.

3. **Audio pass.** `bevy_kira_audio` is in deps but no assets and no playback wiring. Title music + per-NPC interaction stings + win-condition fanfare. Would need sound design (Matt-territory or licensed pack).

4. **Mobile / Tauri 2.0 wrapper.** Per `design/05-tech-architecture.md`. Separate workstream — Matt's `tauri-frontend` skill applies.

5. **Acts 3+ curriculum.** v1 covers Acts 1+2. New acts need new missions, new NPCs, new art. Out of v1 scope per the locked design.

## Notes for Next Session

- **Read this file first.** Global Matt rule.
- **Run native game:** terminal 1 `cargo run -p pledge_compile_api`, terminal 2 `cargo run -p pledge_and_crown`. Title screen → Space → 21 NPCs to talk to via F.
- **Run wasm build + serve:** `powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1` then `cargo run -p web-serve --release`. Tab `127.0.0.1:8080`. Title sprite *should* now render — pre-fix it was the open blocker.
- **Render PNG refs:** `cargo run -p render-refs --release`. Pure Rust, no browser, no Python.
- **Cargo target dir:** still `G:/cargo-target/pledgeandcrown` per `.cargo/config.toml`. Worktree isolation happens automatically via `scripts/ci.ps1`.
- **The lean team is live.** `.claude/agents/{rust-lead,art-lead,voice-editor}.md`. Routine review/voice/art passes can be dispatched in parallel via worktrees. art-lead is gated to claude.ai/design — DO NOT let it hand-author grids again.
- **Pre-commit hook is real CI** (`scripts/pre-commit.sh` → `scripts/ci.ps1`): fmt + check + clippy `-D warnings` + test. ~40s warm rebuild on G:.
- **Egui version diamond** (still): `bevy_egui 0.39` ships egui 0.33; `egui_code_editor 0.2` pins egui 0.34. Hand-rolled syntax highlighter via `LayoutJob` lives in `editor.rs`.
- **Compile API design hard rule:** player input never touches `Cargo.toml`. Server owns the manifest.
- **GitHub Actions is banned** (global rule). Local CI only.
- **bevy_egui bindless gotcha:** if a future `bevy_egui` upgrade reverts the `EguiPlugin { bindless_mode_array_size: None, ..default() }` workaround in `editor.rs` (e.g. struct field rename), wasm will silently lose image-backed sprites again. The fix is unconditional; native is unaffected; keep the override.
- **Wasm grader is the stub.** `compile_client.rs` cfg-gates the network path off on wasm. Any change to mission grading needs to update both the live API (`/compile` route) AND `stub_grader.rs` to keep parity (the `stub_does_not_cross_grade_solutions` and `stub_passes_every_canonical_solution` tests already enforce this; lean on them).

## Action items for Matt (require Matt-only access)

1. **Visually verify the wasm rendering fix** — see Blocking Issue #1. This is the single highest-leverage action: confirms whether v1 is browser-shippable today.
2. **Drive claude.ai/design via claude-in-chrome to produce real NPC art** — per the art-lead role file. 4 placeholders + 17 redesigns. Can be batched.
3. **Decide on item 1 of "What's Next"** — is `cargo check`-based grading worth the 5-10s/request latency for v1.1, or do we keep the pattern grader and direct effort at Acts 3+ instead?

## Where to look next

- Curriculum → `design/01-curriculum.md`
- Art canon → `design/03-art-style-bible.md` and `design/04b-art-deliverables.md`
- Tech → `design/05-tech-architecture.md`
- Compile-API security → `design/05-tech-architecture.md` §2
- Team roles → `.claude/agents/`
- Audit gap log → `.claude/audit-gaps.md`
- Rust render pipeline → `tools/render-refs/`
- Web build → `scripts/web-build.ps1`, `tools/web-serve/`, `web/`
- Curriculum gating + epilogue → `game/src/plugins/mission.rs`
- Wasm compile path → `game/src/plugins/compile_client.rs`
- Wasm rendering fix → `game/src/plugins/editor.rs` (EguiPlugin) + `game/src/lib.rs` (AssetPlugin)
