# Pledge & Crown — finish plan

*Written 2026-11-13 from a full repo review. Audience: the Opus sessions that
implement it. Supersedes the "What's Next" list in HANDOFF.md.*

## Roles

- **Fable** — overview and art design. Designs each art tranche as draft grids,
  previews them through `render-refs`, and stages them on the design canvas.
- **Matt** — sole approver. Approves or redlines boards on the canvas.
- **Opus** — implements everything below. Wires art **only after** Matt approves
  the board it came from.

Design canvas (private to Matt): https://claude.ai/artifact/G67kHCdEaPS6KpYnD3jA1k

## 1. Where the project actually is

| Area | State | Evidence |
|---|---|---|
| Curriculum | 57 missions, Acts 1–8. Acts 9, 10 absent. Act 11 (Temple) is v2.0 by design | `mission.rs`, `design/01` |
| Loop | Walk, talk (F), edit, Compile, autosave of cleared set | `plugins/` |
| RPG layer | **None.** No combat, party, items, bosses, zones, interiors, pause, settings, dialogue beyond the mission prompt | `state.rs` has 2 states |
| Grading | Native: pattern grader over HTTP, real compile opt-in. **Web and mobile: offline pattern grader only** | `compile_client.rs:56-131` |
| compile-api | Runs locally. No deploy artefacts, auth, rate limit, container hardening, telemetry. Builds run on the async thread with no concurrency cap | `compile-api/src` |
| Web | Builds (42.6 MB wasm + 15 MB wav). No wasm-opt, no hosting, no loader | `scripts/web-build.ps1` |
| Mobile | Unsigned Android APK builds. No touch controls, no iOS | `mobile/` |
| Monetization | Nothing implemented (no paywall, demo gate, store pages, email capture) | grep of `game/src` |
| Art | See §2. One 16-tile atlas, 57 idle frames, title, no animation, no UI art | `game/assets/sprites` |
| Audio | 2 music tracks, 4 SFX, uncompressed wav, no volume control | `audio.rs` |
| Legal gates | TESS search, RF policy re-check, domain, Ferris sign-off all open | `CLAUDE.md` rule 6 |

Unverified and worth 5 minutes first: every mission has `prereq: None`
(`mission.rs:93+`) while HANDOFF says progression is strict-linear. Confirm
whether gating comes from registry order or is effectively off.

The "30 days to revenue" clock in CLAUDE.md was set in May. It is November.
D1 below resets the target.

## 2. Art review

Reviewed from live captures (`pitch/world-shots/`), the 57-sprite contact sheet
(`scripts/art-review-sheet.ps1`), and every REF png.

1. **The world is the weakest thing on screen.** Cobble is a per-pixel
   three-colour checker that reads as static at 3x. `tile_tint` multiplies each
   grass tile by one of five tones, so fields read as a checkerboard. The dirt
   tile has grass baked into both sides, so the 2-wide roads render as stripes.
   There is no architecture, water, or tree taller than 16 px.
2. **40 of 57 villagers are one body.** From REF-30 on, every sprite is the same
   grey-haired figure in a wine robe with a gold belt and green hem, told apart
   by a small prop. The bible's per-zone palettes (Tavern bronze, Forge basalt,
   Coast cobalt, Vault gold) are not used. Acts 1–2 (REF-11..25) are distinct
   and fine.
3. **Nothing moves.** Every character has exactly one frame. The bible asks for
   2-frame idles and 4-direction walks.
4. **Title art is broken.** The lower third of `title.png` is blank white bands.
5. **`worldmap.png` is stale** promo art for a world that does not exist, and is
   unregistered.
6. **No UI art.** Editor, dialogue and menus are default egui. No pixel font, no
   frames, no icons.
7. **Seven legacy refs cannot be re-rendered.** `render-refs` skips REF-03
   (row 53 is 60 wide, expected 64), REF-08 (row 5 is 15 wide), and REF-04, 05,
   06, 09, 10 (no literal grid). Their PNGs are from the deleted browser
   harness. The Borrow Checker sprite is one of them.
8. **The upstream is closing.** Standalone `claude.ai/design` shuts down by
   2026-12-14. `art-lead.md` and the no-fallbacks memory name it as the only
   design source. The replacement is the Design canvas above; the approved
   "NPC Sprites" artboard in the old `pledgeandcrown` project should be
   exported before the shutdown (the six remaining rough-9 conversions depend
   on it — do them first or lose them).

