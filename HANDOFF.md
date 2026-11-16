# HANDOFF.md

## Last Updated
2026-11-15 — **Orchestrated build-out: waves 1-14 merged; wave 15 (Act 10, the last act: the game now has all ten acts) on its branch for review.**

Branch **`finish/integration`** (not pushed, `main` untouched). Fable
orchestrates Opus agents in git worktrees, reviews each branch, merges, and
runs `scripts/ci.ps1` on the merged result as the sign-off gate.

### Signed off (CI green on the merged branch)
- **Wave 1** (`83f5daf`): terrain v2 + grass-edge overlay layer, cottage/oak
  props with y-sort and collision, title/pause/settings menus, save v2 with v1
  migration, rebuilt editor, compile-api auth/rate-limit/queue/metrics, web
  build 49.9 -> 22.5 MB, ogg audio, itch zip script, render-refs hygiene.
- **Wave 2** (`ffd62c1`): dialogue box (authored lines for the first 9 NPCs),
  configurable authenticated client on native and wasm, 57/57 player-facing
  solutions proven to compile, table-driven graders, compile-api sandbox fixes,
  art docs for the new pipeline.
- **Wave 3** (`bc9becf`): scripted capture scenes (`scripts/capture-scenes.ps1`,
  29 scenes), tutorial corrections (104 snippets run through rustc), `demo`
  cargo feature (21 free missions, ends at `while_loop`), comment-aware token
  grader, per-mission behaviour checks on `/compile-real` (41 of 57), Act 9 and
  10 specs with proven solutions.

### Waves 4-6 (signed off through wave 5 at `9cea928`; wave 6 merged)
- **Wave 4** (`7084912`): palette-driven UI theme (title, pause, settings, HUD,
  panels); editor rebuilt with a fixed layout and honest grade banners
  (VERIFIED / NOT VERIFIED / PATTERN CHECK ONLY); dialogue inline code, a
  correct "come back later" line, persisted greeting state; props fade to 62%
  when they hide a character; compile-api split into a token-holding API unit
  and a secret-free builder unit; `fn main` required; edition 2024 everywhere.
  **Tests no longer touch the real save** (`ProgressPlugin::on_disk()` for the
  game, `::in_memory()` for every test).
- **Wave 5** (`9cea928`): flagstone plazas (REF-75 via combined atlas REF-76);
  one colour source and one markdown renderer in `ui_theme`; 54 of 57 missions
  behaviour-checked by the server (3 static-only: two thread missions and
  `async_fn`); **the sandbox proven on Linux (cnc)** with five Linux-only fixes.
- **Wave 6** (merged, CI on the merged tree is the gate): ink code surface as
  bible v2.2 candidate behind `ui_theme::DEFAULT_CODE_SURFACE`; wasmtime 44 ->
  50 with 0 advisories left in the server tree and `deny.toml`; run timeout
  enforced (a sleeping guest now answers in ~5 s); OOM path exercised;
  `verify-sandbox.sh` has a pass that runs under the real unit (46 checks).

### Waves 7-8 (signed off, CI green on the merged tree)
- **Wave 7** (`774828f`, `e6cdfde`): third security review closed. wasmtime now
  runs in its own bwrap run sandbox, connect timeout, no internal paths in
  500s, `#![forbid(unsafe_code)]` in the harness entry. Verified on cnc (391
  Linux tests, `verify-sandbox.sh` PASS 111 checks). Verdict: safe to deploy on
  a host like the one tested; Debian 12, Ubuntu 24.04 and SELinux enforcing are
  untested (all fail closed). Never deployed.
