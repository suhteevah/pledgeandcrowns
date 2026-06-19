# HANDOFF.md

## Last Updated
2026-06-19 — Big session: closed the entire post-v1 roadmap, then built out
the curriculum from the flat 21-mission prelude to **Acts 1–7 (51 missions,
51 NPCs, all with distinct art)**. Everything is committed + pushed to
`main` (`suhteevah/pledgeandcrowns`).

## Project Status
🟢 **v1 complete, browser-verified, and curriculum-expanded to Acts 1–7.**
Native + web both build and run. 51 missions cover the entire core Rust
language (variables → ownership → traits/generics → errors →
collections/iterators → concurrency). All graders + the offline stub are in
byte-parity; every canonical solution compiles under real `cargo check`.

## What Was Done This Session

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
- The doc's RPG layer (party tab, combat, per-act bosses, multi-zone world) is **not built** — the code is the mission-tutorial loop. All Act NPCs live in Hearthstone Village.
- `compile-real` is opt-in, not the prod default (needs `wasm32-wasip1` on the Hetzner VPS).
- The ~36 first-pass sprites (batches 4–10) are **mine**, not artist-final — pending your locked art review.

## Blocking Issues (all owner-actions, none block the code)
1. **Art review** — 36 first-pass NPC sprites (batches 4–10) await your 3-revision approval. Each is one `cargo run -p render-refs --bin render-refs --release` from a revision (edit the `ref-NN-*.jsx` grid). Rougher ones flagged in `design/04b-art-deliverables.md` batch notes.
2. **Tauri Android bundle** — ✅ **builds end-to-end** (2026-06-19). NDK r27d `27.3.13750724` + cmdline-tools installed into `G:\AndroidSdk` (via `mobile/install-android-ndk.ps1`); env persisted; 4 android targets added; `cargo tauri android init` + `build --apk --target aarch64` produce a 31 MB `app-universal-release-unsigned.apk`. **Remaining owner-step: signing** — APK is release-unsigned so not yet sideloadable; use `build-android.ps1 dev` (auto debug-signs to a device) for testing, or wire a release keystore for distribution. Build via `mobile/build-android.ps1`. See `mobile/README.md`.
3. **compile-real as prod default** — `rustup target add wasm32-wasip1` on the VPS, then flip the client.

## What's Next (prioritized)
1. **Act 9 — Forbidden Library** (`unsafe`, FFI `extern "C"`, `macro_rules!`, `Drop`/`Send`/`Sync`): gradeable by token/compile, but `unsafe`/FFI are harder to make *meaningful* in a sandbox. **Note:** `unsafe` in a graded canonical is fine (it compiles under cargo check), but watch the hard-rule about no `unsafe` in *shipped game code* — that's about the game crate, not player canonicals.
2. **Act 10 — Throne** (perf, `no_std`, ecosystem): mostly conceptual; weakest fit for the mission loop — consider capstone/non-mission treatment.
3. Owner-gated: art review, AudioLDM2 bake (P100 rig), **Android signing** (build works; sign the APK/AAB with your keystore to ship — see blocker #2), compile-real prod default.

## Notes for Next Session

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
