# 04 — Art Handoff Prompts

> Copy-paste ready prompts for image generation. Every prompt below assumes the **Style Bible v1.0** rules (`03-art-style-bible.md`) and prepends a **Style Suffix** that locks the constraints. Do not edit the Style Suffix without bumping the bible version.

## How to use this document

1. **Initial reference pass:** generate the 10 reference images listed in `03-art-style-bible.md` § "Reference image set" using the prompts in this doc. These are also tagged below.
2. **Approval gate:** Matt reviews. Reference images either pass (proceed to bulk gen) or fail (style bible may need amending; regenerate).
3. **Bulk gen:** use per-category prompts in `prompts/characters.md`, `prompts/tilesets.md`, `prompts/ui.md`, `prompts/bosses.md`. Each prompt in those files is composed by combining the per-category template with the Style Suffix from this document.
4. **Quality control:** every batch of 20 generations gets reviewed against the reference set. Rejects regenerate. Accepts go into `assets/sprites/`.

---

## The Style Suffix

Append this to **every** generation prompt unless explicitly noted. Treat it as a literal string. Tweak only the bracketed `[fields]` per asset.

```
Pixel art, 16-bit / SNES-era aesthetic, hand-crafted feel. Strict pixel grid,
no anti-aliasing within sprites, integer-multiple scalable. 1-pixel dark outline
on all character and object sprites (palette-derived, never pure black).
Lighting from top-left at 45 degrees, hard-edged shading bands of three or fewer
per surface, no gradients except in skies and magical effects. Color palette
is the EDG32-RUST 32-color palette: Rust oranges (#be4a2f, #d77643), warm reds
(#a22633, #e43b44), forge yellows (#feae34, #fee761), forge greens (#63c74d,
#3e8948), deep teals (#193c3e, #265c42), royal blues (#124e89, #0099db),
cool greys (#8b9bb4, #5a6988), deep navy (#262b44, #181425), magic magentas
(#68386c, #b55088), and parchment cream (#ead4aa, #c0cbdc). No off-palette colors.
No text, no signage with text, no logos. No extra fingers, no melted features,
no over-detailed hair. Cohesive Stardew-Valley-meets-Factorio mood. Resolution
[RESOLUTION]. [SUBJECT-SPECIFIC INSTRUCTIONS].
```

For each prompt below, replace `[RESOLUTION]` and `[SUBJECT-SPECIFIC INSTRUCTIONS]` from the per-prompt template.

---

## Reference Image Set (the 10 lock-the-look images)

### REF-01: Player character (default)

```
[STYLE SUFFIX with:
  RESOLUTION = 32×32 pixels, transparent background.
  SUBJECT = Front-facing human adventurer, idle pose.
  Wearing a simple traveler's tunic in deep blue (#124e89), a tan leather belt,
  brown boots, and a small satchel at the hip. Short hair, friendly face, neutral
  expression. Holding a wooden staff with a small rust-orange crystal at the tip.
  Light skin tone, but designed for easy palette-swap to other skin tones in
  post-production. Single still frame; this is the idle base pose.]
```

### REF-02: Ferris guide

```
[STYLE SUFFIX with:
  RESOLUTION = 32×32 pixels, transparent background.
  SUBJECT = A small, friendly crab character. Round body, two large expressive
  eyes on stalks, two front claws raised in a welcoming gesture. Body color is
  a warm rust orange (#be4a2f to #d77643 shading), with a slightly lighter
  belly. The crab is whimsical and approachable, NOT the official Rust Foundation
  Ferris mascot — distinct silhouette, lighter coloration, slightly more cartoonish
  proportions. Standing on tiny feet, looking up and to the right as if narrating.
  Single still frame, idle pose.]
```

### REF-03: The Borrow Checker NPC

```
[STYLE SUFFIX with:
  RESOLUTION = 64×64 pixels, transparent background.
  SUBJECT = A tall, austere robed figure standing on a stone bridge. Robe is
  deep navy (#262b44) with embroidered glowing cyan (#2ce8f5) reference symbols
  (ampersand "&" and ampersand-mut "&mut" runes) along the hem and sleeves.
  Hood pulled low, face mostly in shadow but two faint cyan-glowing eyes visible.
  Holding a tall iron staff topped with a balance scale. Posture is upright,
  formal, slightly intimidating but not evil — he is doing his job. The robe
  has subtle motion as if a faint magical wind moves it. Single still frame.]
```

