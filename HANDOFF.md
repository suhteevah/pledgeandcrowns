# HANDOFF.md

## Last Updated
2026-06-20 — **World overhaul + art-through-claude.ai/design.** Rebuilt the
village from procedural noise into a designed, districted town (Matt walked it
and approved), and stood up the claude.ai/design → JSX → render-refs → game
sprite-conversion pipeline (3 of 9 rough-9 NPCs converted so far). Prior session
(2026-06-19, below): closed the post-v1 roadmap + curriculum to Acts 1–8 (57
missions) + Android build. **57-mission/Act 1–8 work and Android are committed
+ pushed; this session's world+art work is committed this handoff.**

## Project Status
🟡 **v1 + Acts 1–8 shipped; world is now a designed village; art conversion
3/9 in progress.** Native + web build/run. The flat noise-map is replaced by a
180×140 hub-and-quarters town (Matt-approved). 3 of the 9 rough-9 NPC sprites
are now real claude.ai/design-designed grids in-game; 6 remain (method proven).

## What Was Done — 2026-06-20 (this session)

### World overhaul (game/src/plugins/{world,npc,player,mod,editor}.rs)
Matt flagged the world felt "small / non-sensical / cramped." Root causes found
+ fixed: a 3-way scale mismatch (tiles 1×, NPCs 1.5×, player 2× = a 4-tile
giant on 16px tiles) and a hash-noise tilemap with scattered NPC coords.
- **Designed village** (`world.rs`): a `DISTRICTS` table — Hearthstone Square
  hub + 8 themed quarters — with an orthogonal dirt **street grid** (axis-aligned
  segments; diagonal spokes looked jagged), cobble plazas + signature props, a
  treeline border. NPCs placed by `district_slot(roster_index)` into quarters
  (no more scatter). **Map 30×20 → 180×140** (~25k tiles, spawns clean).
- **Per-tile `tile_tint`**: weathered grayscale wobble on stone/dirt + green-TONE
  variation on grass (sunlit/shade/meadow) via `TileColor` — killed the flat
  "stamp" look. More flowers/tufts.
- **Scale harmonized**: `PLAYER_SCALE`/`NPC_SCALE` → 1.0 (2-tile characters).
  Camera zoomed out `VIEW_HEIGHT` 180 → **280** (mod.rs), follow-clamp now
  **derived** from VIEW_HEIGHT/aspect (player.rs) so it can't desync. Faster walk.
- **Parchment code page** (editor.rs): `CODE_PAGE_BG` — the editor rendered
  Abyssal (`#062123`) plain identifiers like `main` invisibly on egui's dark
  TextEdit; now a warm aged-parchment page so the dark syntax reads (bible REF-06).

