# 04b — Art Deliverables Spec
*Companion to `03-art-style-bible.md` and `04-art-handoff-prompts.md`. This doc is the contractor-facing checklist: what to deliver, how to format it, and how it gets approved.*

**Doc version:** 1.0 — 2026-04-25.

## 1. Master asset manifest (MVP only)

Total: ~180 deliverables for MVP (Acts 1–2 + Borrow Checker boss). Numbers are sprite *files*, not frames. Multi-frame animations count as one row here; per-frame counts live in §3.

### 1.1 Characters (priority A — blocks gameplay)

| ID | Asset | Resolution | States required | Priority |
|----|-------|------------|-----------------|----------|
| C-01 | Player (default) | 32×32 | idle, walk-4dir, attack, cast, hit, death | A |
| C-02 | Ferris guide | 32×32 | idle, walk-4dir, talk | A |
| C-03 | Borrow Checker NPC (Act 2 boss) | 64×64 | idle, intro, attack-A, attack-B, phase-2-transform, defeat | A |
| C-04 | Hearthstone village elder | 32×32 | idle, talk | A |
| C-05 | Trait Mage | 32×32 | idle, talk, cast | A |
| C-06–C-10 | 5 named villager NPCs | 32×32 | idle, walk-4dir | B |
| C-11 | Goblin (generic enemy) | 32×32 | idle, walk-4dir, attack, hit, death | A |
| C-12 | Slime | 24×24 | idle, hop, hit, death | A |
| C-13 | Bandit | 32×32 | idle, walk-4dir, attack, hit, death | A |
| C-14 | Forest wolf | 32×32 | idle, walk-4dir, attack, hit, death | B |
| C-15 | Dust mite (tutorial enemy) | 16×16 | idle, hit, death | A |

### 1.2 Tilesets (priority A — blocks level building)

| ID | Asset | Tile size | Tiles required | Priority |
|----|-------|-----------|----------------|----------|
| T-01 | Hearthstone Village exterior | 16×16 | 64 tiles (grass, dirt, cobble, fence, well, flowers, water edge, building walls/roofs) | A |
| T-02 | Village interior | 16×16 | 32 tiles (wood floor, stone walls, hearth, doors, furniture) | A |
| T-03 | Trait Mage's Tower interior | 16×16 | 48 tiles (purple-gold marble, magic runes inset, stairs, pillars) | A |
| T-04 | Bridge / borrow boss arena | 16×16 | 32 tiles (stone bridge, teal mist edge, broken railings) | A |
| T-05 | Forest path | 16×16 | 48 tiles (grass variants, trees, stumps, brush, path) | A |
| T-06 | World map painted scene | 320×180 single image | n/a | B |

### 1.3 UI (priority A — blocks editor/menus)

| ID | Asset | Resolution | Notes | Priority |
|----|-------|------------|-------|----------|
| U-01 | Code editor frame | 640×360 (scalable) | Parchment bg, rusty borders, 9-slice corners | A |
| U-02 | Dialogue box frame | 480×96 (9-slice) | Same border family as U-01 | A |
| U-03 | Health bar | 96×16 | Empty + full + 10 fill states (or a shader-friendly mask) | A |
| U-04 | Mana/cast bar | 96×16 | Same shape as U-03, teal fill | A |
| U-05 | Button (idle/pressed) | 96×24 | 2 states | A |
| U-06 | Inventory slot | 24×24 | Empty + selected (2 states) | A |
| U-07 | Title screen | 640×360 | Wordmark + hero shot (REF-10) | A |
| U-08 | Pause menu frame | 320×240 (9-slice) | A |
| U-09 | Quest log frame | 320×240 (9-slice) | B |

### 1.4 Icons (priority B — gameplay polish)

| ID | Asset | Resolution | Count | Priority |
|----|-------|------------|-------|----------|
| I-01 | Item icons (potions, scrolls, keys) | 16×16 | 24 | B |
| I-02 | Spell icons | 24×24 | 12 | B |
| I-03 | Status effect icons (poisoned, blessed, borrowed) | 16×16 | 8 | B |
| I-04 | Map markers (town, dungeon, boss, NPC) | 16×16 | 6 | B |

### 1.5 Effects (priority B — juice)

| ID | Asset | Resolution | Frames | Priority |
|----|-------|------------|--------|----------|
| E-01 | Hit-spark | 24×24 | 4 | B |
| E-02 | Heal sparkle | 24×24 | 6 | B |
| E-03 | Magic cast aura (player) | 32×32 | 6, looping | B |
| E-04 | Borrow Checker glow (his signature aura) | 64×64 | 8, looping | A |
| E-05 | Door open dust | 16×16 | 4 | C |

**Priority key:** A = blocks MVP. B = ships in MVP if time permits, otherwise placeholder. C = post-launch.

## 2. Deliverable file format

- **Format:** PNG-32 with alpha channel. No JPEG, no WebP.
- **Color space:** sRGB. No embedded ICC profiles.
- **Bit depth:** 8 bits/channel. No 16-bit PNGs.
- **Background:** fully transparent (alpha 0) outside the sprite silhouette. No checkerboard, no magenta key.
- **Anti-aliasing:** none inside sprites. Edges are pixel-perfect per the bible.
- **Single frame per file.** Do NOT pre-pack into sprite sheets — the build pipeline (`bevy_asset_loader` + `bevy_titan` or equivalent atlas packer) packs at build time. Submitting pre-packed sheets forces re-slicing.
- **Anchor / pivot:** bottom-center for characters and enemies. Top-left for tiles and UI elements. Center for icons and effects.
- **Padding:** zero transparent padding around sprites in source files. The atlas packer adds bleed.
- **Color discipline:** every pixel must come from the 32-color "Heraldic Code" palette (`03-art-style-bible.md` §Palette). A palette-conformance script will check this on PR; off-palette deliveries are auto-rejected.

## 3. Animation frame specs

The bible (`03-art-style-bible.md` §Animation conventions) sets timing principles. This section is the contractor's authoritative frame-count table.