## 3. Art pipeline from here

```text
Fable drafts grids  ->  design/art/refs/ref-NN-<slug>.jsx   (header says DRAFT)
cargo run -p render-refs --release --bin render-refs        (RENDER_REFS_OUT for previews)
PNG uploaded to the design canvas  ->  Matt approves on the board
Opus: drop the DRAFT header, copy PNG into game/assets, wire, ci.ps1, commit
```

Done this session (uncommitted, lint and tests green):

- `tools/render-refs/src/main.rs` — tile-sheet support. A ref with
  `const REF<NN>_TILES = { name: [rows], ... };` and `const REF<NN>_COLS = n;`
  packs row-major into one atlas. Tile-sheet errors are fatal and name the tile;
  grid width errors now name the row.
- Tranche 1 drafts, `design/art/refs/ref-66` .. `ref-74` (see §4).

## 4. Asset manifest

Tranche 1 is designed and on the canvas. Everything else is still to design.

| Tranche | Contents | Spec | Status |
|---|---|---|---|
| **T1** | REF-66 terrain v2 (22 tiles: 2 grass, tuft, 2 flowers, 2 cobble, 2 dirt, water, 12 transparent grass-edge overlays). REF-67 oak 32×48. REF-68 cottage 64×64. REF-69..74 costume-kit exemplars for Acts 3–8 | 16×16 tiles, 8-col atlas; props anchor bottom-centre | **Designed, awaiting approval** |
| T2 | Player walk: down, up, left × 4 frames (right = flip). Ferris the same. Remaining 34 villagers of Acts 3–8 redrawn on their approved kit (same costume, new head, build and lesson prop) | 32×32 | Blocked on T1 kit approval |
| T3 | UI kit: dialogue frame, editor frame, button idle/pressed, 9-slice JSON, 16 px item icons, quest-marker restyle, pixel font choice | bible §UI | Not started |
| T4 | District landmarks (guildhall, tavern, forge, lighthouse, vault door, mage tower, market stall, well), then zone ground sets for Acts 2–8 if D2 chooses separate zones | props + 16×16 | Not started |
| T5 | Borrow Checker redraw as a renderable 64×64 grid with intro/defeat frames; one boss per act per D1; Act 9 and 10 casts and kits | bible §Sprite resolutions | Not started |
| T6 | Title screen redraw, world map redraw or removal, store capsules (after TESS) | 640×360 | Not started |

NPC idle animation needs no art: a 1 px, 600 ms engine-side bob satisfies the
bible's "breathing only" idle. Only the player and Ferris need drawn frames.

## 5. Work tracks for Opus

Tracks are independent unless a dependency is named. Fan them out.

### A — Art integration (starts when T1 is approved)

- **A1 Renderer hygiene.** Commit the tile-sheet change with unit tests for
  `render_tiles` (happy path, wrong row width, missing COLS). Fix the two
  malformed legacy grids (REF-03 row 53, REF-08 row 5) so they render again and
  diff against the committed PNGs with `compare_png`. Add a test that fails if
  any non-scaffold ref is skipped.
- **A2 Terrain.** Copy `REF66.png` to `game/assets/sprites/tiles_village_v2.png`.
  In `world.rs`: base layer picks variants by hash (grass_a/b, cobble_a/b,
  dirt_a/b); delete `tile_tint`; add a second tilemap layer for overlays chosen
  by a pure `fringe_for(neighbours) -> Option<u32>` with unit tests (edge,
  convex corner, concave nub, none); widen roads to 3 tiles.
  *Done when* a `PNC_SHOTS` capture shows no static, no checkerboard, no striped
  roads.
- **A3 Props.** A `Prop` component with y-sorting (`z` from `-y`), AABB
  collision in `player.rs`, and a data table placing a cottage ring in each
  district and an oak treeline in place of the sapling border.
- **A4 Animation.** Facing + `AnimationTimer` for player and Ferris, `flip_x`
  for right, engine-side idle bob for NPCs. Depends on T2.
- **A5 UI skin.** egui `Style` from palette role constants, pixel font, 9-slice
  frames. Depends on T3. Confirm the font licence before bundling.
- **A6 Docs.** Rewrite `.claude/agents/art-lead.md` and `design/04b` §9 for the
  pipeline in §3; update the manifest as assets land.

### B — Game systems (no art dependency)

