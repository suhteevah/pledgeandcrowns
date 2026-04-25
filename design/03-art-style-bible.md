# 03 — Art Style Bible

> This document is the contract for every visual asset in the game. It is referenced from every art-generation prompt. Drift is the leading cause of indie game art looking AI-generated. We do not allow drift.

## Style summary

**Pixel-art fantasy with subtle industrial undertones.** Think *Stardew Valley* meets *Factorio* meets the Rust mascot's cheerful pragmatism. Hand-crafted feel, never glossy, never over-detailed, never airbrushed. Outlines clean and dark. Lighting consistent. Animation snappy.

## The non-negotiables

These rules apply to every asset without exception. If a generated image violates any of them, regenerate.

1. **Pixel grid is sacred.** All sprites are pixel-perfect at 1x. No anti-aliasing within sprites. No sub-pixel rendering. Outputs must be integer-multiple-scalable.
2. **Outline rule.** All character/object sprites have a 1-pixel dark outline (color from palette, not pure `#000000`). Backgrounds and tiles do not require outlines.
3. **Lighting source.** Top-left, 45°. Highlights on top-left edges. Shadows on bottom-right. Consistent across every asset.
4. **Limited palette.** 32 colors, locked. See palette section below. No off-palette colors. Ever.
5. **No AI hallucination tells.** No extra fingers. No melted faces. No over-detailed hair. No nonsensical text on signs. Generations that exhibit these get rejected; we crop, retouch, or regenerate.
6. **No photorealistic blends.** This is not a photo-bash. Colors are flat or have at most three shading bands per surface. Gradients exist only in skies and magical effects.

## Sprite resolutions

| Asset type                   | Resolution      | Notes                              |
|------------------------------|-----------------|------------------------------------|
| Tile (terrain, floor)        | 16×16           | Tilemap base unit                  |
| Object (chest, sign, prop)   | 16×16 to 32×32  | Single-tile or 2×2 props           |
| NPC / player character       | 32×32           | 4-frame walk cycles in 4 directions |
| Mini-boss                    | 64×64           | 6-frame attack animations          |
| Boss                         | 96×96 to 128×128 | Full animation rigs                 |
| UI icon (inventory, status)  | 16×16 or 24×24  | Two scales depending on context    |
| Cinematic / cutscene art     | 320×180 (1×) → upscaled | Backgrounds for talky moments      |
| Title screen / promo art     | 640×360 (1×)    | The exception; bigger but still pixel |

## Palette

Lock to a 32-color custom palette. We use a Rust-themed adaptation of the **Endesga 32 (EDG32)** palette as the base, with two custom Rust-orange shifts:

```
EDG32-RUST palette (hex):
#be4a2f  #d77643  #ead4aa  #e4a672  #b86f50  #733e39  #3e2731  #a22633
#e43b44  #f77622  #feae34  #fee761  #63c74d  #3e8948  #265c42  #193c3e
#124e89  #0099db  #2ce8f5  #ffffff  #c0cbdc  #8b9bb4  #5a6988  #3a4466
#262b44  #181425  #ff0044  #68386c  #b55088  #f6757a  #e8b796  #c28569
```

The first row leans into Rust's signature oranges and warm rusts. Cool tones come from the middle rows. The bottom row gives us flesh tones and accent magentas for magic effects.

**Reasoning for sticking with a known palette:** EDG32 is well-known in the indie pixel-art community, has documented best-practice usage examples, and our minor tweaks keep us recognisable to that audience.

**License note.** EDG32 is published by Endesga on Lospec under the Creative Commons / public-pixel-palette norms; commercial use is permitted but **a credit line in the game's About screen and end-of-demo screen is required** (e.g. "Palette adapted from EDG32 by Endesga"). Confirm the exact license terms on Lospec before launch and pin the citation text in the credits doc.

## Visual themes per zone

Each act has its own dominant palette range pulled from the master palette. This gives zones identity without breaking style.

