# HANDOFF.md

## Last Updated
2026-07-01 — **Pitch deck + quest markers + non-intrusive in-engine capture.**
Built a full investor/publisher PDF pitch from real assets; shipped the WoW-style
golden `!` / `?` quest-marker feature; and stood up a self-driving in-engine
screenshot tool after discovering (and fixing) why the bare release exe rendered
a gray screen. Prior session (2026-06-20) rebuilt the village + art pipeline;
before that (2026-06-19) closed Acts 1–8 (57 missions) + Android.

## Project Status
🟢 **v1 + Acts 1–8 shipped (57 missions, 57 NPCs), native + web build/run, now
with quest markers.** Live in-engine screenshots confirm the designed village
renders correctly. A pitch deck PDF is ready to hand to a partner.

## What Was Done This Session (2026-07-01)

### Quest markers (new feature) — `game/src/plugins/quest_marker.rs`
- Floating **golden `!`** over the next quest-giver, **`?`** over a mission
  you've started but not yet turned in (cleared). WoW idiom (a game mechanic —
  not copyrightable; our own pixel art, no trade-dress concern).
- Glyphs are **procedural pixel-art `Image`s** built at startup: hand-authored
  `G`/`.` fill grids, 1px dark outline auto-derived from the 8-neighbourhood,
  `ImageSampler::nearest`. No font, no new art asset. Color is a **bright
  neon-gold `#FFDE2A`** (Matt's call — HUD affordance runs hotter than the
  Heraldic Code palette so it pops on grass *and* cobble).
- Marker child entity per NPC (attached once via `Without<HasMarker>`), z=50 so
  it floats above everything, gentle sine bob.
- Selection logic is a pure fn `marker_kind()` keyed off the existing
  strict-linear progression (`MissionRegistry::next_locked` + `MissionProgress`
  + `ActiveMission`). **5 unit tests, all green.**
- Registered in `game/src/plugins/mod.rs`.

### In-engine screenshot tool — `game/src/plugins/screenshot.rs`
- **Inert unless `PNC_SHOTS` env is set.** When on: skips the title, teleports
  player+camera through a list of framed vantages, captures each via Bevy's own
  `Screenshot` API (reads the GPU target → works unfocused/occluded), writes
  PNGs to `PNC_SHOTS_DIR`, then idles for the launcher to close it.
- `lib.rs`: in shot mode the window opens **`focused: false`** so an automated
  run can never steal keyboard focus.
- **Why this exists:** the first capture approach drove the real window via
  `SetForegroundWindow` + `keybd_event`, which stole focus *while Matt was
  typing* and leaked keystrokes into his terminal. **Never inject input / steal
  focus on the live machine again.** This tool has zero OS interaction.

### The pitch deck — `pitch/`
- `pitch/Pledge-and-Crown-Pitch.pdf` — 13-slide, 16:9, styled in the Heraldic
  Code palette, built from **real assets**: title art, all 57 NPC sprites, the
  Borrow Checker, and **two live release-build world captures** (Ferris + neon
  `!`, and the south road through grass).
- Pipeline (all in `pitch/`): `deck.template.html` (content/CSS, `@@token@@`
  asset slots) → `build-deck.ps1` (base64-embeds assets in PowerShell, builds
  the 57-cell cast grid, renders via **headless Chrome `--print-to-pdf`**) →
  `capture-world.ps1` (runs the game in `PNC_SHOTS` mode, waits for the PNGs) →
  `qa-shots.ps1` (screenshots the HTML, slices per-slide PNGs for review).

