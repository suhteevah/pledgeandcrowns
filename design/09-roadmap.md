# 09 — Roadmap to the full game

*Written 2026-11-18, after the free demo went public. Supersedes the
day-by-day plan in `06-mvp-scope.md` and the phase dates in
`07-monetization.md` (their prices and sequence still stand).*

## Where the game is

- All ten acts are playable as walk, talk, code: 96 missions, 71 NPCs, a
  staged trial for every boss, an epilogue. The free demo (Acts 1-2, 24
  missions) is public on itch.io and grades with real `rustc` on our server.
- Cut by decision D1 and not built: turn-based combat, the spellbook,
  crafting, town building, party stats and equipment, Cliphy hints,
  leaderboards, the Temple (Act 11).
- Not taught: modules (Book ch. 7) and writing tests (ch. 11).
- Nobody has played the game end to end. Audio has never been heard by a
  person. No native package exists (no asset bundling, no installer).

### How it looks today

Characters are the strong part: 71 distinct NPCs, kit exemplars per act,
two 64 px bosses, two landmarks. The world is the weak part, and it is the
biggest gap between this and a game people pay for:

- The whole world is drawn with four ground types (grass, cobble, dirt,
  flagstone) and four props (cottage, oak, Sealed Stacks, Throne Dais).
- Every act's district is the same pale flagstone plaza with its cast
  standing in a grid. The Forge has no forge, the Wharf has no water, the
  Tavern has no tavern. Only the costumes say which act you are in.
- NPCs do not move. Only the player and Ferris have walk frames.
- One music track for the world, one for the title, one for the epilogue.
- UI text is a system font; the title screen is a first pass.

## What 1.0 is

**The ten acts as a finished, paid game:** each district looks and sounds
like its place, the bosses have an entrance and an exit, the game installs
and runs natively on Windows and Linux and in the browser, and a stranger
can buy it and finish it without help.

Still cut from 1.0 (recommended): combat, spellbook, crafting, town
building, stats and equipment, leaderboards. They are each a second game's
worth of work and none of them teaches Rust better than the trials do.
The Temple stays a post-1.0 expansion.

Added to 1.0 because the pitch promises "learn Rust": a short module/tests
chapter (see R3), so the two skipped Book chapters are covered.

## Milestones

Each one ends in something shippable. Order matters more than dates; rough
sizes assume the current way of working (Opus agents in worktrees, CI as
the gate, art approved on the canvas).

### R0 — Hold the demo up (now, days)
- Owner plays the public demo start to finish; fix what that finds.
- Server: uptime check with a phone alert, a journald size cap, a small
  load test (the box is shared with other production services), a metric
  for toolchain crashes.
- A feedback link and a "tell me when the full game ships" email capture
  on the itch page and the demo's end screen.
- Cover and screenshots on the itch page.

### R1 — Every district is a place (the art milestone, the long pole)
The world pass described under "Art plan" below, plus the engineering it
needs: district ground types, a data-driven prop table with collision and
y-sort, cast placement that uses the props (the smith at the anvil, not in
a grid), one capture scene per district.
Exit test: a screenshot from any act can be told apart from any other with
the characters hidden.

### R2 — Paid early access on itch (first money)
- Native packaging for Windows: release exe, bundled assets, the baked
  server URL, a zip that runs on a clean machine. Linux next.
- How the full game is sold: a paid itch download (native) plus the full
  web build on the same paid page. The free page keeps the demo and says
  where to buy. Save files carry over from demo to full.
- Server capacity for paying players: a second builder host or a queue
  page, rate limits per token, a status line in the game when grading is
  slow.
- `ci-slow.ps1` green and a rust-lead review on the release commit.
- Price and timing are the owner's call (07 says $14.99 early access).

### R3 — Depth and finish
- Boss entrances and exits (intro and defeat frames for the nine bosses,
  starting with the Borrow Checker, the Librarian and the Compiler).
- NPC idle animation (two frames) and a few walkers per district.
- Music per district (the synthwave pipeline exists) and a real SFX set.
- Villager dialogue for Acts 9-10; a voice pass over all ten acts.
- Modules and tests chapter: about six missions, placed in the Guildhall
  (Act 3 is where code first gets organised).
