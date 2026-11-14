# HANDOFF.md

## Last Updated
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

And the prep tool already exists — **this game.** Acts 1–10 teach the Rust Book
through Ch. 20, which is the exact surface an interview probes. So:

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