### Critical discovery — the "gray screen" was an asset-path bug, not a render bug
- Launching the **bare release exe** shows only the egui HUD on a gray
  clear-color — every sprite (title art, tiles, NPCs) missing, "file not found"
  spam. Root cause: Bevy's default `assets/` root resolves to the **exe's own
  directory** (`G:\cargo-target\...\release\assets`, which doesn't exist).
- **Fix: set `BEVY_ASSET_ROOT=J:\pledgeandcrowns\game`** (the crate that holds
  `assets/`). `cargo run` works without it because Cargo sets `CARGO_MANIFEST_DIR`.
- Also confirmed **`worldmap.png` is a STALE promo asset** — it depicts
  buildings/sea/roads that don't exist in the real game (no building tiles yet).
  The deck no longer uses it.

## Current State

### Working
- 57 missions / 57 NPCs, strict-linear curriculum gating, native + wasm build.
- **Quest markers** live and tested; neon `!` over the next NPC, `?` after you
  open a mission. Verified in live captures.
- **In-engine capture** produces clean 1280×720 PNGs with no focus stealing.
- **Pitch PDF** renders end-to-end from `build-deck.ps1`.
- Designed village renders correctly (cobble hub + 8 grass quarters + street
  grid + props). Confirmed by `pitch/world-shots/shot-*.png`.

### Test posture (green)
- 5 new `quest_marker` unit tests pass. Full `scripts/ci.ps1` gate was run this
  session before committing (fmt + check + clippy `-D warnings` + workspace tests).

### Stubbed / limited (unchanged from prior sessions)
- No building/wall/roof tiles (REF-04 atlas is 16 tiles) — quarters read as
  cobble plaza + prop, not literal architecture. Tile-art pass is a follow-up.
- `async_fn` mission is compile-only; `compile-real` is opt-in (needs
  `wasm32-wasip1` on the VPS); RPG combat/party layer not built.
- Art: 3/9 rough-9 NPCs are real claude.ai/design conversions; 6 remain.

## Blocking Issues (all owner-actions; none block code)
1. **Art conversion** — 6 sprites left (REF29 alchemist, 32 locksmith, 33 porter,
   35 armorer, 40 lanternkeeper, 41 loremaster). Method proven; see prior notes.
2. **Tauri Android signing** — bundle builds; needs a release keystore.
3. **compile-real prod default** — `rustup target add wasm32-wasip1` on the VPS.

## What's Next (prioritized)
1. **Pitch follow-ups (optional):** if the world slide wants more punch, a
   building-tile art pass would let a wide shot read as a town. The deck is
   otherwise partner-ready.
2. **Finish the 6 art conversions** (see blocker 1).
3. **Act 9 — Forbidden Library** (`unsafe`, FFI, `macro_rules!`, `Drop/Send/Sync`).
4. **Act 10 — Throne** (conceptual capstone).
5. **Learn-Rust thread** — Matt wants to learn Rust from this project; the
   quest-marker plugin is a clean, small, teachable diff to walk through.

## Notes for Next Session

**Running the game standalone (IMPORTANT):** the bare exe needs
`BEVY_ASSET_ROOT=J:\pledgeandcrowns\game` or it renders gray with "file not
found". Or just use `scripts/dev.ps1` / `cargo run -p pledge_and_crown` (Cargo
sets the asset root for you).

**Capturing marketing/world shots (non-intrusive):**
`pitch/capture-world.ps1` — launches the release exe with `PNC_SHOTS=1`,
`PNC_SHOTS_DIR`, `BEVY_ASSET_ROOT`; the game self-drives + screenshots + the
script waits for the PNGs then closes it. Edit the `VANTAGES` table in
`game/src/plugins/screenshot.rs` (world-space player positions) to reframe;
rebuild release; re-run. **Do NOT** resurrect any `SetForegroundWindow` /
`keybd_event` / topmost approach — it steals focus from whatever Matt is doing.

**Rebuilding the pitch PDF:** `pitch/build-deck.ps1` (needs
`pitch/world-shots/shot-ferris-marker.png` + `shot-south.png` present — capture
first if missing). Renders via headless Chrome/Edge `--print-to-pdf`. QA with
`pitch/qa-shots.ps1` (per-slide PNGs in `pitch/qa/`). The deck's factual claims
were kept honest per the no-overclaim rule; revenue figures are labeled
projections.

**Village layout (for reframing shots):** player spawns at (0,0) = center of
Hearthstone Square (cobble, ±300). Ferris (index 0, the fresh-save `!`) is at
(-240,-240). Grass quarters start beyond ±300; district centers in
`world.rs DISTRICTS`. NPC positions come from `district_slot(roster_index)`.

**Quest-marker tuning:** color `GOLD` const, size = 1 world-unit/glyph-pixel (no
`custom_size`), `MARKER_GAP`/`BOB_*` consts, glyph shapes are the `EXCLAM`/
`QUESTION` fill grids — all in `quest_marker.rs`.

**Windows/PS gotchas (still true):** ASCII-only `.ps1`; the running game locks
`pledge_and_crown.exe` (kill before rebuild); headless-Chrome print needs
`Start-Process -Wait` (backgrounded `&` + short sleep silently produced no PDF).

## Where to look
- Pitch → `pitch/` (deck.template.html, build-deck.ps1, capture-world.ps1, qa-shots.ps1)
- Quest markers → `game/src/plugins/quest_marker.rs`
- Screenshot tool → `game/src/plugins/screenshot.rs` (+ `lib.rs` focused flag)
- Curriculum → `design/01-curriculum.md`; art canon → `design/03-art-style-bible.md`, `design/04b-art-deliverables.md`
- Audit gaps → `.claude/audit-gaps.md`