### Art via claude.ai/design — rough-9 redesign + conversion pipeline
- Drove **claude.ai/design** (Matt's Max account, via claude-in-chrome) in the
  existing `pledgeandcrown` project's "NPC Sprites" artboard. Generated the 9
  "rough-9" NPCs (Quartermaster + 8 concept NPCs), then a **varied-build revision**
  so each reads as a distinct person (Matt-approved the look).
- Built **`design/art/design-mode-prompts.md`** (55 paste-ready Style-Suffix +
  per-NPC prompts) and **`scripts/art-review-sheet.ps1`** (contact-sheet montage).
- **Conversion pipeline established**: have Claude Design rasterize each sprite
  from its `npc-roster.jsx` into a 32×32 palette-char grid (text, extracted via
  `get_page_text`) → write `design/art/refs/ref-NN-*.jsx` → `cargo run -p
  render-refs --release --bin render-refs` → copy `design/art/refs/png/REFNN.png`
  to `game/assets/sprites/npc/<slug>_idle_0.png`. **Done for 3/9**: REF26
  quartermaster, REF27 auditor, REF28 chronicler (rendered, wired, validated 32×32).

## What Was Done — 2026-06-19 (prior session)

### Post-v1 roadmap — all 7 milestones (commits `61a2152`, `20f33ab`, `113c05b`)
- **mission_locked.wav wired** — Bevy 0.18 `Message` (`MissionLockedAttempt`) from `mission.rs` → `audio::sfx_on_locked_attempt`.
- **compile-real + wasm exec** — new `compile-api/src/wasm_builder.rs` bridges player source → `cargo build --target wasm32-wasip1` → `wasm_runner::run_wasm`. `/compile-real` returns an honest "compiled AND ran" verdict. Client opt-in via env `PLEDGE_REAL_COMPILE=1`. **Found + fixed a latent `wasm_runner` bug** (it disabled `bulk_memory`/`reference_types`, which rustc's wasip1 std requires — every real Rust module failed; a test even *asserted* the bug). Logged in `.claude/audit-gaps.md`.
- **wasm render verified** — rebuilt web, served, drove Chrome → title art renders in WebGL2 (the bindless fix is confirmed; v1 is browser-shippable). Also fixed a `web-build.ps1` asset-nesting bug.
- **audio-to-MIDI tool** (`tools/audio-to-midi/`) — ran a real GPU transcription; confirmed the village bass sits on B1.
- **AudioLDM2 SFX tool** (`tools/audioldm2-gen/`) — scaffolded; bake is rig-gated (CC-BY-NC weights, prototype-only).
- **Tauri 2.0 wrapper** (`mobile/`) — **desktop builds + links** (`app.exe`) once you select the VS **BuildTools** install over Community (the Community C++ workload is incomplete; `mobile/build-desktop.bat` queries by the VC.Tools component). **Android bundle builds** (2026-06-19): NDK r27d installed into `G:\AndroidSdk`, env wired, 4 targets added, `cargo tauri android build` → 31 MB release-unsigned APK. Only signing remains (owner-step). Build via `mobile/build-android.ps1`.
- **NPC art** — replaced the last 4 `SPRITE_PLAYER` placeholders (batch 4).

### Curriculum: Acts 3–7 mission batches (commits `e13a408`, `8754f9c`, `92200e8`, `818aac4`, `764b956`, `380b711`)
Each act = 6 gap-filling missions (the prelude already covered some concepts), wired end-to-end with the same recipe + a 6-NPC art batch:
- **Act 3 Guildhall Quarter** — `impl_method`, `assoc_new`, `if_let`, `while_let`, `tuple_struct`, `enum_data_match`.
- **Act 4 Trait Mage's Tower** — `trait_def`, `generic_fn`, `generic_struct`, `dyn_trait`, `lifetimes`, `assoc_type`.
- **Act 5 Tavern of Tribulations** — `result_match`, `custom_error`, `from_error`, `option_map`, `and_then`, `unwrap_or_else`.
- **Act 6 Iterator Forge** — `hashmap_basic`, `iter_filter`, `iter_fold`, `iter_enumerate`, `iter_zip`, `closure_move`.
- **Act 7 Concurrent Coast** — `thread_spawn`, `arc_mutex`, `mpsc_channel`, `atomic`, `thread_scope`, `async_fn` (compile-only).
- **Act 8 Vault of Pointers** (commit `0d51942`) — `box_basic`, `rc_basic`, `refcell`, `cell`, `rc_refcell`, `weak_ref`.

Per-act design specs live in `docs/superpowers/specs/2026-06-18-act{3..7}-*.md` and `2026-06-19-act8-vault-pointers-missions-design.md`.

## Current State

### Working
- **57 missions**, strict-linear chain, full curriculum gating. Each: 4-section tutorial + neutral starter + pattern grader (`compile-api/src/grader.rs`) **mirrored byte-for-byte** in the wasm stub (`game/src/plugins/stub_grader.rs`).
- **57 NPCs**, every one with distinct first-pass art (sprite batches 1–10). Rendered via the pure-Rust `render-refs` pipeline from `design/art/refs/ref-NN-*.jsx`.
- Native game + wasm web build both run; audio score + SFX; win epilogue.
- Honest build-and-run grading available behind `PLEDGE_REAL_COMPILE` (server needs `wasm32-wasip1` for prod).

### Test posture (all green)
- `scripts/ci.ps1` (fmt + check + clippy `-D warnings` + `cargo test --workspace`) is the pre-commit gate; every commit this session passed it.
- Contract suite (`game/tests/contract.rs`): per mission — canonical passes grader, starter doesn't trivially win, server↔stub flavor byte-parity, no freeform fallthrough, **+ slow `#[ignore]` cargo-check parity** (all 57 canonicals are valid Rust).
- Registry suite (`registry.rs`): linear prereq, unique ids/names, sprite-in-registry, tutorial substance.
- ~98 grader unit tests + the wasm_builder/wasm_runner slow suites.

### Stubbed / limited (intentional)
- `async_fn` mission is **compile-only** — `async fn`/`.await` type-check + token-grade, but the dep-free sandbox has no runtime (tokio) to actually run them.
- The doc's RPG layer (party tab, combat, per-act bosses) is **not built** — the code is the mission-tutorial loop. The world is now a **single designed village** with themed *districts* (no separate zones/maps yet; all 57 NPCs live across Hearthstone Village's quarters). True buildings + cobble-pattern variants need new **tile art** (no wall/roof tiles in the 16-tile REF-04 atlas) — a follow-up via the design pipeline.
- `compile-real` is opt-in, not the prod default (needs `wasm32-wasip1` on the Hetzner VPS).
- **Art**: 3 of the 9 rough-9 NPCs (quartermaster, auditor, chronicler) are now real claude.ai/design designs in-game; the other 6 + the ~30 earlier batches are still first-pass. The converted sprites use Claude Design's "parametric body" (geometric/a-bit-flat) — faithful to the approved design but not artist-final detail.

## Blocking Issues (all owner-actions, none block the code)
1. **Art review** — 36 first-pass NPC sprites (batches 4–10) await your 3-revision approval. Each is one `cargo run -p render-refs --bin render-refs --release` from a revision (edit the `ref-NN-*.jsx` grid). Rougher ones flagged in `design/04b-art-deliverables.md` batch notes.
2. **Tauri Android bundle** — ✅ **builds end-to-end** (2026-06-19). NDK r27d `27.3.13750724` + cmdline-tools installed into `G:\AndroidSdk` (via `mobile/install-android-ndk.ps1`); env persisted; 4 android targets added; `cargo tauri android init` + `build --apk --target aarch64` produce a 31 MB `app-universal-release-unsigned.apk`. **Remaining owner-step: signing** — APK is release-unsigned so not yet sideloadable; use `build-android.ps1 dev` (auto debug-signs to a device) for testing, or wire a release keystore for distribution. Build via `mobile/build-android.ps1`. See `mobile/README.md`.
3. **compile-real as prod default** — `rustup target add wasm32-wasip1` on the VPS, then flip the client.

## What's Next (prioritized)
1. **Finish the art conversion — 6 sprites left** (awaiting Matt's fidelity nod on the first 3; he saw the 3-up `scratch/sprite-compare.png`). Same method (see Notes): REF29 alchemist, REF32 locksmith, REF33 porter, REF35 armorer, REF40 lanternkeeper, REF41 loremaster. The approved designs are live in Matt's claude.ai/design `pledgeandcrown` project → "NPC Sprites" artboard.
2. **World polish (optional)**: true building tiles + cobble-pattern variants need a **tile-art pass** through claude.ai/design (the REF-04 atlas has no walls/roofs). Tuning knobs are all named consts (see Notes). If the world feels too sparse/dense, nudge `DISTRICTS`/`NPC_GRID_SPACING`/`VIEW_HEIGHT`.
3. **Act 9 — Forbidden Library** (`unsafe`, FFI, `macro_rules!`, `Drop`/`Send`/`Sync`): gradeable by token/compile; watch the no-`unsafe`-in-shipped-game-code hard rule (that's the game crate, not player canonicals).
4. **Act 10 — Throne** (perf, `no_std`, ecosystem): mostly conceptual; weakest fit — consider capstone treatment.
5. **Learn-Rust thread** — Matt wants to actually learn Rust from this project. Offered: teach-the-diff on engine changes, pair (he drives, I coach), and dogfood the 57-mission campaign. Not yet started; good low-stakes starts are the world tuning consts or the next art-conversion edit.
6. Owner-gated: art review, AudioLDM2 bake (P100 rig), **Android signing** (build works; sign with your keystore — blocker #2), compile-real prod default.

## Notes for Next Session

**Art-conversion method (proven on 3/9 — finish the other 6 this way):**
1. In Matt's Chrome, the claude.ai/design `pledgeandcrown` project's **"NPC Sprites"** artboard holds the approved sprites (built on `palette.js` via a coordinate pixel API in `npc-roster.jsx`). Direct iframe pixel-read is **blocked** (cross-origin sandbox) — so prompt Claude Design (its chat) to *rasterize each sprite from npc-roster.jsx into a 32×32 grid of single-char palette codes* (delimit `=== REFNN slug ===` + 32 single-quoted 32-char rows; `.` = transparent; "don't redesign, dump the exact grid").
2. `get_page_text` on the tab → parse the `=== REFNN ===` blocks.
3. Write each into `design/art/refs/ref-NN-<slug>.jsx` (replace the `const REFNN_GRID = [...]` array; keep the wrapper).
4. `cargo run -p render-refs --release --bin render-refs` (note **`--bin render-refs`** — the crate has 2 bins). It validates widths (skips malformed grids with a debug reason) and writes `design/art/refs/png/REFNN.png`.
5. Copy `REFNN.png` → `game/assets/sprites/npc/<slug>_idle_0.png`. Wiring (assets.rs/npc.rs) already points there — no code change.
6. `scripts/sprite-compare.ps1 <slug> <slug> ...` builds an upscaled montage for review.
Remaining: REF29 alchemist, REF32 locksmith, REF33 porter, REF35 armorer, REF40 lanternkeeper, REF41 loremaster.

**World tuning — all named consts:** `world.rs` `DISTRICTS` (centers/`plaza_half`/`count`/`prop`), `NPC_GRID_SPACING`, `MAP_W/MAP_H`, `tile_tint` (grass/stone tint tables), `ROADS` (orthogonal segments); `mod.rs` `VIEW_HEIGHT` (camera zoom) + `VIEW_ASPECT`; `player.rs` `PLAYER_SCALE`/`PLAYER_SPEED_PX_PER_SEC`; `editor.rs` `CODE_PAGE_BG`. NPC positions come from `district_slot(roster_index)` — the legacy `NpcSpec.pos` field is now unused (kept to avoid touching 57 literals).

**Rebuild gotcha (Windows exe-lock):** the running game holds `pledge_and_crown.exe`; `cargo run`/`build` then fails to link with `Access is denied`. **Before each rebuild:** `Get-Process pledge_and_crown -EA SilentlyContinue | Stop-Process -Force`. Use a background `until grep -qE "Running \`|error" <runlog>` loop to detect launch-or-error (one notification). The game spawns the village on entering InGame (hit start from the title), not at launch.

**The mission-batch recipe (proven 5×). To add a batch of N missions:**
1. `game/src/plugins/mission.rs` — N `Mission` entries appended before the registry's `];` (prereq auto-links linearly). 4-section tutorial; **neutral starter that does NOT pass its own grader** (the contract test enforces this — keep grader tokens out of the starter, including comments).
2. `compile-api/src/grader.rs` — N `grade()` arms (before the `_ =>` freeform fallthrough) + per-mission unit tests at the end of the test module.
3. `game/src/plugins/stub_grader.rs` — N arms **with byte-identical pass/fail strings** (the `server_and_stub_flavor_agree_byte_for_byte` test compares the *pass* string for each canonical; keep them identical anyway).
4. `game/tests/contract.rs` AND `game/tests/stub_grader.rs` — add the canonical solution to BOTH `canonical_solution()` fns (they're duplicated per test-crate). Canonical must pass its grader AND compile under host `cargo check`.
5. `game/src/assets.rs` — N `SPRITE_*` consts + add to `ALL_SPRITE_PATHS`.
6. `game/src/plugins/npc.rs` — N `NpcSpec` (unique name, pos >28px from every existing NPC) + import the consts.
7. Art: dispatch a subagent to author `ref-NN-*.jsx` grids (32×32, exactly 32 ASCII chars/row, palette codes only) → `render-refs` → copy PNGs to `game/assets/sprites/npc/`. Then `04b-art-deliverables.md` + `01-curriculum.md` code-status notes.
8. `cargo fmt --all`, then `git commit` (pre-commit hook runs full CI). Verify slow suite: `cargo test -p pledge_and_crown --test contract -- --ignored`.

**Gotchas burned in this session:**
- **rustfmt wraps long string lines** — write grader/test strings, then `cargo fmt --all` before committing or the pre-commit fmt step blocks. (String *values* are unchanged by wrapping, so byte-parity holds.)
- **`render-refs` silently SKIPS** a grid with a bad char or a row ≠ 32 chars (debug log, not an error). Run with `RUST_LOG=render_refs=debug` and confirm "rendered ref-NN". After editing an existing jsx, `touch tools/render-refs/build.rs` (it bakes refs at build time). Watch for **Cyrillic lookalike chars** (О/Т) — ASCII only.
- **Tauri on this box:** build with the VS **BuildTools** MSVC env, NOT Community (incomplete C++ workload). `vswhere -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64` picks the right one; `mobile/build-desktop.bat` does this.
- **wasm_runner** must keep `bulk_memory` + `reference_types` ENABLED (rustc's wasm std needs them). See `.claude/audit-gaps.md`.
- **Sandbox is dependency-free** — no external crates (`thiserror`, `tokio`, etc.) in player canonicals. Teach the std-only version; mention the crate in the tutorial.
- **Cargo serializes** on the shared `G:/cargo-target` lock — don't run two cargo builds at once. Non-cargo subagent work (art via render-refs is cargo, though) can overlap with editing.
- **GitHub Actions banned. Local CI only.** Pre-commit hook = `scripts/ci.ps1`.

## Where to look
- Curriculum → `design/01-curriculum.md` (Act code-status notes added through Act 7)
- Per-act design specs → `docs/superpowers/specs/2026-06-18-act{3..7}-*.md`
- Art canon → `design/03-art-style-bible.md`, `design/04b-art-deliverables.md` (REF manifest through 59), `design/art/palette.js`
- Audit gaps → `.claude/audit-gaps.md`
- Honest grading → `compile-api/src/wasm_builder.rs` + `/compile-real`
- Tauri wrapper → `mobile/README.md`