- **Wave 8** (`6d022c6`..`50823a3`): boss trials (`game/src/plugins/trial.rs`):
  one NPC owns an ordered run of stage missions; stages are ordinary missions.
  First trial is the Borrow Checker's, **The Bridge** (`bridge_toll`,
  `bridge_quill`, `bridge_lantern`) after `while_loop`; `borrow_preview` stays
  his prologue. **60 missions (69 after wave 9), 57 NPCs; demo = 24 free, ends on
  `bridge_lantern`.** Every error he names is compiled in
  `game/tests/mission_text.rs`. Voice pass done. `PNC_SHOTS_ONLY` filters
  capture scenes.
  Polish left: on completion panels the viewed stage shows a ring, not a tick;
  he has no marker between mission 3 and the trial.

- **Wave 9** (`4e97211`): trials for Acts 3-5: the Variant Wraith (the
  Guildmaster), the Chimera of Vexis (Vexis), the Tavernkeeper's Wager (the
  Barkeep), three stages each, owned by existing NPCs, no new art. **69
  missions, 57 NPCs; demo unchanged (24 free), 45 after.** New
  `MissionCheck::forbids` on the server (only `wager_quest` uses it). All 19
  trial compiler claims are compiled in `game/tests/mission_text.rs`.
  Open: no voice-editor pass on the three owners' new lines; trial owners
  stand on the plaza edge wall in the offer scenes (NPC placement, older
  issue); `pitch/deck.template.html` still says 57 missions.

- **Wave 10** (this branch): trials for Acts 6-8, the last before the
  endgame: the Forge-Golem (the Keymaster: filter before map, E0609; `sum`
  needs a type, E0283; one chain with no `for`/`while`/`loop`/`mut`,
  enforced by `forbids`), the Tide Hydra (the Dockmaster: a sender clone
  per head, E0382; `move` into one thread per head, E0373; wait for every
  `Down`), the Memory Warden (the Vaultwright: `Box` for a recursive
  ledger, E0072; `Rc<RefCell<u32>>`, E0594; a ring of debts linked with
  `Weak`). **78 missions, 57 NPCs; demo unchanged (24 free), 54 after.**
  The server has no threads: `hydra_threads` is static-only (4 static-only
  now) and `hydra_body` is checked by feeding `await_heads` from one thread
  without running `main`; both thread programs are proven natively, 21 runs
  each. No mechanism change, no art.
  Open: no voice-editor pass on the three owners' new lines; still no
  combat layer (Hydra regrowth, Cliphy, the Warden's fourth phase are
  design-doc only); `pitch/` counts untouched (owner's).

- **Wave 11** (`5aaae22`, `361901e`, merged at `f763dfb`, CI green): voice
  pass on the six Act 3-8 trial presenters (18 lines); NPC placement fix:
  the district grid now shrinks to fit each plaza (32 NPCs were standing on
  plaza edge walls), the Cartographer no longer stands on the spawn point,
  `game/tests/npc_placement.rs` guards it, capture scenes use
  `npc::TALK_SPOT_OFFSET`. `NPC_ROSTER.pos` is a leftover spawning never
  reads. Open: curriculum doc calls the barkeep "Old Maybe, speaks in
  `Option`" but no shipped text does.

- **Wave 12** (`cf9bc47`, merged `9a9f4cd`, CI green): server grading mode
  for Act 9's `unsafe` missions, `HarnessMode::UnsafeLateNonce` (opt-in per
  mission in the static table; no `forbid(unsafe_code)`; the nonce arrives
  through a one-shot host import after player code has finished). **No
  mission uses it yet.** Fourth security review
  (`.claude/reviews/security-2026-11-15-cf9bc47.md`): no P0/P1, deploy verdict
  unchanged. Linux run (`linux-verification-2026-11-15c.md`): 406 tests pass,
  one fixture graded end to end through real bwrap build + run;
  `verify-sandbox.sh` was NOT re-run. Known, documented limit: a player can
  forge a pass in this mode by rewriting the checker's stored results
  (`known_limit_heap_patch_forges_a_pass`); accepted, same class as C-2.
  When Act 9 registers missions: turn
  `every_registered_mission_keeps_forbid_unsafe` into an allow-list (P2-3)
  and add the two host-import edge tests (P2-2).
