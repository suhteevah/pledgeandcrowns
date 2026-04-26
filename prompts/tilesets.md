# Tilesets — Art Prompts

> Composes with the **Style Suffix** from `design/04-art-handoff-prompts.md`. Always paste the Style Suffix verbatim before each prompt below.
>
> Tile sheets are typically generated as 8×8 grids of 16×16 tiles (so 128×128 total) with white grid lines for cutting. Once approved, the grid lines are removed in post-processing and tiles are sliced into individual `.png` files.

## Production rules for tiles

1. **Seamless edges.** Adjacent tiles of the same type must tile without visible seams. Generate with this in mind; verify by tiling a test grid in-engine before approving.
2. **Multiple variants.** Generate 3–4 variants of each common ground tile to break visual repetition. The tilemap engine picks variants randomly.
3. **Animated tiles** (water, fire, glowing runes) get a 4-frame loop, each frame on its own row of the tile sheet for easy slicing.
4. **Naming:** `tile_<zone>_<n>_<variant>.png`. Example: `tile_hearthstone_grass_a.png`, `tile_hearthstone_grass_b.png`.

---

## Hearthstone Village (Act 1)

### Ground tiles

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid of 16×16 tiles, white grid
lines between for slicing). SUBJECT = Hearthstone Village ground tile sheet.
Order across grid:
(1) plain grass A (2) plain grass B (3) plain grass C (4) tall grass with
flowers (5) dirt path straight horizontal (6) dirt path straight vertical
(7) dirt path corner NE (8) dirt path corner NW (9) dirt path corner SE
(10) dirt path corner SW (11) dirt path T-intersection (12) cobblestone A
(13) cobblestone B (14) wood planks horizontal (15) wood planks vertical
(16) wood plank corner. Bog umber / Bronze browns + Forest / Spring meadow grass green palette dominant.
```

### Props and structures

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Hearthstone Village
props sheet. Order: (1) wooden fence horizontal (2) wooden fence vertical
(3) wooden fence corner (4) wooden fence end-post (5) stone well top
(6) stone well bottom (7) stone well water (animated frame) (8) wildflower
patch yellow (9) wildflower patch red (10) wildflower patch blue (11) bush
small (12) bush large (13) sapling tree (14) full pine tree top (15) full
pine tree bottom (16) hay bale stack.
```

### Buildings exterior

```
[STYLE SUFFIX] RESOLUTION = 128×64 (4×2 grid of 32×32). SUBJECT = Hearthstone
cottage exterior. Order: (1) wall plain (2) wall with window (3) wall with
door (4) wall corner (5) thatch roof straight (6) thatch roof corner
(7) thatch roof peak (8) chimney with smoke (animated frame).
```

## Borrow Bridge (Act 2)

### Bridge surface

```
[STYLE SUFFIX] RESOLUTION = 128×64 (4×2 grid of 32×32 tiles). SUBJECT = Stone
bridge tile set. Order: (1) bridge plank straight (2) bridge plank with
moss (3) bridge railing left (4) bridge railing right (5) bridge support
column (6) bridge edge cracking (7) keystone center (8) bridge end ramp.
Stone grey (#7A7064) dominant with Cobalt (#377AB8) accents in cracks.
Slight glowing Main teal rune carvings here and there.
```

### Gorge backdrop

```
[STYLE SUFFIX] RESOLUTION = 320×180. SUBJECT = Riftwater Gorge backdrop,
parallax-ready painted-pixel-art landscape. Foreground: bridge edge.
Mid-ground: chasm walls dropping into mist. Far ground: a roaring Cobalt
river far below. Sky: Mist teal to Parchment cream gradient with a few faint
rune-like sigils floating in the air around the bridge level. Mood: solemn,
important, slightly otherworldly.
```

### Decorative runes

