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
- 9-slice frames use the `_9slice.png` suffix and ship with a sibling `.json` describing border insets.
- Boss assets that need >20 files get their own subdirectory under `bosses/`.

## 5. Approval workflow

1. **Reference pass first.** The 10 reference images in `04-art-handoff-prompts.md` §"Reference Image Set" must ship before any §1 asset begins. Matt is the sole approver. Two-round limit: round 1 = first delivery, round 2 = revisions per Matt's notes. If round 2 fails, the bible is amended and the contractor restarts from a new reference brief.
2. **Bulk gen in batches of 20.** Each batch is reviewed against the approved reference set. Reject rate >25% triggers a style-bible review meeting before the next batch.
3. **Per-asset feedback** is given inline in a shared sheet (one row per asset, columns: ID, status, notes, revision count). Three revisions max per asset; further iteration kicks the asset back to the bible-amend path.
4. **Done = merged.** Asset is "done" when the file lands in `assets/sprites/` on `main` and the palette-conformance + naming-convention checks both pass on the pre-commit hook.

## 6. IP and licensing

- **Default:** all art is **work-for-hire**. Contractor assigns full copyright to Ridge Cell Repair LLC at delivery. Standard agreement language; no carve-outs for portfolio use without prior written approval.
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
| 04 | Hearthstone Village tiles | 16×16 ×16 | ref-04-tiles-village.jsx | (bible §Reference set) | `sprites/tiles_village.png` |
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

**NPC batches landed:**
- Batch 1 (2026-05-02) — REF-11..15: Smith, Cartographer, Trait Mage, Bellringer, Cooper. Specs in `design/art/specs/`. Replaces SPRITE_PLAYER placeholders for the corresponding entries in `NPC_ROSTER`.
- Batch 2 (2026-05-02) — REF-16..20: Oracle, Herald, Twin, Tinker, Heraldic Sage (the Sage is the first Act-2 NPC shipped; the others finish Act-1 prelude P1). Specs in `design/art/specs/`. Heraldic Sage uses a foreshadowing-license cobalt sigil (~0.9% canvas, under 1% bible cap) to signpost Act 6's cool-counterweight zone — see spec for cross-ref.
- Batch 3 (2026-05-02) — REF-21..25: Forgewright, Linguist, Pilgrim, Drillmaster, Reckoner. First half of the Act-2 cast. Specs in `design/art/specs/`. PNGs verified pixel-identical to the harness output (sample-pixel parity check against the live React canvas in Chrome). No bible-allowance exceptions used; all five sprites stay inside their assigned ramps.

**Remaining placeholders** (still using `SPRITE_PLAYER`): Quartermaster, Auditor, Chronicler, Alchemist (Act-2 P2). See `game/src/plugins/npc.rs` for live truth.

## 10. Revision history

- **1.0 (2026-04-25)** — initial spec. Companions to bible v2.0 and handoff-prompts v1.0.
- **1.1 (2026-05-02)** — added §9 Rendered REF manifest; logged NPC batch 1 (Smith, Cartographer, Trait Mage, Bellringer, Cooper).
- **1.2 (2026-05-02)** — logged NPC batch 2 (Oracle, Herald, Twin, Tinker, Heraldic Sage). 5 of 9 P2 placeholders remain.
- **1.3 (2026-05-02)** — logged NPC batch 3 (Forgewright, Linguist, Pilgrim, Drillmaster, Reckoner). 4 of 9 Act-2 P2 placeholders remain.
