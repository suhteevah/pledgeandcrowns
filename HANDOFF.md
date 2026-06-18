# HANDOFF.md

## Last Updated
2026-06-18 (session 3) — drove the full post-v1 roadmap. Closed the stated
v1 browser blocker (wasm title art **visually verified rendering in
Chrome**), built honest build+run grading (compile-real + wasm exec) and
**fixed a latent `wasm_runner` bug that would have rejected every real Rust
module**, shipped the audio-to-MIDI tool (with a real transcription),
scaffolded the AudioLDM2 SFX tool and the Tauri 2.0 mobile wrapper, and
teed up the 4 remaining NPC sprites with bible-faithful art briefs.

## Project Status
🟢 **v1 complete and browser-verified.** Native: 21 missions, full
curriculum gating, win epilogue, full synthwave score + SFX (now including
mission_locked). Web: builds, renders, and runs — the bindless rendering
fix is **visually confirmed** (title art + scene render in WebGL2). Honest
build+run grading is available behind an opt-in flag.

## What Was Done This Session (2026-06-18)

Roadmap milestones, by disposition:

### ✅ Wired `mission_locked.wav` into the audio plugin (was "trivial, TODO")
- New Bevy **message** `MissionLockedAttempt` emitted in
  `mission::handle_interact_key` on an F-press against a locked NPC;
  consumed by `audio::sfx_on_locked_attempt` (Bevy 0.18 uses
  `Message`/`MessageReader`/`add_message`, not the old `Event` buffered
  API — verified against the installed source).
- Decoupled by design: the locked-attempt detection stays in the one
  system that already owns the F-press + nearby-NPC + progress reads,
  rather than re-deriving it in `audio.rs`.
- 2 new regression tests in `game/tests/bevy_smoke.rs` (locked NPC →
  exactly one message + editor stays closed; unlocked NPC → zero messages
  + editor opens). 7/7 smoke tests green.

### ✅ compile-real + wasmtime exec wiring (honest grading)
- New `compile-api/src/wasm_builder.rs`: bridges player source →
  `cargo build --target wasm32-wasip1 --release` → raw `.wasm` →
  `wasm_runner::run_wasm`. Same security contract as `cargo_grader`
  (server-owned bin manifest, UUID sandbox, env_clear allowlist,
  wall-clock watchdog, Drop cleanup, `panic="abort"` so panics trap).
- `/compile-real` route now returns an honest **compiled AND ran** verdict
  with the program's real stdout, marked `[real]`.
- Game client opt-in: set **`PLEDGE_REAL_COMPILE=1`** to route to
  `/compile-real` (default stays the fast `/compile` pattern grader —
  keeps MVP latency; the prod default-switch is gated on installing the
  wasip1 toolchain on the server).
- **Latent bug found + fixed:** `wasm_runner`'s `Config` disabled
  `bulk_memory` + `reference_types` as "defence in depth," but those are
  WebAssembly 2.0 baseline that rustc's wasip1 **std** emits — so it
  rejected *every* real Rust module with `zero byte expected`. Never
  caught because no test had ever built real wasm and run it; worse, a
  test *asserted* the buggy behavior. Now enabled (exotic proposals stay
  off); the deny-list guard re-pointed at SIMD. Logged in
  `.claude/audit-gaps.md`.
- Tests: `compile-api/tests/wasm_builder.rs` (9, `#[ignore]`d — real
  build+run: hello-world captures stdout, compile-error/missing-main
  rejected, panic→run-fail, infinite-loop→fuel/timeout, sandbox cleanup).
  `wasm_runner.rs` tests updated (bulk-memory now accepted, SIMD denied).
  All green. `rustup target add wasm32-wasip1` was run on this box.

### ✅ Visual verification of the wasm rendering fix (was the #1 Matt-blocker)
- Rebuilt wasm (`scripts/web-build.ps1`), served via `web-serve`, drove
  **Chrome** to `127.0.0.1:8080` and screenshotted: the "PLEDGE & CROWN"
  title art, crown sprite, and full pixel scene (mountains/water/bridge/
  player/NPCs) all render in WebGL2. **The bindless fix is confirmed; v1
  is browser-shippable.**