- **Wave 13** (`c998890`, merged `a3872fe`): Matt approved all art on
  2026-11-15. The 30 Act 3-8 villager redraws are the in-game sprites; walk
  animation in `game/src/plugins/animation.rs` (player four facings, Ferris
  scuttle, NPC idle bob; draw-anchor lifts only, so collision, y-sort and
  talk range are untouched); `game/tests/art_wiring.rs`; style bible is v2.2
  (ink code surface default). The six kit holders and the district names
  were left open here; wave 14 closed both.
- **Wave 14** (`51e1745`, merged at `bff5188`): (1) the six kit holders
  (Guildmaster, Conjurer, Barkeep, Smelter, Dockmaster, Warden) now ship
  their act's approved kit exemplar REF-69..74 (first-pass refs marked
  SUPERSEDED, `tests/art_wiring.rs` pins it); districts renamed to their
  casts: Tradesmen's End -> The Mage's Tower (Act 4), The Wharf -> The
  Forge (Act 6), Stonebridge -> The Wharf (Act 7), The Old Gate -> The
  Vault (Act 8); no NPC moved district. (2) **Act 9, the Forbidden
  Library**: six missions (`raw_pointer` Shelfmark, `extern_c` Envoy,
  `macro_rules` Macro Monk, `drop_trait` Sexton, `send_sync` Courier,
  `maybe_uninit` Bookbinder) and the Librarian's trial (`librarian_index`,
  `librarian_demand`, `librarian_seal`; no prologue, his roster mission is
  stage 1). **87 missions, 64 NPCs; demo unchanged (24 free), 63 after.**
  Map 180x220: the N-S street runs on to the Forbidden Library (south,
  7 NPCs, the 64 px Librarian centred in the back row) and the Throne
  (north, empty for Act 10); grid clearance now comes from the largest
  sprite (hub 228 px, Library 188 px). Landmarks NOT wired (REF-95/96 are
  drafts): `prop::PENDING_LANDMARKS` holds their spots. Server: five
  stdout checks + four `UnsafeLateNonce` harness checks (`extern_c`,
  `send_sync`, `librarian_index`, `librarian_seal`; explicit allow-list
  test, review P2-3; host-import tests for no `memory` export and
  `ptr = 0xFFFFFFFF`, P2-2). The game labels late-nonce passes
  `PASS · NOT VERIFIED` (`[ok:unsafe]`, keyed on the server's note),
  still a clear. `#![forbid(unsafe_code)]` on the game crate. Every Act 9
  compiler/runtime claim is compiled or run in `mission_text.rs`; spec
  corrections in its "Implementation record". **No Miri run backs the
  `unsafe` solutions** (Miri not installed). Art for Act 9 wired
  (REF-80..85, REF-93; REF-79 kit catalogued).
- **Wave 15** (this branch, one commit): (1) Act 9 follow-ups: the
  `PASS · NOT VERIFIED` banner (and every banner line) renders inline
  `code` as chips, not raw backticks (`widgets::draw_banner` uses
  `ui_theme::markdown`); district line-up captures hide the "Next" pin
  (`mission::HideNextPinForCapture`, capture-only); the "notyet" dialogue
  captures name the Ghostkeeper instead of "the last roster NPC".
  (2) **Act 10, the Throne, the last act**: six missions (`prealloc`
  Assayer, `cow_borrow` Copyist, `const_eval` Astronomer, `chunks_exact`
  Choirmaster, `no_std` Hermit, `semver_caret` Registrar) and the
  Compiler's trial (`compiler_alloc`, `compiler_lifetime`,
  `compiler_bound`; no prologue, his roster mission is stage 1). **96
  missions, 71 NPCs; demo unchanged (24 free, ends on `bridge_lantern`),
  72 after.** Clearing `compiler_bound` clears everything and shows the
  rewritten epilogue (credits block: Chico Cell Repair LLC, the libraries,
  the Rust trademark non-affiliation line; `mission::epilogue_due`). The
  Throne holds its seven NPCs (the 64 px Compiler alone, centred, in the
  back row); the Throne Dais (REF-96, approved 2026-11-15) is wired as a
  solid, y-sorted 64x64 prop ON the plaza at its back-centre, the Compiler
  centred just in front of it (`prop::LANDMARKS`,
  `world::back_landmark_base`; the rows moved south, 152 px apart; second
  commit, after review). The Library's REF-95 reservation moved to the
  same back-centre spot behind the Librarian; REF-95 stays a draft.
  Server: all nine are stdout checks with construct tokens in the plain
  build; the two allocator missions (`prealloc`, `compiler_alloc`) carry
  the player's own counting `#[global_allocator]`, which the plain build
  has always allowed (no harness, no mode change, late-nonce allow-list
  unchanged; HARNESS-NOTES "Act 10"). Nothing grades speed. The
  Compiler's dialogue quotes no compiler output (the result pane shows
  real stderr). Every Act 10 compiler/runtime claim is compiled or run in
  `mission_text.rs`; spec corrections in its "Implementation record".
  Art REF-87..92 and the Compiler REF-94 wired, REF-86 kit catalogued.

