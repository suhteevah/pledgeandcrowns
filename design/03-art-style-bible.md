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

## Palette — "Heraldic Code" (custom, 32 colors, v2.0)

We do **not** use a stock pixel-art palette (EDG32, Apollo, Pico-8). Stock palettes give us indistinguishable-from-every-other-pixel-game art, and EDG32 specifically would push us into the same warm-orange register as the Rust mascot, the Anthropic brand, and roughly half of every pixel-art game on itch.io. We need a unique signature.

Instead the palette is *derived from a named harmonic structure* on the HSL color wheel. This makes per-zone palettes, syntax-highlighting choices, and asset-gen prompts all defensible from one source rather than picked by feel.

### Harmonic structure

```
                ANCHOR
              350° burgundy  (the Crown / pledge / heraldry)
                   |
        +52° |     |     | -180° complement direction
   gold      |     |     |
   42°       |     |     |
  (analog.   |     |     |---- split-complementary pair:
   accent)   |     |          178°  teal-cyan   (code, the Borrow Checker)
             |     |          140°  forest-green (foliage, world)
             |     |
             |     +---- +280° tetradic flicker:
             |              270°  arcane violet (magic only, sparing)
             |
             +-- cool counterweight at 215°: cobalt blue (sea, sky)
```

- **Split-complementary** at 350° (anchor) ↔ 140° + 178° (cool pair). This gives us a stable warm/cool tension without the "two opposites screaming at each other" problem of a pure complementary scheme.
- **Analogous warm accent** at 42° (gold). Sits close enough to the anchor to belong to the same warm family but distinct enough to function as a second brand color. Crown + gold = traditional heraldic pairing, on purpose.
- **Tetradic flicker** at 270° (violet). Completes a loose tetrad with the anchor and the split-comp pair. Used **only** for magic effects and the Trait Mage's Tower — under 5% of any non-magic frame.
- **Cool counterweight** at 215° (cobalt). Not part of the harmony proper; included so sea/sky zones have a non-teal blue option without breaking the structure.
- **Achromatic axis** is warm-biased (parchment-cream → coal-black), which keeps unsaturated areas in conversation with the gold rather than fighting it.