- **B1 States and menus.** Extend `GameState` (Title, InGame, Paused, Settings).
  Title menu: New Game / Continue / Settings. Volume sliders persisted with the
  save.
- **B2 Editor.** Error-line highlighting from compiler output, goal pane, reset
  to starter, "show me the solution" after N failures. Remove the Day-6 stub
  starter text (`editor.rs:15`).
- **B3 Dialogue.** A small dialogue box for NPC greeting and post-clear lines so
  voice-editor's text has somewhere to live.
- **B4 Save v2.** Versioned migration from v1; store settings and last position.
- **B5 Grader table.** The pattern grader is 2049 lines of hand-rolled matches
  and past the rust-lead review's own refactor trigger. Make it table-driven,
  keep the byte-parity test green.

### C — Curriculum (uses `docs/superpowers/specs/` as the template)

- **C1 Act 9 Forbidden Library** — `unsafe`/raw pointers, FFI, `macro_rules!`,
  `Drop`, `Send`/`Sync`, `MaybeUninit`. Six missions, spec first, canonical
  solutions through `cargo check`. Hard rule 1 still holds: `unsafe` appears in
  *player* code inside the sandbox only.
- **C2 Act 10 Throne** — the capstone set per `design/01`, credits ending.
- **C3 Bosses** — shape depends on D1. Under the recommended option a boss is a
  3-stage chained mission with its own NPC sprite and arena prop, no combat
  system. Borrow Checker first (hard rule 3: its dialogue must match real
  compiler behaviour — test each stage against `compile-real`).
- **C4** Invoke `voice-editor` after each batch.

### D — Backend (security-reviewer activates: this touches `compile-api/`)

- **D1** `spawn_blocking` + a build semaphore; drop the run memory cap to the
  design's 16 MB or record why not.
- **D2** Per-mission expected-output check on `/compile-real`, so "ok" means
  correct rather than "exited cleanly".
- **D3** Bearer auth, per-IP rate limit, restricted CORS, container hardening
  per `design/05` §2.
- **D4** Deploy to the VPS with a systemd unit and `/metrics`; install
  `wasm32-wasip1` there (open HANDOFF blocker).
- **D5** Client: configurable API URL, and network grading from wasm so web
  players get the real compiler. Without this the public build grades by
  substring match.

### E — Ship

- **E1 Web.** wasm-opt, wav → ogg, loader page with progress, itch.io upload
  script (butler). Gate `web-build.ps1` and a wasm32 `cargo check` in `ci.ps1`.
- **E2 Demo gate.** Acts 1–2 free, the rest behind the full build (cargo
  feature), end-of-demo panel with email capture.
- **E3 Store.** itch page first, then Steam page. Needs T6 and the legal gates.
- **E4 Mobile.** Touch controls, signed AAB. After E1.
- **E5** Remove the personal email from `Cargo.toml` `authors`; fill
  `license`/`repository` in the Tauri manifest.

## 6. Order

1. **Now, parallel:** A1, B1–B5, C1 spec, D1–D3, E1, E5. None needs art approval.
2. **On T1 approval:** A2, A3, then a fresh capture set and a pitch-deck rebuild.
3. **Fable T2/T3** → A4, A5. **Fable T4/T5** alongside C1–C3.
4. D4, D5, then E2, E3. E4 last.
5. `rust-lead` review after each merged batch; `ci.ps1` green before every commit.

## 7. Decisions for Matt

- **D1 — What is v1.0?** *Recommended:* Acts 1–10 as a walk-talk-code RPG with
  staged boss trials, **no** combat, party or inventory. The design docs
  describe a full JRPG layer; none of it exists, and it is the largest single
  cost in the project. Alternative: build the combat layer first and ship later.
- **D2 — One village or separate zones?** *Recommended:* keep one village for
  v1.0, differentiate districts with landmarks and costume kits (T1/T4), and
  defer per-act zone maps. Separate zones add seven ground tilesets and zone
  transitions.
- **D3 — Tranche 1 approval** on the canvas, board by board.
- **D4 — Owner-only gates:** TESS + RF check, domain, Ferris sign-off, Android
  keystore, VPS access for D4.

## 8. Guardrails that still apply

No GitHub Actions. No Python or side-renderers in pipelines. No `unsafe` in
shipped game code. Player code never reaches the host or `Cargo.toml`. No art
wired before approval. Run the bare exe with `BEVY_ASSET_ROOT` set. Never
inject input or steal focus on the live machine — captures go through
`PNC_SHOTS`.