- **Also fixed a `web-build.ps1` bug found doing this:** `Copy-Item
  -Recurse` into an existing `web/assets/` *nested* the fresh assets under
  `web/assets/assets/`, so every rebuild after the first silently served
  STALE sprites and stranded new audio. Now clears the dest first.

### ✅ Audio-to-MIDI tool (you asked whether SAO output can be transcribed)
- `tools/audio-to-midi/` + `scripts/audio-to-midi.ps1`: Demucs (htdemucs
  stem split) → Spotify Basic Pitch (per-stem transcription) → per-stem +
  combined `.mid`. onnxruntime backend (no TensorFlow). Manifest-driven,
  same pattern as synthwave-gen.
- **Actually run end-to-end** (py 3.10 venv, torch cu121, GPU): produced
  real MIDI for `village` — and it answered the load-bearing question:
  **the village bass IS anchored on B1** (B1 dominant every run),
  confirming the SAO prompt steering. MuseScore sheet-music step is
  documented as a manual follow-up (not implemented).

### ✅ AudioLDM2 diegetic-SFX tool (scaffold; bake is rig-gated by your note)
- `tools/audioldm2-gen/` + `scripts/audioldm2-gen.ps1`: `cvssp/audioldm2`
  via diffusers, 16 kHz mono, 9 SFX (door creak/open, parchment, footsteps,
  coin, sword, anvil, quill, latch) → `game/assets/audio/sfx/`. Validated
  by py_compile + a torch-free `--dry-run`. **No bake run** (deferred to
  the P100 rig per your note). ⚠️ AudioLDM2 weights are **CC-BY-NC** —
  prototype-only; clear SFX licensing before shipping.

### ✅ Tauri 2.0 mobile wrapper — desktop build WORKS
- `mobile/src-tauri/` via `cargo tauri init`: frontendDist → `../../web`,
  standalone `[workspace]` (detached from the game workspace), 1280×720
  window, provisional identifier `com.pledgeandcrown.game`, npm hooks
  removed (frontend is the prebuilt `web/`).
- **Builds + links to a runnable `app.exe` (37 MB PE32+)** via
  `mobile/build-desktop.bat`. The helper's key trick: the box has TWO VS
  2022 installs and only **BuildTools** has a complete C++ workload
  (Community is OneCore-libs-only, no `vcvarsall.bat`). `vswhere -latest`
  picks the broken Community; `build-desktop.bat` queries
  `-requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64`, which
  returns BuildTools → links cleanly. GNU can't link it (webview2 export
  overflow), so the wrapper alone uses MSVC; the game stays on GNU.
- **Android bundle** is the remaining step and needs the Android SDK+NDK
  (`ANDROID_HOME`/`NDK_HOME` + android rust targets + `cargo tauri
  android`). None found on this box (checked user + machine scope), so
  that's the one mobile-packaging gate. Desktop is done. Details in
  `mobile/README.md`.

### ✅ NPC art — last 4 placeholders replaced with real sprites
- Wrote 4 bible-faithful **specs** (`design/art/specs/{quartermaster,
  auditor,chronicler,alchemist}.md`), then authored the **JSX grids**
  (`design/art/refs/ref-26..29-*.jsx`) from them and rendered to PNG via
  the canonical pure-Rust pipeline (`cargo run -p render-refs`), wired into
  `assets.rs` (`SPRITE_QUARTERMASTER/AUDITOR/CHRONICLER/ALCHEMIST` +
  `ALL_SPRITE_PATHS`) and `npc.rs::NPC_ROSTER`. **All 21 NPCs now carry
  distinct art** — no `SPRITE_PLAYER` placeholders left.
- Palette-compliant (Heraldic Code), each prop tied to its mission (gold
  sacks = slice window, ledger+`!`Err-seal, clockwork tome = derive,
  teal vials = iter/map/collect). The Auditor uses a documented ~0.6%
  Alarm-scarlet exception (under the 1% cap).