### Art status (Matt is the sole approver; design canvas)
- **Wired 2026-11-15 (wave 14):** the kit exemplars REF-69..74 as the six
  kit holders; Act 9 cast REF-80..85 and the Librarian REF-93 (REF-79, the
  Library kit, catalogued, not an NPC).
- **Wired 2026-11-15 (wave 15):** Act 10 cast REF-87..92, the Compiler
  REF-94 (REF-86, the Throne kit, catalogued, not an NPC), the Throne Dais
  REF-96 (landmark prop).
- **Awaiting approval:** landmark REF-95 Sealed Stacks (spot reserved in
  `prop::PENDING_LANDMARKS`).
- **Not drawn:** boss intro/defeat frames (Librarian, Compiler), the
  Librarian's lectern, pixel font pick, title redraw, a renderable Borrow
  Checker.


### Known-open, in priority order
1. **compile-api has never been deployed.** Three reviews in
   `.claude/reviews/security-*.md`; before a VPS deploy, run the builder
   self-test and `deploy/verify-sandbox.sh` on that exact host.
2. **Nobody has played the build.** UI is checked only through capture scenes.
   No human has confirmed audio is audible (wav never decoded; now ogg).
3. **4 missions are static-only on the server** (`thread_spawn`,
   `thread_scope`, `hydra_threads`, `async_fn`); the other 92 are
   behaviour-checked (`hydra_body` without running its threaded `main`;
   four Act 9 harness checks in `UnsafeLateNonce`, labelled NOT VERIFIED
   because a player's `unsafe` code can forge them; the nine Act 10
   missions are stdout checks).
4. **Art still owed by the designer (see Art status above):** pixel font,
   approval of the Sealed Stacks landmark (REF-95), Borrow Checker,
   Librarian and Compiler intro/defeat frames, the Librarian's lectern,
   title redraw.