- Hints: after two failed compiles, a nudge from the mission giver before
  "show solution". (Cliphy as a character waits for post-1.0.)
- Settings people expect: key remap, text size, volume per channel,
  colour-safe result banners, gamepad for walking and menus.
- Pixel font and the title redraw.

### R4 — Steam
- Store page, capsule art, trailer, screenshots (the $100 fee and the page
  are the owner's).
- Steam build: achievements per act, cloud saves, Steam Deck check.
- Launch as 1.0 at the full price once R3 is in.

### R5 — After 1.0
The Temple (Act 11), Cliphy, the team dashboard for B2B seats, Android
(the bundle builds; signing is open), macOS, localisation.

## Art plan (R1 in detail)

Rule for the whole pass: each district gets **one ground, one landmark and
three to five props**, all from the 32-colour palette, drawn through the
existing `ref-NN.jsx` → render-refs → canvas approval path. One district
is drawn and approved first as the style sample; the rest follow it.

| Wave | District | Ground | Landmark | Props |
|---|---|---|---|---|
| A1 (sample) | The Forge (Act 6) | soot-dark slab | the great furnace | anvil, quench trough, coal heap, tool rack |
| A2 | Hearthstone Square (Act 1) | exists | bell tower | well, woodpile, signpost, chicken coop |
| A2 | Market Row (Act 1) | exists | market stall | crates, barrels, handcart |
| A2 | The Commons (Act 2) | exists | the Checker's Gate | lantern post, bench, fence |
| A3 | Guildhall (Act 3) | chequered hall floor | guildhall front | ledger desk, notice board, crates |
| A3 | Mage's Tower (Act 4) | violet-veined slab | the tower base | orrery, rune stone, bookstand |
| A3 | Tavern (Act 5) | plank floor | tavern front and sign | tables, barrels, hearth |
| A4 | Wharf (Act 7) | dock planks, water (animated) | crane and mooring | bollards, nets, crates, a moored boat |
| A4 | Vault (Act 8) | iron-banded slab | the vault door | chests, chains, lockboxes |
| A4 | Library (Act 9) | exists + carpet runner | Sealed Stacks (done) | lectern, shelves, reading desk |
| A4 | Throne (Act 10) | exists + runner | Throne Dais (done) | columns, braziers, banners |
| A5 | Characters | | | Borrow Checker sprite, boss intro/defeat frames, NPC idle frames |
| A5 | UI and store | | | pixel font, title redraw, icons, Steam capsules |

About 55 new pieces before characters. (There is no bridge in the world: Acts 1-2 play across Hearthstone Square, Market Row and The Commons.)

Status 2026-11-19: waves A1 to A4 are done. Every district is dressed (REF-97..172), all approved by the owner and in the game, with each cast gathered at props, 29 ambient townsfolk (REF-173..179, seven figures on bodies of their own) and the Wharf's harbour water. REF-122 (Guildhall floor) and REF-154 (Vault floor) are second drawings: the owner agreed both were too loud. The itch demo is at 0.2.1 and covers the first three districts. Next in art is wave A5, characters, then UI and store.

## Risks

- **The server is one shared box.** A burst of players, or one abusive
  one, competes with the owner's other services. R0 measures it; R2 moves
  or doubles it.
- **Art volume.** 55 props is more than everything drawn so far. The
  sample district exists to catch a wrong direction after 6 pieces, not 55.
- **Nobody has played it.** Ten acts of untested pacing. R0 starts that;
  two or three outside players before R2 would be worth more than any
  feature.
- **Unverified today:** native release build, Tauri gate, audio, reboot
  survival of the server units, the slow test suites at the current commit.

## Decisions that are the owner's

1. Early-access price and when to start charging (R2).
2. Steam: the fee and the page (R4).
3. Whether combat stays cut for 1.0 (recommended: yes).
4. A domain, so the compile server and the game stop living on a borrowed
   hostname.
5. Art approval, piece by piece, on the design canvas.