The dominant brand color is **oxblood burgundy** (#6B1F35) — the "Pledge & Crown" wordmark, key UI accents, and the player's heraldic motif all key off it. Rust the *language* is referenced through aged-iron grays in the world, **never** through orange: that's a deliberate dodge of the obvious-to-the-point-of-cliché choice.

### The 32 colors

Each color has a role name; reference colors by name in prompts and code, never by raw hex.

**Burgundy ramp — anchor (hue ~352°):** primary brand, blood, heraldry, Pledge UI accents.
```
#1B0810  Inkblood        — outline / deepest shadow on red surfaces
#3E1220  Crypt           — deep heraldic shadow
#6B1F35  Oxblood         — PRIMARY BRAND — wordmark, banners
#982D52  Wineflesh       — main mid-tone for cloth, banners
#C56883  Dusty rose      — soft fabric highlight
#EBC2CC  Pink quartz     — pale rim light, blush
```

**Gold ramp — analogous accent (hue ~42°):** the literal Crown, treasure, parchment, achievement.
```
#2D1F0A  Bog umber       — gold deepest shadow
#5E4116  Bronze          — antique metal mid-shadow
#9C7026  Antique brass   — gold mid-tone
#D2A53F  Old gold        — THE CROWN — coins, rim light on metal
#F0D27D  Brass leaf      — bright highlight on metal
#FCEFC8  Parchment cream — primary UI surface, paper
```

**Teal ramp — split-complementary (hue ~178°):** the Borrow Checker's domain, code, magic-of-rules.
```
#062123  Abyssal         — code-editor cursor, deep magic shadow
#154548  Deep teal       — borrow-checker's robe shadow
#2A8482  Main teal       — borrow-checker's signature, type sigils
#5BB8AF  Bright teal     — magic emission, ice highlights
#A4DED4  Mist teal       — magic glow rim, fog
```

**Forest/spring green ramp — split-complementary (hue ~140°):** foliage, world, Hearthstone, life.
```
#142A19  Mossbed         — foliage shadow
#27502E  Pine            — tree trunks, dark grass
#487E40  Forest          — primary grass
#82B450  Spring meadow   — fresh growth, lit grass
#C9DC97  Hayfield        — pale grass, wheat, sun-bleached straw
```

**Warm-biased neutrals:** outlines, stone, paper.
```
#161313  Coalblack       — universal outline color (NEVER #000000)
#3E3833  Basalt          — stone shadow, code-editor frame
#7A7064  Stone grey      — stone mid-tone, distant terrain
#BFB2A0  Aged paper      — old documents, faded parchment
```

**Tetradic violet (hue ~270°):** magic only. The Forbidden Library, Trait Mage's Tower, spell effects.
```
#3A1559  Royal arcane    — magic shadow, corruption
#9D6FE0  Mage glow       — spell emission, runes
```

**Cool counterweight + alarms + specular:**
```
#0E2E54  Cobalt deep     — night sky, abyssal sea
#377AB8  Cobalt          — daytime water, ice
#E63946  Alarm scarlet   — error highlights, panic UI (distinct from anchor)
#FFFFFF  Specular white  — pure highlights only, ≤1% of any frame
```

### Why this avoids the "AI-generated pixel art" smell

- **Named harmony, not vibes.** Every color has a role and a position on the wheel. When a generation comes back and we ask "is this on-palette?" the question has an answer.
- **Underused hue territories.** Most pixel-art games default to either NES-throwback or warm-orange-and-teal. We sit in burgundy + gold + cool-pair, a heraldic-manuscript register that's rare in the indie pixel space.
- **Scarcity discipline.** Specular white and arcane violet are gated to <5% of any frame. Most palettes get muddy because every color competes; ours has a strict hierarchy (anchor > split-comp pair > accent > flicker).

## Visual themes per zone

Each act draws a dominant subset from the 32-color palette by **role name**, never by raw hex. This both keeps zones distinct and forces every prompt/asset to stay inside the harmonic structure.

| Zone                          | Dominant roles                                                              | Mood                                    |
|-------------------------------|-----------------------------------------------------------------------------|-----------------------------------------|
| Hearthstone Village (Act 1)   | Forest, Spring meadow, Hayfield, Parchment cream + Old gold accents          | Cozy, golden hour, safe                |
| Borrow Bridge (Act 2)         | Stone grey, Basalt, Deep teal, Main teal, Brass leaf accents (banners)       | Solemn — teal is the Borrow Checker's domain |
| Guildhall Quarter (Act 3)     | Oxblood, Wineflesh, Aged paper, Coalblack, Old gold trim                     | Bustling urban heraldry                |
| Trait Mage's Tower (Act 4)    | Royal arcane, Mage glow, Old gold, Cobalt deep, Specular white runes         | Mystical, vertical, awe                |
| Tavern (Act 5)                | Bronze, Antique brass, Basalt, Wineflesh, Crypt shadows                      | Lamplight, smoky, warm-but-treacherous |
| Iron Vale Forge (Act 6)       | Basalt, Coalblack, Bronze, Stone grey + Alarm scarlet (forge fire only)      | Mechanical, smoky, honest              |
| Briney Cove (Act 7)           | Cobalt deep, Cobalt, Mist teal, Hayfield (sand), Parchment cream             | Open, wind-swept                       |
| The Vault (Act 8)             | Old gold, Brass leaf, Bog umber, Cobalt deep, Mist teal                       | Ancient, treasure-rich                 |
| Forbidden Library (Act 9)     | Royal arcane, Crypt, Aged paper, Alarm scarlet (corruption glitches only)    | Dangerous, dim, glitchy                |
| Throne of the Compiler (Act 10)| Stone grey, Aged paper, Specular white, Brass leaf, Cobalt deep              | Climactic, vast, celestial             |
| The Temple (Act 11)           | Oxblood + Main teal + Forest (the split-comp triangle) + Old gold + Mage glow | Reverent-meets-glitchy stained glass   |

The Temple's stained-glass effect is intentionally the literal split-complementary triad of the palette structure — when the player reaches the climactic religious zone, they're seeing the *harmonic skeleton* of the whole game's color system rendered as light through glass. This is also a curriculum payoff: by Act 11 the player has used every color family the game knows.

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
- **Code editor frame:** Inset bevel, **Parchment cream** (`#FCEFC8`) background, **Basalt** (`#3E3833`) frame, syntax highlighting drawn from palette role names only. The keyword/type pairing is deliberately the anchor + split-complementary teal — the two highest-frequency token classes form the same harmonic conversation as the rest of the game:
  - Keywords (`fn`, `let`, `mut`): **Wineflesh** `#982D52`
  - Types (`String`, `&mut T`): **Main teal** `#2A8482` — the Borrow Checker's signature color, on purpose
  - Strings: **Forest** `#487E40`
  - Comments: **Stone grey** `#7A7064`
  - Numbers / literals: **Old gold** `#D2A53F`
  - Errors / underline squiggles: **Alarm scarlet** `#E63946`
  - Cursor: **Abyssal** `#062123`
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
10. **Title screen / wordmark** — 640×360, the game's logo over a hero shot of the world. Working title "PLEDGE & CROWN" in pixel font, **Oxblood** wordmark with **Old gold** rim, on a **Parchment cream** banner over a **Cobalt deep** sky.

Once these 10 are approved, the rest of asset gen can proceed in batches with confidence.

## Versioning the bible

This document is versioned. Any change to the palette, the resolution table, or the animation conventions requires an update bumping the version number at the bottom and a note in `HANDOFF.md`. **Bible v1.0** is the version corresponding to MVP scope.

---

**Bible version:** 2.0 — palette swapped from EDG32-RUST to "Heraldic Code" (split-complementary @ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker). Per-zone roles, editor syntax colors, and reference-image titles updated to match.
**Last updated:** 2026-04-25