5. **All ten acts are implemented** (Act 9 in wave 14, Act 10 in wave
   15). **Before any deploy that serves Act 9**, run the security review's
   Linux list (`.claude/reviews/security-2026-11-15-cf9bc47.md`, "Must run
   on Linux"): the bwrap late-nonce test, end-to-end grades of the
   registered `librarian_index` and `extern_c`, the journal nonce grep,
   `verify-sandbox.sh`. Miri has never checked the Act 9 solutions or the
   Act 10 counting allocators. Act 10's allocation counts were proven in
   the sandbox only on Windows (`expectations_proof`); re-run that suite
   on Linux before a deploy.
6. **Bosses of Acts 2-10 all have trials** (wave 15 added the
   Compiler's, the last). Not built, by decision D1: the design doc's
   combat flavour (Hydra regrowth, Cliphy the imp, a fourth Warden phase,
   the Sanity meter, the Compiler's HP and timed optimisation battle), the
   watching cast on the Throne plaza, credits over a `cargo doc`
   rendering. No voice-editor pass on the Act 9-10 lines yet.
7. Owner actions: TESS + RF check; domain; Android keystore;
   export the old claude.ai/design artboard before 2026-12-14; install
   `wasm-opt`, `butler`, Miri; decide the `repository` URL.

### How to work in this repo now
- Run cargo from Git Bash, not the PowerShell tool (C build scripts fail there).
- Never merge into the checkout while a build is running in it.
- Two tests are wall-clock sensitive under load (being fixed in wave 4):
  `compile_api_client::full_build_queue_is_busy`,
  `dialogue::f_opens_dialogue_then_last_line_opens_the_mission`.
- Look at the game only via `scripts/capture-scenes.ps1` (unfocused,
  self-driving). Never inject input or steal focus.

---

## Session 2026-11-13
2026-11-13 — **Full repo review, finish plan, art tranche 1 designed.**

**Read `docs/FINISH-PLAN.md` first.** It replaces the "What's Next" list below
and carries the verified state table, the art review, the asset manifest, the
Opus work tracks and the four decisions waiting on Matt.

### What was done this session (2026-11-13)
- **Plan:** `docs/FINISH-PLAN.md`.
- **Art tranche 1 (DRAFT, not wired, awaiting Matt's approval)** on the design
  canvas https://claude.ai/artifact/G67kHCdEaPS6KpYnD3jA1k — sources in
  `design/art/refs/ref-66-terrain.jsx` (22-tile terrain v2 with transparent
  grass-edge overlays), `ref-67-oak.jsx`, `ref-68-cottage.jsx`,
  `ref-69..74-*kit.jsx` (one costume-kit exemplar per Act 3–8).
- **`tools/render-refs`:** tile-sheet support (`REF<NN>_TILES` + `REF<NN>_COLS`
  pack into an atlas) and row-numbered width errors. clippy `-D warnings` and
  the crate's 6 tests pass. No tests cover `render_tiles` yet (plan task A1).
- **Nothing is committed.** Working tree: the 9 draft refs, `main.rs`, the plan,
  this file.

### Findings that change earlier notes
- `claude.ai/design` standalone **closes by 2026-12-14**. The six remaining
  rough-9 conversions (blocker 1 below) depend on an artboard there: export or
  convert before that date. New art goes through the design canvas above.
- `render-refs` cannot reproduce REF-03, 04, 05, 06, 08, 09, 10; their PNGs are
  leftovers from the deleted browser harness.
- Web and mobile builds grade with the offline pattern grader only.
- compile-api has no deploy artefacts, auth, rate limiting or telemetry.
- All 57 missions carry `prereq: None`; whether strict-linear gating is real
  was not verified this session.

---

## Prior session
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

<!-- ================================================================== -->
<!-- ⚠️  OUT-OF-TREE APPEND — NOT part of Pledge & Crown development.    -->
<!-- Written from OUTSIDE this repo by the job-hunter-mcp project        -->
<!-- (J:\job-hunter-mcp) via Claude Code on 2026-07-01.                  -->
<!-- Everything ABOVE this line is real P&C project state. Everything    -->
<!-- BELOW is a personal interview-prep plan that happens to use this    -->
<!-- game as its training ground. Do not let it pollute P&C planning.    -->
<!-- ================================================================== -->

# ⚠️ OUT-OF-TREE APPEND — Rust Interview Self-Prep Plan
> **PROVENANCE:** This section did **not** originate in the Pledge & Crown repo.
> It was appended by the **job-hunter-mcp** project (`J:\job-hunter-mcp`) on
> **2026-07-01**. It is Matt's personal prep plan, filed here because P&C *is*
> the training tool. It is **not** a P&C feature, task, or roadmap item — future
> P&C sessions should ignore it for project planning. Delete freely once used.

## Why this is here

Matt is going after Rust-heavy SWE roles: **xAI** (Forward Deployed Engineer – X
API; Backend Engineer – API; Software Engineer – Internal Tools), **ZoomInfo**
(Sr SWE, API & MCP Platform), **Genius Sports** (Sr SWE, Rust), **OP Labs**
(Sequencing Protocol Engineer, Rust).

The stated fear: *"without Claude I wouldn't be viable."* The resolution:

> **Viability on the job ≠ viability in the interview.** On the job, everyone
> uses AI (ZoomInfo is literally hiring for *MCP* fluency). The only AI-free gate
> is a ~45-min live interview. That gate is **prep-able**, not a verdict.

And the prep tool already exists — **this game.** Acts 1–10 teach most of the
Rust Book through Ch. 20, which is the exact surface an interview probes (not
modules/visibility, Book ch. 7, or writing tests, ch. 11: see
`design/01-curriculum.md`, "What is not taught"). So:

> **The plan is: play Pledge & Crown, unaided, as the interview gym.**

---

## Part A — The AI-free Rust gym (mapped to this game's own acts)

Run the missions **with Claude/Copilot OFF.** The game already grades you. If you
can clear these acts unaided, you can clear a whiteboard.

| Interview topic (what they'll probe)        | Play this Act | Missions to drill unaided                        | Roles that lean on it |
|---------------------------------------------|---------------|--------------------------------------------------|-----------------------|
| Ownership / borrows / lifetimes             | **Act 2**     | First Move, Mutable Quill, Lifetime Lantern      | ALL Rust roles        |
| Structs / enums / exhaustive `match`        | Act 3         | impl_method, enum_data_match, if_let, while_let  | ALL                   |
| Traits / generics / `dyn` / assoc types     | **Act 4**     | trait_def, generic_fn, dyn_trait, assoc_type     | Genius, OP Labs, xAI   |
| `Result`/`Option`/`?` / custom errors       | Act 5         | result_match, custom_error, from_error, and_then | ALL (idiomatic error handling is table stakes) |
| Collections + iterator chains + closures    | **Act 6**     | iter_filter, iter_fold, closure_move, iter_zip   | ALL                   |
| Threads / `Arc<Mutex>` / channels / async   | **Act 7**     | thread_spawn, arc_mutex, mpsc_channel, async_fn  | xAI Backend, OP Labs (consensus), Genius |
| Smart pointers / interior mutability / cycles | Act 8       | box_basic, rc_refcell, weak_ref                  | Genius, OP Labs        |
| `unsafe` / FFI / macros / `Drop`/`Send`/`Sync` | Act 9 (WIP) | (concepts; tutorials live even if missions pending) | xAI Backend/Inference, OP Labs |
| Perf / profiling / alloc reduction / `no_std` | Act 10 (WIP)| (concepts)                                       | xAI Backend/Inference  |

**Bold acts = the ones interviewers actually camp on.** If time is short, drill
Acts **2, 4, 6, 7, 8** in that priority. Those five are the whole ballgame.

**Suggested cadence (2 weeks, ~1hr/day):**
- Days 1–3: Act 2 (ownership) until Lifetime Lantern is reflexive. This is *the*
  Rust gate — if you can explain "why does this need `'a`" out loud, you're 60% there.
- Days 4–5: Act 4 (traits/generics/dyn). Be able to say when you'd pick a generic
  bound vs a `Box<dyn Trait>` and why — the Chimera boss is literally that question.
- Days 6–7: Act 6 (iterators). Rewrite one real loop from Wraith as a pure chain,
  no intermediate vars (Forge-Golem rules).
- Days 8–10: Act 7 (concurrency). Explain a deadlock and how you'd dodge it
  (Lighthouse Crisis). xAI Backend + OP Labs both probe this.
- Days 11–12: Act 8 (smart pointers). `Rc` cycle → `Weak` fix, out loud.
- Days 13–14: Mock round — pick one mission per act, no game hints, no AI, talk
  through it as if an interviewer is watching. Time yourself.

**Rule:** if you reach for AI during a mission, that's a flagged gap — note the
concept, re-drill that act. The game keeps score; so should you.

---

## Part B — The "explain your own work" script (this is where you WIN)

Most of these interviews open with *"walk me through something you built."* This
is not a coding-under-pressure test — it's a "do you understand your own work"
test, and you do. Rehearse a **5-minute cold walkthrough** for each lead artifact:

1. **Wraith Browser** (27K lines Rust, 130 MCP tools, CDP + native engine).
   - Architecture: engine abstraction (native vs cdp-attach), the MCP tool surface,
     the stealth/anti-detect layer. Have one hard bug ready (the BR-6→BR-10
     react-select / reCAPTCHA saga is a *great* debugging story — real, specific,
     shows systems reasoning).
2. **ClaudioOS** (bare-metal Rust agent host, `no_std` crates).
   - Why bare metal, what `no_std` cost you, the trade-offs. Ties straight to
     xAI Backend/Inference and Act 10's `no_std` outpost.
3. **10+ MCP servers + this job-hunter orchestrator.**
   - THE lead for **xAI FDE – X API** and **ZoomInfo MCP Platform.** You've
     *authored* the thing they're staffing for. Frame: protocol design, tool
     ergonomics, why an agent-first API differs from a REST API.
4. **Kalshi "Pixie Dust"** (trading engine, 20x, autonomous).
   - Shows end-to-end ownership + real stakes. Good for "tell me about impact."
5. **Pledge & Crown itself** (this repo — Bevy, WASM, a compiler-grading sandbox).
   - You built a *Rust teaching engine.* That is an absurdly good signal for a
     Rust role: you understand the language well enough to grade it. Mention the
     strict-linear curriculum gating and the compile-real sandbox.

For each: **Context → why you built it → hardest technical decision → one bug you
chased and how.** Write these out once; say them out loud three times.

---

## Part C — Per-role focus (where to spend the marginal hour)

- **xAI – FDE, X API** *(best fit anywhere right now)* — Lead Part B #3 (MCP/SDK/
  DevEx). Light on algorithms, heavy on "can you build developer-facing tooling
  and talk to customers." You've done both. Prep the *narrative*, not leetcode.
- **xAI – Backend Engineer, API** — Expert Rust + high-throughput serving + agent
  orchestration. Drill Acts **6, 7, 8**; lead the inference-rig (8×P100) + Wraith
  concurrency stories. Highest raw-Rust bar of the xAI three.
- **xAI – Internal Tools** — Lowest pedigree bar ("all levels," values shipping).
  Lead this hunter + Wraith + OpenClaw ecosystem. Most realistic door-opener.
- **ZoomInfo – Sr SWE, API & MCP Platform** — Your home turf; least fear-justified.
  Lead MCP authorship. Prep: API/protocol design tradeoffs.
- **Genius Sports – Sr SWE, Rust** — General Rust fluency. Acts 2, 4, 6, 7, 8.
- **OP Labs – Sequencing Protocol Engineer** *(the real stretch — and it's NOT
  about Claude)* — This one needs **distributed-systems / consensus / L2 sequencer
  domain depth**, not just Rust. If pursued, budget separate study time on the
  Optimism sequencer + rollup basics. Deprioritize vs the others.

---

## Part D — Interview-day framing

- Think out loud; ask clarifying questions. Interviewers grade *reasoning*, not
  just the final code.
- No AI in the room — but **everyone** knows you use it daily. If it comes up,
  frame it honestly: *"I'm leveraged by AI the same way every engineer here is;
  here's a system I architected and can walk you through line by line."* That's
  strength, not apology.
- The whiteboard tests fundamentals the game already drilled. Trust the reps.

**Done-signal:** you can clear Acts 2/4/6/7/8 missions with AI off, and talk
through Wraith's architecture cold for 5 minutes. Hit that and the fear is retired.

<!-- END OUT-OF-TREE APPEND (job-hunter-mcp, 2026-07-01) -->