| Zone                          | Dominant range                          | Mood                                    |
|-------------------------------|-----------------------------------------|-----------------------------------------|
| Hearthstone Village (Act 1)   | Warm browns + grass greens + cream     | Cozy, safe, golden hour                |
| Borrow Bridge (Act 2)         | Stone grey + ice blue + faint orange   | Solemn, austere, important             |
| Guildhall Quarter (Act 3)     | Brick reds + tavern yellows + black     | Bustling, urban, lived-in              |
| Trait Mage's Tower (Act 4)    | Deep purples + magic cyan + gold        | Mystical, vertical, awe                |
| Tavern (Act 5)                | Lamplight oranges + smoky greys         | Warm-but-treacherous, low light        |
| Iron Vale Forge (Act 6)       | Industrial grey + forge orange + soot   | Mechanical, smoky, honest              |
| Briney Cove (Act 7)           | Sea blues + sand + lighthouse white     | Open, wind-swept, slightly chaotic     |
| The Vault (Act 8)             | Gold + deep blue + arcane green         | Ancient, secretive, treasure-rich      |
| Forbidden Library (Act 9)     | Dark purple + corrupted red + parchment | Dangerous, dim, with red glitches       |
| Throne of the Compiler (Act 10)| Silver + chrome + sunlight white         | Climactic, vast, celestial             |
| The Temple (Act 11)           | Stained-glass primary RGB + gold         | Reverent-meets-glitchy, hand-of-god     |

## Animation conventions

- **Idle:** 2-frame breathing loop, 600ms per frame.
- **Walk:** 4-frame cycle, 150ms per frame, 4-directional (down, up, left, right). Right reuses left flipped.
- **Attack:** 4–6 frames, 80–120ms each, with a wind-up frame and a recovery frame.
- **Cast:** 6-frame loop, 100ms each, with magic effect drawn on a separate layer.
- **Hit:** 1-frame flash white, 60ms, then return to idle.
- **Death:** 4-frame collapse + fade.

Bosses get more animation states (intro, phase-2 transformation, defeat). Document those per-boss in `prompts/bosses.md`.

## UI style

- **Font:** Pixel-perfect. Use [m6x11plus](https://managore.itch.io/m6x11) or equivalent open-source pixel font. **No web fonts**, **no system fonts**.
- **Window frames:** Hand-drawn pixel borders, 2-pixel double-line with corner ornaments. Corners are rusty/iron-bound, never plastic-looking.
- **Code editor frame:** Inset bevel, parchment-cream background (`#ead4aa`), syntax highlighting using palette colors only:
  - Keywords: `#be4a2f` (rust orange)
  - Types: `#0099db` (cool blue)
  - Strings: `#63c74d` (forge green)
  - Comments: `#8b9bb4` (cool grey)
  - Numbers: `#feae34` (warm yellow)
  - Errors: `#e43b44` (alarm red)
- **Buttons:** Two states — idle (slight bevel up) and pressed (bevel down + 1px offset).
- **Icons:** 16×16 or 24×24 with the same outline rule as characters.

## What to avoid

- Drop shadows under text. (Use a 1-pixel outline instead.)
- Generated text in images (signs, books, banners). All text in the world goes through the engine's text renderer.
- Blurry or "soft" shading. We use hard-edged bands.
- Trendy 8-bit nostalgia color schemes (NES limitations are not our limitations).
- Anything that looks remotely like Pokémon-clone art. We are not that.
- Bevy logos, Rust logos, Ferris-the-real-mascot. Our Ferris is *inspired by* but visually distinct from the official one to avoid trademark issues. Lighter shell, different proportions, signed off by Matt before any public posting.

## Reference image set (must be generated and approved before bulk asset gen)

Generate exactly these 10 reference images first. They serve as the "lock the look" milestone. **No other assets are produced until these are approved by Matt.**

1. **Player character (default)** — front-facing, idle, 32×32. Standing on a transparent background.
2. **Ferris guide** — small crab, 32×32 (slightly smaller within the canvas), four-direction walk-cycle base frame.
3. **Borrow Checker NPC** — tall robed figure, glowing reference symbols on his robe, 64×64.
4. **Hearthstone Village tile sample** — 8×8 tile grid showing grass, dirt path, cobblestone, well, fence, flowers.
5. **Trait Mage's Tower interior tile sample** — purple-and-gold marble, magic runes inset, 8×8 grid.
6. **Code Editor frame** — full UI frame, no code yet, showing the parchment background and the rusty borders.
7. **Generic goblin enemy** — 32×32, idle pose with weapon, mid-range threat read.
8. **Healing Potion icon** — 16×16, in the inventory style.
9. **World map cinematic background** — 320×180, painted in our pixel-art style, showing all 11 zones at a high level.
10. **Title screen / wordmark** — 640×360, the game's logo over a hero shot of the world. Working title placeholder ("CARGO & CROWNS") in pixel font.

Once these 10 are approved, the rest of asset gen can proceed in batches with confidence.

## Versioning the bible

This document is versioned. Any change to the palette, the resolution table, or the animation conventions requires an update bumping the version number at the bottom and a note in `HANDOFF.md`. **Bible v1.0** is the version corresponding to MVP scope.

---

**Bible version:** 1.0
**Last updated:** 2026-04-25