| Action | Frames | ms/frame | Loops? | Notes |
|--------|--------|----------|--------|-------|
| Idle | 2 | 600 | yes | Breathing only — no walk-bob |
| Walk (per direction) | 4 | 150 | yes | 4 directions; right is left flipped, only deliver 3 |
| Attack | 5 | 100 | no | Frame 1 = wind-up, frame 5 = recovery |
| Cast | 6 | 100 | yes (during channel) | Magic FX is a separate layer (E-03) |
| Hit | 1 | 60 | no | White-flash, then return to idle |
| Death | 4 | 120 | no | Final frame holds for 1s before despawn |

**Bosses:** add intro (8 frames, ~100ms each, no loop), phase-2 transform (10 frames, 100ms each, no loop), and defeat (8 frames, 150ms each, no loop). Per-boss deviations live in `prompts/bosses.md`.

**Effects:** see §1.5. Each effect lists its frame count.

## 4. Filename + folder convention

```
assets/
  sprites/
    characters/
      player_idle_0.png
      player_idle_1.png
      player_walk_down_0.png
      ...
    enemies/
      goblin_attack_0.png
      ...
    npc/
      borrow_checker_idle_0.png
      borrow_checker_phase2_0.png
      ...
    bosses/
      borrow_checker/        # boss subdirs allowed when frame counts are large
        intro_0.png
        ...
  tiles/
    village/
      grass_0.png
      cobble_corner_ne.png
      ...
  ui/
    editor_frame_9slice.png    # 9-slice files use the _9slice suffix
    button_idle.png
    button_pressed.png
  icons/
    items/
      potion_health.png
      ...
  fx/
    hit_spark_0.png
    ...
```

Rules:
- All lowercase. Underscores between words. No spaces, no dashes.
- Frame index is `_0`, `_1`, … — zero-indexed, no zero-padding.
- Direction suffixes: `_down`, `_up`, `_left`. (Right is flipped from left at runtime.)
- A ref drawn as a tile sheet ships as the one atlas PNG render-refs writes (e.g. `player/player_walk.png`, `tiles_village_v3.png`); its frames are atlas indices in the ref's declaration order, not separate files.
- 9-slice frames use the `_9slice.png` suffix and ship with a sibling `.json` describing border insets.
- Boss assets that need >20 files get their own subdirectory under `bosses/`.

## 5. Approval workflow

**Current flow (since 2026-11-13).** Standalone claude.ai/design closes by 2026-12-14, so new art no longer starts there. The orchestrating model (Fable) designs art as draft ASCII-grid refs; Matt is the sole approver; implementers (including the `art-lead` agent) wire art only after approval.

1. **Draft.** Write `design/art/refs/ref-NN-<slug>.jsx` with a `DRAFT` header line. A slug must not contain `-v` (render-refs parses it as a version suffix). Tile sheets use `const REF<NN>_TILES = { name: ['row', ...], ... };` plus `const REF<NN>_COLS = n;` and are packed row-major into an atlas.
2. **Preview.** `cargo run -p render-refs --release --bin render-refs` with `RENDER_REFS_OUT=<scratch dir>`, so unapproved PNGs stay out of `design/art/refs/png/`. Any ref that fails to render (outside a six-file legacy allow-list) makes the binary exit non-zero.
3. **Review.** Upload the preview PNGs to the design canvas, https://claude.ai/artifact/G67kHCdEaPS6KpYnD3jA1k. Matt approves or rejects on the board.
4. **Land.** On approval: replace the `DRAFT` header with an approved status line, render into `design/art/refs/png/`, copy into `game/assets/`, wire it in, and run CI.

The rules below (1-4) describe the original contractor-era flow. The palette and naming checks in rule 4 still apply; the contractor batch mechanics do not.

1. **Reference pass first.** The 10 reference images in `04-art-handoff-prompts.md` §"Reference Image Set" must ship before any §1 asset begins. Matt is the sole approver. Two-round limit: round 1 = first delivery, round 2 = revisions per Matt's notes. If round 2 fails, the bible is amended and the contractor restarts from a new reference brief.
2. **Bulk gen in batches of 20.** Each batch is reviewed against the approved reference set. Reject rate >25% triggers a style-bible review meeting before the next batch.
3. **Per-asset feedback** is given inline in a shared sheet (one row per asset, columns: ID, status, notes, revision count). Three revisions max per asset; further iteration kicks the asset back to the bible-amend path.
4. **Done = merged.** Asset is "done" when the file lands in `assets/sprites/` on `main` and the palette-conformance + naming-convention checks both pass on the pre-commit hook.

## 6. IP and licensing

- **Default:** all art is **work-for-hire**. Contractor assigns full copyright to Chico Cell Repair LLC at delivery. Standard agreement language; no carve-outs for portfolio use without prior written approval.
- **Portfolio rights:** contractor may display delivered art in a personal portfolio after the game's public launch (currently scheduled day 30 of MVP), with attribution to "Pledge & Crown" and a link to the project page.
- **Game licensing context:** the game binary + assets ship under a proprietary license (or MIT, pending HANDOFF open Q3). The compile-API server is dual-licensed AGPL+CLA. Art assets are distributed *with* the game binary; the contractor's work-for-hire assignment is what makes that distribution legal regardless of which game-binary license Matt picks.
- **No third-party assets without explicit clearance.** No traced reference, no Photoshopped stock, no AI-upscaled CC0 sprites unless the source license is added to `THIRD_PARTY_LICENSES.md` and the contractor has confirmed the license is compatible with proprietary distribution.

## 7. Out of scope (explicitly)

- Music and SFX. (Separate audio bible TBD — likely a Phase 2 hire after the visual reference pass clears.)
- Marketing assets — Steam capsule, itch.io banner, social-media cards. These are post-MVP, post-TESS-clearance, and depend on the final wordmark lockup.
- 3D / vector / animation rigs. The game is hand-pixeled 2D. No Spine, no DragonBones, no Aseprite-only proprietary formats — final delivery is flat PNGs.

## 9. Rendered REF manifest

Refs landed against the bible. Source `.jsx` lives at `design/art/refs/ref-NN-<slug>.jsx`; rendered PNG at `design/art/refs/png/REFNN.png`. Game-asset wiring (where applicable) is in `game/src/assets.rs::ALL_SPRITE_PATHS` and `game/src/plugins/npc.rs::NPC_ROSTER`.