```
[STYLE SUFFIX] RESOLUTION = 64×64 (2×2 grid of 32×32). SUBJECT = Floating
glyph tiles for Borrow Bridge. (1) ampersand "&" rune glowing Main teal
(#2A8482) (2) "&mut" rune glowing slightly more intense Bright teal (#5BB8AF)
(3) lifetime tick "'a" rune glowing Old gold (#D2A53F) (4) generic decorative
rune. Each on transparent background, designed to float above the bridge as
ambient details.
```

## Guildhall Quarter (Act 3) — STUB, generate when Act 3 in scope

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Guildhall Quarter
city tile sheet. Cobblestone, brick walls, banner colors per guild (Oxblood
warriors, Cobalt mages, Forest rangers, Old gold artificers). Defer detail
until Act 3 work begins.
```

## Trait Mage's Tower (Act 4) — STUB

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Tower interior tile
sheet. Royal arcane marble floors, Old gold-inlaid runes, magic circles, arcane
bookshelves, crystal pedestals. Defer until Act 4 work begins.
```

## Iron Vale Forge (Act 6) — STUB

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Industrial forge
tile sheet. Basalt / Stone grey floors, Bronze conveyor belts (animated 4-frame),
Alarm scarlet forge fires (animated), assembly tables, ore piles. Defer until
Act 6 work begins.
```

## Briney Cove (Act 7) — STUB

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Coastal town tile
sheet. Wooden piers, rope coils, fishing nets, sand, Cobalt sea (animated 4-frame),
lighthouse stone. Defer until Act 7 work begins.
```

## The Vault (Act 8) — STUB

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Vault interior tile
sheet. Old gold floor tiles, Cobalt deep marble walls, glowing artifact pedestals,
arcane lockboxes, treasure piles. Defer until Act 8 work begins.
```

## Forbidden Library (Act 9) — STUB

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Library tile sheet.
Royal arcane walls, Parchment cream floors, towering bookshelves, candles,
glitch-corrupted floor sections (Alarm scarlet glitch artifacts only — palette
only, no real RGB noise). Defer until Act 9 work begins.
```

## Throne of the Compiler (Act 10) — STUB

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Silver-and-chrome
celestial throne room tile sheet. Bright sunlit white floors, silver pillars,
chrome accents, throne dais, ornate balustrades. Defer until Act 10 work begins.
```

## The Temple (Act 11) — STUB

```
[STYLE SUFFIX] RESOLUTION = 128×128 (8×8 grid). SUBJECT = Stained-glass cathedral
tile sheet. Stone floors, stained-glass window edges (Oxblood / Cobalt / Old gold
panes), pew rows, altar steps, candelabras. Defer until Temple build begins (post-MVP).
```

---

## Animated tiles spec

For any tile with motion (water, flame, smoke, glowing rune), produce a **4-frame animation strip**:

```
[STYLE SUFFIX] RESOLUTION = 64×16 (4 frames of 16×16, side-by-side).
SUBJECT = <tile name> animation, 4-frame loop. Frame timings: 250ms each.
Frames should loop seamlessly (frame 4 → frame 1 with no jarring transition).
```

Examples:
- `tile_hearthstone_well_water_anim.png` (4× 16×16 = 64×16)
- `tile_borrow_glyph_amp_anim.png`
- `tile_forge_conveyor_anim.png` (Act 6)
- `tile_briney_sea_anim.png` (Act 7)
- `tile_temple_stained_glow_anim.png` (Act 11)

## Production checklist per zone

When finalizing a zone's tileset:
- [ ] Ground tile sheet (3+ ground variants)
- [ ] Props sheet (zone-thematic objects)
- [ ] Buildings/structures sheet
- [ ] Animated tiles strips (water, fire, magic)
- [ ] Backdrop / parallax layers if outdoor zone
- [ ] In-engine tile-test verifies seamless edges
- [ ] Filed under `assets/tilemaps/<zone>/`
- [ ] Logged in `assets/tilemaps/MANIFEST.md`