- **These are first-pass sprites** authored by me, a clear upgrade from the
  player-clones — NOT a substitute for Matt's locked art-review (the
  3-revision approval flow). Re-render after any edit with
  `cargo run -p render-refs --bin render-refs --release`. Note: a web
  rebuild (`scripts/web-build.ps1`) will pick them up for wasm too.

## Verification (this session)
- Full local CI green: `scripts/ci.ps1` (fmt + check + clippy `-D warnings`
  + `cargo test --workspace`). The new tests all pass; clippy clean.
- wasm_builder/wasm_runner `#[ignore]`d slow tests run + green via
  `cargo test -p pledge_compile_api --test wasm_builder --test wasm_runner -- --include-ignored`.
- wasm render confirmed visually in Chrome.

## Blocking Issues (remaining = Matt-actions)
1. **Tauri Android bundle** — desktop builds today via
   `mobile/build-desktop.bat`. The Android bundle additionally needs the
   Android SDK+NDK (set `ANDROID_HOME`/`NDK_HOME`) + the android rust
   targets, then `cargo tauri android init/build`. No SDK found on this box
   (checked user + machine scope). Full runbook in `mobile/README.md`.
   (Note: build the wrapper with the VS **BuildTools** MSVC env, not
   Community — see README; `build-desktop.bat` already selects the right
   one.)
2. **NPC art generation** — drive `claude.ai/design` from the 4 new specs
   (+ the 17 redesigns), approve, render via `render-refs`, wire in.
3. **compile-real as the prod default** — needs `rustup target add
   wasm32-wasip1` on the Hetzner VPS; then flip the client to
   `PLEDGE_REAL_COMPILE` / drop the `[real]` marker. ~build+run latency
   (a few seconds) vs ~100ms pattern grading — a v1.1 call.

## What's Next (post-this-session)
1. **Bake the AudioLDM2 SFX** once the P100 rig lands; wire `sfx/*.wav`
   into `audio.rs` (mirror the existing SFX edge systems).
2. **Per-encounter expected-output grading** on top of compile-real (the
   execution layer is now in place; define expected stdout per mission).
3. **Acts 3+ curriculum** — new missions/NPCs/art. Still **Matt-territory**
   for content design; not expanded this session (scope discipline).
4. Audio-to-MIDI: run `--all` + the MuseScore sheet-music step if you want
   full scores.

## Notes for Next Session
- **Read this file first.**
- **Run native:** terminal 1 `cargo run -p pledge_compile_api`, terminal 2
  `cargo run -p pledge_and_crown`. For honest grading: set
  `PLEDGE_REAL_COMPILE=1` before launching the game (needs the compile-api
  up + `wasm32-wasip1` installed locally — it is, on this box).
- **Run wasm:** `scripts/web-build.ps1` → `cargo run -p web-serve --release`
  → `127.0.0.1:8080`. Title art renders (confirmed).
- **Default toolchain is GNU** (`x86_64-pc-windows-gnu`) — the game links
  fine on it; the Tauri wrapper does not (needs MSVC + VS Build Tools).
- **wasm_runner feature gotcha:** keep `bulk_memory` + `reference_types`
  ENABLED — rustc's wasm std requires them. See `.claude/audit-gaps.md`.
- New tools each have their own `.venv` (gitignored) + an ASCII-only
  `scripts/*.ps1` launcher mirroring `synthwave-gen.ps1`.
- **GitHub Actions banned. Local CI only.**

## Where to look next
- Curriculum → `design/01-curriculum.md`
- Art canon → `design/03-art-style-bible.md`, `design/04b-art-deliverables.md`; new specs in `design/art/specs/`
- Compile-API security → `design/05-tech-architecture.md` §2
- Audit gap log → `.claude/audit-gaps.md`
- **Honest grading → `compile-api/src/wasm_builder.rs` + `/compile-real` in `lib.rs`; client opt-in in `game/src/plugins/compile_client.rs`**
- **Locked-attempt SFX → `game/src/plugins/mission.rs` (MissionLockedAttempt) + `audio.rs`**
- **Audio-to-MIDI → `tools/audio-to-midi/`; AudioLDM2 → `tools/audioldm2-gen/`**
- **Tauri wrapper → `mobile/` (README has the toolchain runbook)**