### REF-04: Hearthstone Village tile sample sheet

```
[STYLE SUFFIX with:
  RESOLUTION = 128×128 pixels (an 8×8 grid of 16×16 tiles, each tile a separate
  square cell, white grid lines between tiles for clarity).
  SUBJECT = Tile reference sheet showing, in this exact order across the grid:
  (1) plain grass, (2) tall grass with flowers, (3) dirt path straight, (4) dirt
  path corner, (5) cobblestone, (6) wooden fence horizontal, (7) wooden fence
  corner, (8) stone well top-half, (9) stone well bottom-half, (10) wildflower
  patch (yellow), (11) wildflower patch (red), (12) bush, (13) sapling tree,
  (14) hay bale, (15) wooden cart, (16) signpost (no text). All in the warm
  Hearthstone Village palette (browns, grass greens, cream).]
```

### REF-05: Trait Mage's Tower interior tile sample sheet

```
[STYLE SUFFIX with:
  RESOLUTION = 128×128 pixels (8×8 grid of 16×16 tiles, white grid lines for clarity).
  SUBJECT = Mystical tower interior tile reference sheet. Order: (1) purple
  marble floor plain, (2) purple marble with gold inlay rune, (3) cyan-glowing
  magic circle floor section, (4) staircase up, (5) staircase down, (6) purple
  brick wall, (7) wall with gold sconce (lit), (8) wall with bookshelf, (9)
  arched doorway, (10) crystal pedestal, (11) levitating spell book, (12) brazier
  with cyan flame, (13) ornate carpet edge, (14) tall window with starlight,
  (15) wizard table with potions, (16) crystal ball stand. Colors lean into
  deep purples (#68386c), gold (#feae34), and arcane cyan (#2ce8f5).]
```

### REF-06: Code Editor frame

```
[STYLE SUFFIX with:
  RESOLUTION = 640×360 pixels.
  SUBJECT = A pixel-art UI frame for the in-game code editor. Layout:
  - Top bar (24px tall) with three pixel-art buttons labeled "CAST", "TEST",
    "COMMIT" (text rendered as pixel-font shapes only, no font rendering inside
    the image — these are placeholder glyphs).
  - Main area divided 60/40 left/right.
  - Left panel: parchment-cream background (#ead4aa), rusty iron-bound border,
    looks like an old open book where code would go. Show NO code text yet —
    just the empty parchment with subtle leather binding on the left edge.
  - Right panel: divided into two stacked sub-panels with the same iron border
    style. Top sub-panel for "GOAL" (parchment color, slightly darker cream
    #c28569). Bottom sub-panel for "COMPILER OUTPUT" (dark navy #262b44 to
    suggest a slate or terminal). All borders use rivets at the corners.
  - Outer frame: hand-drawn pixel double-line border in iron grey (#5a6988)
    with rust accents at the corners.
  No text, no code, no labels — this is the empty UI shell.]
```

### REF-07: Generic goblin enemy

```
[STYLE SUFFIX with:
  RESOLUTION = 32×32 pixels, transparent background.
  SUBJECT = A small green-skinned goblin, idle pose, looking right. Hunched
  posture, pointed ears, mischievous expression (not evil — opportunistic).
  Wearing scraps of brown leather armor over a tattered tunic. Holding a
  rusty short sword in the right hand. Skin tone uses #3e8948 with #265c42
  shadows. Single still frame, idle.]
```

### REF-08: Healing Potion icon

```
[STYLE SUFFIX with:
  RESOLUTION = 16×16 pixels, transparent background.
  SUBJECT = A small glass potion bottle filled with bright red liquid (#e43b44),
  cork stopper at the top, slight highlight on the upper-left of the glass to
  show roundness. Tag/label area on the bottle is a tiny cream rectangle (no
  text). Designed to read clearly at 16×16 even in a busy inventory grid.]
```

### REF-09: World map cinematic background