| REF | Asset | Resolution | Source JSX | Spec | Wired sprite path |
|-----|-------|------------|------------|------|-------------------|
| 01 | Player character | 32×32 | ref-01-player.jsx | (bible §Reference set) | `sprites/player/player_idle_0.png` |
| 02 | Ferris guide | 32×32 | ref-02-ferris.jsx | (bible §Reference set) | `sprites/npc/ferris_idle_0.png` |
| 03 | Borrow Checker NPC | 64×64 | ref-03-borrow-checker.jsx | (bible §Reference set) | `sprites/npc/borrow_checker_idle_0.png` |
| 04 | Hearthstone Village tiles | 16×16 ×16 | ref-04-tiles-village.jsx | (bible §Reference set) | `sprites/tiles_village.png` (superseded by REF-66; file kept, unused) |
| 05 | Trait Mage's Tower tiles | 16×16 ×16 | ref-05-tiles-tower.jsx | (bible §Reference set) | (Act 4, not yet loaded) |
| 06 | Code editor frame | 640×360 | ref-06-editor.jsx | (bible §Reference set) | (egui, not bevy sprite) |
| 07 | Goblin enemy | 32×32 | ref-07-goblin.jsx | (bible §Reference set) | (Act 1 enemy, not yet loaded) |
| 08 | Healing potion icon | 16×16 | ref-08-potion.jsx | (bible §Reference set) | (inventory, not yet loaded) |
| 09 | World map cinematic | 320×180 | ref-09-worldmap-v2.jsx | (bible §Reference set) | `sprites/worldmap.png` (asset present, not registered) |
| 10 | Title screen wordmark | 640×360 | ref-10-title.jsx | (bible §Reference set) | `sprites/title.png` |
| 11 | The Smith | 32×32 | ref-11-smith.jsx | specs/smith.md | `sprites/npc/smith_idle_0.png` |
| 12 | The Cartographer | 32×32 | ref-12-cartographer.jsx | specs/cartographer.md | `sprites/npc/cartographer_idle_0.png` |
| 13 | The Trait Mage | 32×32 | ref-13-trait-mage.jsx | specs/trait-mage.md | `sprites/npc/trait_mage_idle_0.png` |
| 14 | The Bellringer | 32×32 | ref-14-bellringer.jsx | specs/bellringer.md | `sprites/npc/bellringer_idle_0.png` |
| 15 | The Cooper | 32×32 | ref-15-cooper.jsx | specs/cooper.md | `sprites/npc/cooper_idle_0.png` |
| 16 | The Oracle | 32×32 | ref-16-oracle.jsx | specs/oracle.md | `sprites/npc/oracle_idle_0.png` |
| 17 | The Herald | 32×32 | ref-17-herald.jsx | specs/herald.md | `sprites/npc/herald_idle_0.png` |
| 18 | The Twin | 32×32 | ref-18-twin.jsx | specs/twin.md | `sprites/npc/twin_idle_0.png` |
| 19 | The Tinker | 32×32 | ref-19-tinker.jsx | specs/tinker.md | `sprites/npc/tinker_idle_0.png` |
| 20 | The Heraldic Sage | 32×32 | ref-20-heraldic-sage.jsx | specs/heraldic-sage.md | `sprites/npc/heraldic_sage_idle_0.png` |
| 21 | The Forgewright | 32×32 | ref-21-forgewright.jsx | specs/forgewright.md | `sprites/npc/forgewright_idle_0.png` |
| 22 | The Linguist | 32×32 | ref-22-linguist.jsx | specs/linguist.md | `sprites/npc/linguist_idle_0.png` |
| 23 | The Pilgrim | 32×32 | ref-23-pilgrim.jsx | specs/pilgrim.md | `sprites/npc/pilgrim_idle_0.png` |
| 24 | The Drillmaster | 32×32 | ref-24-drillmaster.jsx | specs/drillmaster.md | `sprites/npc/drillmaster_idle_0.png` |
| 25 | The Reckoner | 32×32 | ref-25-reckoner.jsx | specs/reckoner.md | `sprites/npc/reckoner_idle_0.png` |
| 26 | The Quartermaster | 32×32 | ref-26-quartermaster.jsx | specs/quartermaster.md | `sprites/npc/quartermaster_idle_0.png` |
| 27 | The Auditor | 32×32 | ref-27-auditor.jsx | specs/auditor.md | `sprites/npc/auditor_idle_0.png` |
| 28 | The Chronicler | 32×32 | ref-28-chronicler.jsx | specs/chronicler.md | `sprites/npc/chronicler_idle_0.png` |
| 29 | The Alchemist | 32×32 | ref-29-alchemist.jsx | specs/alchemist.md | `sprites/npc/alchemist_idle_0.png` |
| 30 | The Guildmaster | 32×32 | ref-69-guildkit.jsx (kit exemplar; supersedes ref-30-guildmaster.jsx) | specs/guildmaster.md | `sprites/npc/guildmaster_idle_0.png` (REF69.png, wired 2026-11-15) |
| 31 | The Recruiter | 32×32 | ref-31-recruiter-v2.jsx (supersedes ref-31-recruiter.jsx) | specs/recruiter.md | `sprites/npc/recruiter_idle_0.png` (REF31v2.png, tranche 2) |
| 32 | The Locksmith | 32×32 | ref-32-locksmith-v2.jsx (supersedes ref-32-locksmith.jsx) | specs/locksmith.md | `sprites/npc/locksmith_idle_0.png` (REF32v2.png, tranche 2) |
| 33 | The Porter | 32×32 | ref-33-porter-v2.jsx (supersedes ref-33-porter.jsx) | specs/porter.md | `sprites/npc/porter_idle_0.png` (REF33v2.png, tranche 2) |
| 34 | The Surveyor | 32×32 | ref-34-surveyor-v2.jsx (supersedes ref-34-surveyor.jsx) | specs/surveyor.md | `sprites/npc/surveyor_idle_0.png` (REF34v2.png, tranche 2) |
| 35 | The Armorer | 32×32 | ref-35-armorer-v2.jsx (supersedes ref-35-armorer.jsx) | specs/armorer.md | `sprites/npc/armorer_idle_0.png` (REF35v2.png, tranche 2) |
| 36 | Vexis the Archmage | 32×32 | ref-36-vexis-v2.jsx (supersedes ref-36-vexis.jsx) | specs/vexis.md | `sprites/npc/vexis_idle_0.png` (REF36v2.png, tranche 2) |
| 37 | The Wandwright | 32×32 | ref-37-wandwright-v2.jsx (supersedes ref-37-wandwright.jsx) | specs/wandwright.md | `sprites/npc/wandwright_idle_0.png` (REF37v2.png, tranche 2) |
| 38 | The Conjurer | 32×32 | ref-70-towerkit.jsx (kit exemplar; supersedes ref-38-conjurer.jsx) | specs/conjurer.md | `sprites/npc/conjurer_idle_0.png` (REF70.png, wired 2026-11-15) |
| 39 | The Familiar | 32×32 | ref-39-familiar-v2.jsx (supersedes ref-39-familiar.jsx) | specs/familiar.md | `sprites/npc/familiar_idle_0.png` (REF39v2.png, tranche 2) |
| 40 | The Lanternkeeper | 32×32 | ref-40-lanternkeeper-v2.jsx (supersedes ref-40-lanternkeeper.jsx) | specs/lanternkeeper.md | `sprites/npc/lanternkeeper_idle_0.png` (REF40v2.png, tranche 2) |
| 41 | The Loremaster | 32×32 | ref-41-loremaster-v2.jsx (supersedes ref-41-loremaster.jsx) | specs/loremaster.md | `sprites/npc/loremaster_idle_0.png` (REF41v2.png, tranche 2) |
| 42 | The Barkeep | 32×32 | ref-71-tavernkit.jsx (kit exemplar; supersedes ref-42-barkeep.jsx) | specs/barkeep.md | `sprites/npc/barkeep_idle_0.png` (REF71.png, wired 2026-11-15) |
| 43 | The Bouncer | 32×32 | ref-43-bouncer-v2.jsx (supersedes ref-43-bouncer.jsx) | specs/bouncer.md | `sprites/npc/bouncer_idle_0.png` (REF43v2.png, tranche 2) |
| 44 | The Interpreter | 32×32 | ref-44-interpreter-v2.jsx (supersedes ref-44-interpreter.jsx) | specs/interpreter.md | `sprites/npc/interpreter_idle_0.png` (REF44v2.png, tranche 2) |
| 45 | The Mixologist | 32×32 | ref-45-mixologist-v2.jsx (supersedes ref-45-mixologist.jsx) | specs/mixologist.md | `sprites/npc/mixologist_idle_0.png` (REF45v2.png, tranche 2) |
| 46 | The Tabkeeper | 32×32 | ref-46-tabkeeper-v2.jsx (supersedes ref-46-tabkeeper.jsx) | specs/tabkeeper.md | `sprites/npc/tabkeeper_idle_0.png` (REF46v2.png, tranche 2) |
| 47 | The Cellarer | 32×32 | ref-47-cellarer-v2.jsx (supersedes ref-47-cellarer.jsx) | specs/cellarer.md | `sprites/npc/cellarer_idle_0.png` (REF47v2.png, tranche 2) |
| 48 | The Keymaster | 32×32 | ref-48-keymaster-v2.jsx (supersedes ref-48-keymaster.jsx) | specs/keymaster.md | `sprites/npc/keymaster_idle_0.png` (REF48v2.png, tranche 2) |
| 49 | The Sifter | 32×32 | ref-49-sifter-v2.jsx (supersedes ref-49-sifter.jsx) | specs/sifter.md | `sprites/npc/sifter_idle_0.png` (REF49v2.png, tranche 2) |
| 50 | The Smelter | 32×32 | ref-72-forgekit.jsx (kit exemplar; supersedes ref-50-smelter.jsx) | specs/smelter.md | `sprites/npc/smelter_idle_0.png` (REF72.png, wired 2026-11-15) |
| 51 | The Tallywright | 32×32 | ref-51-tallywright-v2.jsx (supersedes ref-51-tallywright.jsx) | specs/tallywright.md | `sprites/npc/tallywright_idle_0.png` (REF51v2.png, tranche 2) |
| 52 | The Riveter | 32×32 | ref-52-riveter-v2.jsx (supersedes ref-52-riveter.jsx) | specs/riveter.md | `sprites/npc/riveter_idle_0.png` (REF52v2.png, tranche 2) |
| 53 | The Bondsmith | 32×32 | ref-53-bondsmith-v2.jsx (supersedes ref-53-bondsmith.jsx) | specs/bondsmith.md | `sprites/npc/bondsmith_idle_0.png` (REF53v2.png, tranche 2) |
| 54 | The Dockmaster | 32×32 | ref-73-coastkit.jsx (kit exemplar; supersedes ref-54-dockmaster.jsx) | specs/dockmaster.md | `sprites/npc/dockmaster_idle_0.png` (REF73.png, wired 2026-11-15) |
| 55 | The Lighthouse Keeper | 32×32 | ref-55-lighthousekeeper-v2.jsx (supersedes ref-55-lighthousekeeper.jsx) | specs/lighthousekeeper.md | `sprites/npc/lighthousekeeper_idle_0.png` (REF55v2.png, tranche 2) |
| 56 | The Signaler | 32×32 | ref-56-signaler-v2.jsx (supersedes ref-56-signaler.jsx) | specs/signaler.md | `sprites/npc/signaler_idle_0.png` (REF56v2.png, tranche 2) |
| 57 | The Tidewatch | 32×32 | ref-57-tidewatch-v2.jsx (supersedes ref-57-tidewatch.jsx) | specs/tidewatch.md | `sprites/npc/tidewatch_idle_0.png` (REF57v2.png, tranche 2) |
| 58 | The Harbormaster | 32×32 | ref-58-harbormaster-v2.jsx (supersedes ref-58-harbormaster.jsx) | specs/harbormaster.md | `sprites/npc/harbormaster_idle_0.png` (REF58v2.png, tranche 2) |
| 59 | The Tideforecaster | 32×32 | ref-59-tideforecaster-v2.jsx (supersedes ref-59-tideforecaster.jsx) | specs/tideforecaster.md | `sprites/npc/tideforecaster_idle_0.png` (REF59v2.png, tranche 2) |
| 60 | The Vaultwright | 32×32 | ref-60-vaultwright-v2.jsx (supersedes ref-60-vaultwright.jsx) | specs/vaultwright.md | `sprites/npc/vaultwright_idle_0.png` (REF60v2.png, tranche 2) |
| 61 | The Sharekeeper | 32×32 | ref-61-sharekeeper-v2.jsx (supersedes ref-61-sharekeeper.jsx) | specs/sharekeeper.md | `sprites/npc/sharekeeper_idle_0.png` (REF61v2.png, tranche 2) |
| 62 | The Warden | 32×32 | ref-74-vaultkit.jsx (kit exemplar; supersedes ref-62-warden.jsx) | specs/warden.md | `sprites/npc/warden_idle_0.png` (REF74.png, wired 2026-11-15) |
| 63 | The Swapwarden | 32×32 | ref-63-swapwarden-v2.jsx (supersedes ref-63-swapwarden.jsx) | specs/swapwarden.md | `sprites/npc/swapwarden_idle_0.png` (REF63v2.png, tranche 2) |
| 64 | The Strongbox | 32×32 | ref-64-strongbox-v2.jsx (supersedes ref-64-strongbox.jsx) | specs/strongbox.md | `sprites/npc/strongbox_idle_0.png` (REF64v2.png, tranche 2) |
| 65 | The Ghostkeeper | 32×32 | ref-65-ghostkeeper-v2.jsx (supersedes ref-65-ghostkeeper.jsx) | specs/ghostkeeper.md | `sprites/npc/ghostkeeper_idle_0.png` (REF65v2.png, tranche 2) |
| 66 | Hearthstone Village terrain v2 | 16×16 ×22 (8-col atlas, 128×48) | ref-66-terrain.jsx | (header) | `sprites/tiles_village_v2.png` (file kept, unused; its tiles ship in the world via REF-76, indices 0-21) |
| 67 | Village oak (prop sprite) | 32×48 | ref-67-oak.jsx | (header) | `sprites/props/oak.png` |
| 68 | Hearthstone cottage (prop sprite) | 64×64 | ref-68-cottage.jsx | (header) | `sprites/props/cottage.png` |
| 69 | Costume kit, Act 3 Guildhall (Guildmaster) | 32×32 | ref-69-guildkit.jsx | (header) | `sprites/npc/guildmaster_idle_0.png` (The Guildmaster's sprite, wired 2026-11-15) |
| 70 | Costume kit, Act 4 Tower (Conjurer) | 32×32 | ref-70-towerkit.jsx | (header) | `sprites/npc/conjurer_idle_0.png` (The Conjurer's sprite, wired 2026-11-15) |
| 71 | Costume kit, Act 5 Tavern (Barkeep) | 32×32 | ref-71-tavernkit.jsx | (header) | `sprites/npc/barkeep_idle_0.png` (The Barkeep's sprite, wired 2026-11-15) |
| 72 | Costume kit, Act 6 Forge (Smelter) | 32×32 | ref-72-forgekit.jsx | (header) | `sprites/npc/smelter_idle_0.png` (The Smelter's sprite, wired 2026-11-15) |
| 73 | Costume kit, Act 7 Coast (Dockmaster) | 32×32 | ref-73-coastkit.jsx | (header) | `sprites/npc/dockmaster_idle_0.png` (The Dockmaster's sprite, wired 2026-11-15) |
| 74 | Costume kit, Act 8 Vault (Warden) | 32×32 | ref-74-vaultkit.jsx | (header) | `sprites/npc/warden_idle_0.png` (The Warden's sprite, wired 2026-11-15) |
| 75 | Plaza flagstone | 16×16 ×2 (2-col sheet, 32×16) | ref-75-flagstone.jsx | (header) | via REF-76: `sprites/tiles_village_v3.png` indices 22-23 (plaza interiors) |
| 76 | Village combined atlas (REF-66 + REF-75 + REF-97, mechanical copy, no new art) | 16×16 ×26 (8-col atlas, 128×64, row 4 part-empty) | ref-76-village-atlas.jsx | (header) | `sprites/tiles_village_v3.png` (the world's terrain atlas) |
| 77 | Player walk frames (down_0, down_2, up_0, up_1, up_2, left_0, left_1, left_2) | 32×32 ×8 (4-col atlas, 128×64) | ref-77-playerwalk.jsx | (header) | `sprites/player/player_walk.png` (atlas; cycles with `player_idle_0.png`, see below) |
| 78 | Ferris walk frames (walk_0, walk_2) | 32×32 ×2 (2-col atlas, 64×32) | ref-78-ferriswalk.jsx | (header) | `sprites/npc/ferris_walk.png` (atlas; cycles with `ferris_idle_0.png`) |
| 79 | Costume kit, Act 9 Forbidden Library | 32×32 | ref-79-librarykit.jsx | (header) | (kit exemplar, not wired; the Act 9 cast is drawn on it) |
| 80 | The Shelfmark | 32×32 | ref-80-shelfmark.jsx | (header) | `sprites/npc/shelfmark_idle_0.png` |
| 81 | The Envoy | 32×32 | ref-81-envoy.jsx | (header) | `sprites/npc/envoy_idle_0.png` |
| 82 | The Macro Monk | 32×32 | ref-82-macromonk.jsx | (header) | `sprites/npc/macromonk_idle_0.png` |
| 83 | The Sexton | 32×32 | ref-83-sexton.jsx | (header) | `sprites/npc/sexton_idle_0.png` |
| 84 | The Courier | 32×32 | ref-84-courier.jsx | (header) | `sprites/npc/courier_idle_0.png` |
| 85 | The Bookbinder | 32×32 | ref-85-bookbinder.jsx | (header) | `sprites/npc/bookbinder_idle_0.png` |
| 93 | The Librarian (Act 9 boss) | 64×64 | ref-93-librarian.jsx | (header) | `sprites/npc/librarian_idle_0.png` (native 64 px, like the Borrow Checker) |
| 86 | Costume kit, Act 10 Throne | 32×32 | ref-86-thronekit.jsx | (header) | (kit exemplar, not wired; the Act 10 cast is drawn on it) |
| 87 | The Assayer | 32×32 | ref-87-assayer.jsx | (header) | `sprites/npc/assayer_idle_0.png` |
| 88 | The Copyist | 32×32 | ref-88-copyist.jsx | (header) | `sprites/npc/copyist_idle_0.png` |
| 89 | The Astronomer | 32×32 | ref-89-astronomer.jsx | (header) | `sprites/npc/astronomer_idle_0.png` |
| 90 | The Choirmaster | 32×32 | ref-90-choirmaster.jsx | (header) | `sprites/npc/choirmaster_idle_0.png` (prop: four brass pipes, not a tuning fork) |
| 91 | The Hermit | 32×32 | ref-91-hermit.jsx | (header) | `sprites/npc/hermit_idle_0.png` |
| 92 | The Registrar | 32×32 | ref-92-registrar.jsx | (header) | `sprites/npc/registrar_idle_0.png` |
| 94 | The Compiler (Act 10 final boss) | 64×64 | ref-94-compiler.jsx | (header) | `sprites/npc/compiler_idle_0.png` (native 64 px) |
| 95 | The Sealed Stacks (Act 9 landmark) | 64×64 | ref-95-sealedstacks.jsx | (header) | `sprites/props/sealed_stacks.png` (prop, bottom-centre anchor, y-sorted, solid tower) |
| 96 | The Throne Dais (Act 10 landmark) | 64×64 | ref-96-thronedais.jsx | (header) | `sprites/props/throne_dais.png` (prop, bottom-centre anchor, y-sorted, solid steps) |
| 97 | Forge yard slab (Act 6 district ground: soot_a, soot_b) | 16×16 ×2 (2-col sheet, 32×16) | ref-97-forgeslab.jsx | (header) | via REF-76: `sprites/tiles_village_v3.png` indices 24-25 (the Forge plaza interior; soot_b on 1 tile in 6) |
| 98 | The Great Furnace (Act 6 landmark) | 64×64 | ref-98-greatfurnace.jsx | (header) | `sprites/props/great_furnace.png` (prop, bottom-centre anchor, y-sorted, solid plinth to hood) |
| 99 | Anvil on a stump (Forge yard prop) | 32×32 | ref-99-anvil.jsx | (header) | `sprites/props/anvil.png` (prop, y-sorted, solid stump) |
| 100 | Quench trough (Forge yard prop) | 32×32 | ref-100-quenchtrough.jsx | (header) | `sprites/props/quench_trough.png` (prop, y-sorted, solid trough) |
| 101 | Coal heap (Forge yard prop) | 32×32 | ref-101-coalheap.jsx | (header) | `sprites/props/coal_heap.png` (prop, y-sorted, solid mound) |
| 102 | Tool rack (Forge yard prop) | 32×32 | ref-102-toolrack.jsx | (header) | `sprites/props/tool_rack.png` (prop, y-sorted, solid only at its two feet) |
| 103 | The Bell Tower (Hearthstone Square landmark) | 64×64 | ref-103-belltower.jsx | (header) | `sprites/props/bell_tower.png` (landmark, y-sorted, solid base to eave) |
| 104 | Village well (Hearthstone prop) | 32×32 | ref-104-well.jsx | (header) | `sprites/props/well.png` (prop, y-sorted, solid ring) |
| 105 | Woodpile (Hearthstone prop) | 32×32 | ref-105-woodpile.jsx | (header) | `sprites/props/woodpile.png` (prop, y-sorted, solid stack) |
| 106 | Signpost (Hearthstone prop) | 32×32 | ref-106-signpost.jsx | (header) | `sprites/props/signpost.png` (prop, y-sorted, solid at the post; `SignpostWest` draws it mirrored) |
| 107 | Market stall (Market Row landmark) | 64×64 | ref-107-marketstall.jsx | (header) | `sprites/props/market_stall.png` (landmark, y-sorted, solid counter foot to awning) |
| 108 | Crate stack (Market Row prop) | 32×32 | ref-108-crates.jsx | (header) | `sprites/props/crates.png` (prop, y-sorted, solid stack) |
| 109 | Barrels (Market Row prop) | 32×32 | ref-109-barrels.jsx | (header) | `sprites/props/barrels.png` (prop, y-sorted, solid group) |
| 110 | Handcart (Market Row prop) | 32×32 | ref-110-handcart.jsx | (header) | `sprites/props/handcart.png` (prop, y-sorted, solid body and wheel) |
| 111 | The Checker's Gate (The Commons landmark) | 64×64 | ref-111-checkersgate.jsx | (header) | `sprites/props/checkers_gate.png` (landmark, y-sorted, solid only at its two pillars; walk through the arch) |
| 112 | Lantern post (The Commons prop) | 32×32 | ref-112-lanternpost.jsx | (header) | `sprites/props/lantern_post.png` (prop, y-sorted, solid at the foot) |
| 113 | Bench (The Commons prop) | 32×32 | ref-113-bench.jsx | (header) | `sprites/props/bench.png` (prop, y-sorted, solid seat) |
| 114 | Fence (The Commons prop) | 32×32 | ref-114-fence.jsx | (header) | `sprites/props/fence.png` (prop, y-sorted, solid rails; pieces 32 px apart join) |
| 115 | Chicken coop (Hearthstone prop) | 32×32 | ref-115-chickencoop.jsx | (header) | `sprites/props/chicken_coop.png` (prop, y-sorted, solid coop, not the hen) |

Props (REF-67, REF-68) are sprites, not tiles: y-sorted, anchored bottom-centre. Terrain (REF-66) uses an opaque base layer plus a transparent grass-edge overlay layer.

A tilemap binds one atlas texture, so every plaza ground reaches the world through REF-76, a verbatim copy of the REF-66 tiles (0-21) followed by the REF-75 flagstone (22-23) and the REF-97 Forge slab (24-25). render-refs' `ref76_atlas_is_verbatim_copy_of_its_sources` test fails if the copy drifts from any source; edit the source ref, never REF-76's rows. Plaza interiors are flagstone inside a one-tile REF-66 cobble ring; roads stay dirt.

**District sets (R1, `design/09-roadmap.md` "Art plan").** A district is dressed by data, not code: its approved ground's tiles are APPENDED to REF-76 (never inserted; existing indices are pinned by `world.rs` tests) and named by the district's `interior` in `world::DISTRICTS` (none = flagstone); its 64×64 landmark goes in `prop::LANDMARKS` (placed on the plaza's back-centre plot); its 32×32 props go in `prop::DISTRICT_PROPS` as (district, kind, offset from the plaza centre); its cast stands at their work through the district's `cast` offsets (one per roster slot; empty = the grid). Each new prop needs a `PropKind` with size, ground line, solid parts and visual box taken from its ref header (the `prop.rs` tests re-derive them from the grid). The tests then check the props stand on the plaza paving, keep street mouths open, leave no walled-off pocket, and that every talk spot is reachable. Capture: `scene-district-<name>.png` (whole plaza, zoomed out) and `scene-district-<name>-yard.png`. The Forge (Act 6) is the first: REF-97 slab, REF-98 Great Furnace, REF-99..102 anvil, quench trough, coal heap, tool rack (approved 2026-11-18). The free demo's three plazas follow on flagstone (R1 wave A2, approved 2026-11-18): Hearthstone Square (REF-103 Bell Tower, REF-104 well, REF-105 woodpile, REF-106 signpost, REF-115 chicken coop), Market Row (REF-107 market stall, REF-108 crates, REF-109 barrels, REF-110 handcart), The Commons (REF-111 Checker's Gate, REF-112 lantern post, REF-113 bench, REF-114 fence). A kind may be placed many times and in other districts; two plaza props may share sprite margins but never an opaque pixel (the tests compare the ref grids), so a fence run can meet a gate pillar.

Walk cycles (`game/src/plugins/animation.rs`, 150 ms per frame, sheets sliced as texture atlases by the ref's column count, index = tile declaration order): player down `down_0`, idle, `down_2`, idle; up `up_0`, `up_1`, `up_2`, `up_1`; left `left_0`, `left_1`, `left_2`, `left_1`; right = left flipped. The second and fourth frames (the pass) are the standing pose lifted 1 px by the engine. Standing shows idle / `up_1` / `left_1` (flipped for right). Ferris: `walk_0`, idle, `walk_2`, idle in every direction, no lift. NPCs get a 1 px idle bob (2.4 s period, phase-offset per roster index). All lifts move the sprite's anchor, never its transform: collision, y-sort, talk range and quest markers are unaffected.

**NPC batches landed:**
- Batch 1 (2026-05-02) — REF-11..15: Smith, Cartographer, Trait Mage, Bellringer, Cooper. Specs in `design/art/specs/`. Replaces SPRITE_PLAYER placeholders for the corresponding entries in `NPC_ROSTER`.
- Batch 2 (2026-05-02) — REF-16..20: Oracle, Herald, Twin, Tinker, Heraldic Sage (the Sage is the first Act-2 NPC shipped; the others finish Act-1 prelude P1). Specs in `design/art/specs/`. Heraldic Sage uses a foreshadowing-license cobalt sigil (~0.9% canvas, under 1% bible cap) to signpost Act 6's cool-counterweight zone — see spec for cross-ref.
- Batch 3 (2026-05-02) — REF-21..25: Forgewright, Linguist, Pilgrim, Drillmaster, Reckoner. First half of the Act-2 cast. Specs in `design/art/specs/`. PNGs verified pixel-identical to the harness output (sample-pixel parity check against the live React canvas in Chrome). No bible-allowance exceptions used; all five sprites stay inside their assigned ramps.
- Batch 4 (2026-06-18) — REF-26..29: Quartermaster, Auditor, Chronicler, Alchemist. Closes the last four `SPRITE_PLAYER` placeholders. Specs in `design/art/specs/`; grids authored from those specs and rendered via `cargo run -p render-refs`. The Auditor uses a documented ~0.6% Alarm-scarlet exception (the flagged `Err` seal), under the 1% cap, cross-ref the Heraldic Sage's cobalt precedent. **First-pass sprites — pending Matt's art-review pass** (the locked 3-revision approval flow); they are a deliberate upgrade from the player-clone placeholders, not a final lock.
- Batch 5 (2026-06-18) — REF-30..35: Guildmaster, Recruiter, Locksmith, Porter, Surveyor, Armorer. The Act 3 Guildhall Quarter cast (missions `impl_method`/`assoc_new`/`if_let`/`while_let`/`tuple_struct`/`enum_data_match`). First-pass; Locksmith/Porter/Armorer flagged rougher. Pending art review.
- Batch 6 (2026-06-18) — REF-36..41: Vexis the Archmage, Wandwright, Conjurer, Familiar, Lanternkeeper, Loremaster. The Act 4 Trait Mage's Tower cast (missions `trait_def`/`generic_fn`/`generic_struct`/`dyn_trait`/`lifetimes`/`assoc_type`). The three mages use a small magic-violet accent (orb/flame) verified under the 5% bible cap (vexis 21px, conjurer 18px, lanternkeeper 9px of 51 max); the Familiar's caged creature carries a teal+forest element-clash accent (fits `Box<dyn Trait>`). First-pass; Lanternkeeper/Loremaster flagged rougher. Pending art review.
- Batch 7 (2026-06-18) — REF-42..47: Barkeep, Bouncer, Interpreter, Mixologist, Tabkeeper, Cellarer. The Act 5 Tavern of Tribulations cast (missions `result_match`/`custom_error`/`from_error`/`option_map`/`and_then`/`unwrap_or_else`). Tavern folk — no magic violet; gold-ramp ale/props. First-pass. Pending art review.
- Batch 8 (2026-06-18) — REF-48..53: Keymaster, Sifter, Smelter, Tallywright, Riveter, Bondsmith. The Act 6 Iterator Forge cast (missions `hashmap_basic`/`iter_filter`/`iter_fold`/`iter_enumerate`/`iter_zip`/`closure_move`). Forge/craft folk — no magic violet; gold/bronze forge props, a molten-metal glow. First-pass. Pending art review.
- Batch 9 (2026-06-18) — REF-54..59: Dockmaster, Lighthouse Keeper, Signaler, Tidewatch, Harbormaster, Tideforecaster. The Act 7 Concurrent Coast cast (missions `thread_spawn`/`arc_mutex`/`mpsc_channel`/`atomic`/`thread_scope`/`async_fn`). Maritime folk — teal/cobalt sea accents encouraged (the one zone where the cool ramp is on-theme), no magic violet. First-pass. Pending art review.
- Batch 10 (2026-06-19) — REF-60..65: Vaultwright, Sharekeeper, Warden, Swapwarden, Strongbox, Ghostkeeper. The Act 8 Vault of Pointers cast (missions `box_basic`/`rc_basic`/`refcell`/`cell`/`rc_refcell`/`weak_ref`). Vault/treasury folk — gold-ramp chests/locks/keys; the Ghostkeeper (weak ref) uses a faded/cobalt translucent treatment to read as non-owning. First-pass. Pending art review.

**Tranches (canvas flow):**
- Tranche 1 (approved by Matt 2026-11-13) — REF-66 terrain v2, REF-67 oak, REF-68 cottage, REF-69..74 costume-kit exemplars for Acts 3-8. REF-66/67/68 are wired into the game. REF-69..74 were confirmed by Matt on 2026-11-15 as the six kit holders' own sprites and wired the same day (REF-69 Guildmaster, REF-70 Conjurer, REF-71 Barkeep, REF-72 Smelter, REF-73 Dockmaster, REF-74 Warden; mapping from each kit's header and the act casts). Their first-pass refs (REF-30, 38, 42, 50, 54, 62) are kept, marked SUPERSEDED, and still render to their old `REFNN.png`.
- REF-75 plaza flagstone (approved by Matt 2026-11-14) — wired 2026-11-14 via the REF-76 combined atlas.
- Tranche 2 (approved by Matt 2026-11-15, wired 2026-11-15) — the 30 Act 3-8 villagers redrawn on the REF-69..74 kits: REF-31..37, 39..41, 43..49, 51..53, 55..61, 63..65, each as `ref-NN-<slug>-v2.jsx` rendered to `REFNNv2.png` and installed over that NPC's own `sprites/npc/<slug>_idle_0.png` (every one was already its own file, no shared placeholder). The first-pass `ref-NN-<slug>.jsx` files are kept, marked SUPERSEDED in their headers, and still render to their old `REFNN.png`. `game/tests/art_wiring.rs` pins every game file to its v2 render. Same approval: REF-77 player walk and REF-78 Ferris walk frames, wired as atlases (see "Walk cycles" above).
- Act 9 cast (approved by Matt 2026-11-15, wired 2026-11-15) — REF-79 Library kit exemplar (rendered to `refs/png`, not any NPC's sprite), REF-80..85 Shelfmark, Envoy, Macro Monk, Sexton, Courier, Bookbinder (32×32) and REF-93 the Librarian (64×64), each rendered to `REFNN.png` and installed as its own new `sprites/npc/<slug>_idle_0.png`. `game/tests/art_wiring.rs` pins each file to its render.
- Act 10 cast (approved by Matt 2026-11-15, wired 2026-11-15) — REF-86 Throne kit exemplar (rendered to `refs/png`, not any NPC's sprite), REF-87..92 Assayer, Copyist, Astronomer, Choirmaster, Hermit, Registrar (32×32) and REF-94 the Compiler (64×64), each rendered to `REFNN.png` and installed as its own new `sprites/npc/<slug>_idle_0.png`; REF-96 the Throne Dais (approved 2026-11-15) rendered and installed as `sprites/props/throne_dais.png`, a 64×64 prop on the Throne plaza at its back-centre, behind the Compiler. `game/tests/art_wiring.rs` pins each file to its render; ref headers say "Wired as ..." (grids untouched). REF-95 Sealed Stacks was then a draft; see the next entry.
- REF-95 the Sealed Stacks (approved by Matt 2026-11-16, wired 2026-11-16) — rendered to `REF95.png` and installed as `sprites/props/sealed_stacks.png`, a 64×64 prop on the Forbidden Library plaza at its back-centre, behind the Librarian (`prop::LANDMARKS`), solid across the whole tower. `game/tests/art_wiring.rs` pins the file to its render; the ref header says "Wired as ..." (grid untouched).

**Remaining placeholders:** no `SPRITE_PLAYER` placeholders remain; all 71 NPCs (Acts 1-10) carry distinct art. In Acts 3-8, 30 villagers carry the approved kit redraws and the six kit holders their act's kit exemplar; Acts 9 and 10 carry their approved casts. Not drawn: the Librarian's and the Compiler's intro/defeat frames, the Librarian's lectern and arena. See `game/src/plugins/npc.rs` for live truth.

**Owner action, due before 2026-12-14:** six of the "rough nine" NPC conversions (REF-29 Alchemist, REF-32 Locksmith, REF-33 Porter, REF-35 Armorer, REF-40 Lanternkeeper, REF-41 Loremaster) depend on an approved artboard in the old claude.ai/design project. Five of them (REF-32/33/35/40/41) are now replaced in game by approved v2 redraws, so the export matters for the game only for REF-29 Alchemist; the others are history. Matt must export that artboard before standalone claude.ai/design closes if he wants it kept.

## 10. Revision history

- **1.0 (2026-04-25)** — initial spec. Companions to bible v2.0 and handoff-prompts v1.0.
- **1.1 (2026-05-02)** — added §9 Rendered REF manifest; logged NPC batch 1 (Smith, Cartographer, Trait Mage, Bellringer, Cooper).
- **1.2 (2026-05-02)** — logged NPC batch 2 (Oracle, Herald, Twin, Tinker, Heraldic Sage). 5 of 9 P2 placeholders remain.
- **1.3 (2026-05-02)** — logged NPC batch 3 (Forgewright, Linguist, Pilgrim, Drillmaster, Reckoner). 4 of 9 Act-2 P2 placeholders remain.
- **1.4 (2026-11-14)** — §5 rewritten for the canvas flow (Fable drafts, Matt approves on the design canvas, render-refs preview to scratch). §9: added REF-66..74 and the tranche 1 log; corrected the placeholder line (Acts 3-8 await the kit redraw); logged the pre-2026-12-14 artboard export as an owner action.
- **1.5 (2026-11-15)** — §9: tranche 2 approved and wired (30 Act 3-8 villager v2 redraws; manifest rows point at the v2 sources), REF-77/78 walk sheets added and wired, walk-cycle assembly documented, placeholder and owner-action lines updated.
- **1.6 (2026-11-15)** — §9: the six kit holders wired to their kit exemplars REF-69..74 (manifest rows for REF-30/38/42/50/54/62 point at the kit sources); Act 9 cast REF-80..85 and REF-93 wired, REF-79 kit exemplar catalogued; placeholder line updated (64 NPCs).
- **1.7 (2026-11-15)** — §9: Act 10 cast REF-87..92 and REF-94 wired, REF-86 kit exemplar catalogued, REF-96 Throne Dais wired as a prop, REF-95 listed as a draft; placeholder line updated (71 NPCs).
- **1.8 (2026-11-16)** — §9: REF-95 Sealed Stacks approved and wired as a prop on the Library plaza.