```
[STYLE SUFFIX with:
  RESOLUTION = 320×180 pixels.
  SUBJECT = A painted-pixel-art world map showing all 11 zones in a single
  continent-style layout. From south to north: Hearthstone Village (warm
  greens), Borrow Bridge crossing a great gorge, Guildhall City (red brick
  rooftops), Trait Mage's Tower (tall purple spire), the Tavern (small lit
  building at a crossroads), Iron Vale Forge (industrial smoke), Briney Cove
  (coastal, sea blue), The Vault (mountain entrance, gold gleam), Forbidden
  Library (dark cathedral silhouette), Throne of the Compiler (silver
  citadel atop a peak), and the Temple (a stained-glass cathedral floating
  in clouds, slightly off the map at the top). Stylized, cohesive, painted
  pixel-art map style with pin markers for each zone (small flame icons in
  zone-themed colors). No text labels; pin markers only.]
```

### REF-10: Title screen / wordmark

```
[STYLE SUFFIX with:
  RESOLUTION = 640×360 pixels.
  SUBJECT = Title screen for the game. Top two-thirds: the working title
  "CARGO & CROWNS" rendered as a pixel-art logo — stylized blocky letters
  in rust orange (#be4a2f) with a darker outline (#733e39) and small gold
  crown accents above the letters. Below the title, in smaller pixel-letters,
  the tagline "AN RPG WHERE THE COMPILER IS A BOSS" in cream (#ead4aa).
  Bottom third: a hero shot showing the player character and Ferris guide
  standing in front of the Borrow Bridge at sunset, the Borrow Checker
  silhouetted at the far end. Sky is a gradient from rust orange (top) to
  deep navy (bottom). The whole composition reads "epic but cozy."]
```

---

## Per-category prompt templates

Use these to generate batches once references are approved. Each template has slot fields you fill in per asset.

### Character template

```
[STYLE SUFFIX with:
  RESOLUTION = 32×32 pixels, transparent background.
  SUBJECT = [CHARACTER DESCRIPTION].
  POSE = [idle / walking-down frame X / walking-up frame X / attacking frame X / casting frame X / hit / death frame X].
  Mood: [ADJECTIVE]. Class signature color: [PALETTE HEX].
  Single still frame.]
```

### Tile template

```
[STYLE SUFFIX with:
  RESOLUTION = 16×16 pixels (single tile) OR 128×128 pixels (8×8 sheet, white grid lines between tiles).
  SUBJECT = [ZONE NAME] [TILE DESCRIPTION].
  Must tile seamlessly with neighbors of the same type.
  Zone palette dominant: [ZONE PALETTE COLORS].]
```

### UI element template

```
[STYLE SUFFIX with:
  RESOLUTION = [16×16 / 24×24 / 32×32].
  SUBJECT = UI icon for [PURPOSE]. Read clearly at base resolution.
  Outline: 1px palette-dark.
  Fill: [PALETTE COLORS].
  No text.]
```

### Boss template

```
[STYLE SUFFIX with:
  RESOLUTION = [64×64 / 96×96 / 128×128] pixels, transparent background.
  SUBJECT = [BOSS NAME], [DESCRIPTION], [POSE], [PHASE if applicable].
  Distinct silhouette readable at small scale. Color signature: [HEX].
  Animation state: [idle / phase-2-transformation / attack-windup / attack-strike / defeat].
  Single still frame.]
```

---

## Workflow notes for the Claude design page

When sending these prompts via Claude's image-generation tooling:

1. **Always paste the Style Suffix verbatim.** Do not summarize or "you know what I mean" it.
2. **Generate 4 variants per prompt.** Pick the best, regenerate the rest.
3. **Keep transparent backgrounds for character/object sprites.** Backgrounds for tilesheets are a different prompt class.
4. **Save naming convention:** `[category]_[name]_[state]_[frame].png`, e.g. `npc_borrow_checker_idle_0.png`, `tile_hearthstone_grass_01.png`, `ui_potion_health_16.png`.
5. **Keep the reject pile.** Failed generations go in `assets/sprites/_rejects/` for later review of style-bible drift patterns.

---

## When the bible needs amending

If, during initial reference generation, a constraint in the bible turns out to be impossible or produces consistently bad results: stop, document the issue, propose an amendment to the bible (e.g., "outline rule should be 2px on bosses for silhouette readability"), get Matt to sign off, bump the bible version, and only then resume generation. Style consistency wins over any individual asset.
