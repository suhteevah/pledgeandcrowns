# UI — Art Prompts

> Composes with the **Style Suffix** from `design/04-art-handoff-prompts.md`. Always paste the Style Suffix verbatim before each prompt below.
>
> Naming convention: `ui_<area>_<element>_<state>.png`
> Examples: `ui_hud_health_bar_full.png`, `ui_editor_frame_idle.png`, `ui_dialog_box_open.png`

## Production rules for UI

1. **Pixel-perfect at base resolution.** UI elements are designed at 1× and integer-scaled at runtime. No half-pixel offsets.
2. **Outline rule applies.** UI containers and icons use 1px palette-derived dark outlines, never `#000000`.
3. **Two states minimum** for interactive elements: idle and pressed/hover. Disabled state if applicable.
4. **No text in generated images.** All text is rendered by the engine using the locked pixel font (`m6x11plus`). UI mock-ups may show placeholder glyph shapes; finals do not.
5. **9-slice friendly.** Frames and panels designed as 9-slice resizable elements where appropriate (corners + edges + center repeatable).

---

## Code Editor (the central UI)

### Editor frame — empty shell

```
[STYLE SUFFIX] RESOLUTION = 640×360. SUBJECT = Pixel-art UI frame for the
in-game code editor. Layout:
- Top bar (24px tall) with three pixel-art button shapes: "CAST" (Oxblood
  #6B1F35 fill, Coalblack outline), "TEST" (Old gold #D2A53F fill), "COMMIT"
  (Forest #487E40 fill). No text inside buttons in this asset — placeholder
  blocks only.
- Main area divided 60/40 left/right.
- Left panel: Parchment cream background (#FCEFC8), iron-bound border (Basalt)
  with visible Oxblood rivets at corners, looks like an open leather-bound book
  where code would go. No code text yet.
- Right panel: split into two stacked sub-panels.
  * Top sub-panel "GOAL": Aged paper (#BFB2A0 — slightly darker cream), iron border.
  * Bottom sub-panel "COMPILER OUTPUT": Cobalt deep slate (#0E2E54), iron border.
- Outer frame: hand-drawn pixel double-line border in Stone grey (#7A7064) with
  Oxblood accents at corners.
- Lower-right: small ESC indicator block.
- All borders: pixel-perfect, no anti-aliasing.
```

### Editor frame — error state (compile failed)

```
[STYLE SUFFIX] RESOLUTION = 640×360. SUBJECT = Same code editor frame as
empty shell, but with the COMPILER OUTPUT sub-panel glowing Alarm scarlet
(#E63946) along its border, suggesting a compile error. Left parchment panel
has a single horizontal line highlighted in pale Alarm scarlet wash to indicate
the error line. No actual text — just the glow and highlight.
```

### Editor frame — success state (cast accepted)

```
[STYLE SUFFIX] RESOLUTION = 640×360. SUBJECT = Same code editor frame, but
with all three top buttons glowing softly with Bright teal (#5BB8AF) magic accent,
and the left parchment panel's border glowing Forest (#487E40). Suggests
victory / acceptance. No text.
```

### Code editor — line-number gutter strip

```
[STYLE SUFFIX] RESOLUTION = 24×320. SUBJECT = Vertical gutter strip for line
numbers, Aged paper background (#BFB2A0 — slightly darker than main editor),
iron divider on the right edge, faint horizontal pixel ticks every 16 pixels
to suggest line baselines. No numbers rendered in image — engine renders those.
```

---

## Heads-Up Display (HUD)

### Health bar — full / mid / low / empty

```
[STYLE SUFFIX] RESOLUTION = 96×16. SUBJECT = Pixel-art health bar, four
versions side-by-side or as separate assets: (1) full = Forest (#487E40) fill,
(2) mid = Old gold (#D2A53F) fill at 60% width, (3) low = Alarm scarlet
(#E63946) fill at 25% width, (4) empty = Cobalt deep (#0E2E54) fill. All four
use the same iron-bound frame: 1px dark outline, 1px inner
highlight on top edge to suggest depth. End caps are slightly rounded
(within pixel grid limits).
```

### Mana bar — full / mid / low / empty

```
[STYLE SUFFIX] RESOLUTION = 96×16. SUBJECT = Same frame style as health bar
but with mana-blue fills: (1) full = Cobalt (#377AB8), (2) mid = Mist teal
(#A4DED4) at 60%, (3) low = Aged paper (#BFB2A0) at 25%, (4) empty = Coalblack.
```

### Sanity meter — full / corrupted / depleted (Act 9+ only)

```
[STYLE SUFFIX] RESOLUTION = 96×16. SUBJECT = Sanity meter with eldritch
character. (1) full = Brass leaf (#F0D27D) fill with subtle pulsing pattern,
(2) corrupted = Mage glow (#9D6FE0) fill with jagged top edge suggesting
instability, (3) depleted = Coalblack (#161313) with a faint Alarm scarlet pulse.
Frame style consistent with health/mana bars.
```

### Turn indicator (combat)

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = Animated turn
indicator arrow, four-frame loop. Arrow points downward, glowing Old gold
(#D2A53F). Frames show subtle bouncing motion. Used to indicate which unit
is acting in combat.
```

### Damage / heal floating-number badge

```
[STYLE SUFFIX] RESOLUTION = 24×24 transparent. SUBJECT = Two badge
backgrounds: (1) damage badge = jagged Alarm scarlet star-burst shape
(#E63946 fill, Crypt outline), (2) heal badge = soft Forest plus-shape
(#487E40 fill). Numbers rendered by engine; this is just the badge background.
```

---

## Inventory and item icons (16×16)

All inventory icons are 16×16 transparent. Designed to read clearly at native
size in a busy grid.

### Consumables

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Healing Potion icon.
Glass bottle, Alarm scarlet liquid (#E63946), cork stopper, single highlight
on upper-left of glass. Tiny Parchment cream label rectangle (no text).
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Mana Potion icon.
Glass bottle, Cobalt mana-blue liquid (#377AB8), cork stopper, single highlight.
Tiny Parchment cream label rectangle.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Antidote icon. Glass
bottle, swirling green-and-violet liquid (Forest #487E40 / Mage glow #9D6FE0),
cork stopper.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Bread loaf icon
(food). Round golden-brown loaf with cross-hatch top, slight steam rising.
```

### Weapons

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Wooden staff icon.
Vertical orientation, wooden shaft, small Old gold (#D2A53F) crystal at top.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Iron sword icon.
Diagonal orientation (hilt bottom-left to tip top-right), simple cross-guard,
brown grip, polished blade.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Magical wand icon.
Horizontal orientation, ornate Old gold-trimmed wand, tip glowing Mage glow
violet (#9D6FE0) with small particle.
```

### Armor

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Leather chest armor
icon. Front-facing torso outline, brown leather, shoulder straps, small belt.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Iron helm icon.
Front-facing helmet, simple iron with eye slit, slight highlight on top.
```

### Key items (story)

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Borrowstone icon
(Act 2 reward). Smooth oval river-stone, Cobalt deep color, glowing Main teal
(#2A8482) ampersand "&" rune carved into face.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Lifetime Lantern
icon. Tiny brass lantern with Old gold flame inside, "'a" tick-rune visible on
the side.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Spellbook icon.
Closed leather-bound book, Oxblood (#6B1F35) cover, Old gold corner-caps, small
Main teal crystal embedded in center of cover.
```

### Materials (Act 6+ crafting)

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Iron ore chunk.
Irregular grey rock with metallic glints.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Bundle of herbs.
Three leafy green stems tied with cream twine.
```

```
[STYLE SUFFIX] RESOLUTION = 16×16 transparent. SUBJECT = Mana crystal shard.
Cobalt crystal facet (#377AB8) with Mist teal highlight, irregular shape.
```

---

## Status effect icons (24×24)

Slightly larger than inventory icons — these appear above unit sprites
during combat and need to read at a glance.

```
[STYLE SUFFIX] RESOLUTION = 24×24 transparent. SUBJECT = Poisoned status
icon. Skull silhouette with green wisps rising from it, palette greens
(Pine #27502E / Forest #487E40).
```

```
[STYLE SUFFIX] RESOLUTION = 24×24 transparent. SUBJECT = Burning status
icon. Small flame motif, layered Old gold / Brass leaf / Alarm scarlet palette colors.
```

```
[STYLE SUFFIX] RESOLUTION = 24×24 transparent. SUBJECT = Frozen status icon.
Snowflake-style geometric shape, Mist teal (#A4DED4) and Specular white.
```

```
[STYLE SUFFIX] RESOLUTION = 24×24 transparent. SUBJECT = Stunned status icon.
Three small spinning stars in a halo arrangement, Old gold (#D2A53F).
```

```
[STYLE SUFFIX] RESOLUTION = 24×24 transparent. SUBJECT = Borrowed status
icon (unique to this game). Glowing Main teal (#2A8482) ampersand "&" with
a faint chain link wrapping around it. Indicates the unit's reference is held elsewhere.
```

```
[STYLE SUFFIX] RESOLUTION = 24×24 transparent. SUBJECT = Moved status icon.
A small ghost-silhouette of an object with motion lines behind it. Indicates
ownership has transferred and the original is no longer valid.
```

---

## Menus and panels

### Main menu background

```
[STYLE SUFFIX] RESOLUTION = 640×360. SUBJECT = Main menu backdrop. Hero shot:
the player character and Ferris guide standing on a hill at dawn, looking
toward the Borrow Bridge in the middle distance and the Throne of the Compiler
on the far horizon. Sky gradient: Oxblood (top) to Old gold (horizon)
to Cobalt deep (bottom edge cropped). No menu buttons in this asset — those
are rendered by the engine on top.
```

### Main menu button — idle / hover / pressed

```
[STYLE SUFFIX] RESOLUTION = 192×40. SUBJECT = Pixel-art menu button frame,
three states. Idle = Parchment cream (#FCEFC8) fill, iron border, slight
top-left highlight. Hover = same but with a faint Bright teal glow on the
border. Pressed = bevel inverted (highlight bottom-right instead), 1px downward
offset suggested by shading. No text — engine renders.
```

### Settings panel frame

```
[STYLE SUFFIX] RESOLUTION = 480×320. SUBJECT = Settings panel frame, 9-slice
ready. Outer iron-bound double-line border, Parchment cream interior, a
section divider line halfway down to suggest two zones (audio above, gameplay
below). Small Oxblood decorative corners. No labels in image.
```

### Inventory grid background (per-cell)

```
[STYLE SUFFIX] RESOLUTION = 32×32. SUBJECT = Single inventory cell background.
Iron-bound square frame, Cobalt deep interior (#0E2E54), 1px inner shadow
suggesting depth. Designed to tile in a grid. Slot is empty in this asset.
```

### Inventory cell — hovered / selected

```
[STYLE SUFFIX] RESOLUTION = 32×32. SUBJECT = Same inventory cell, but with
border glowing Old gold (#D2A53F) for hover / selected state.
```

### Pause menu frame

```
[STYLE SUFFIX] RESOLUTION = 320×240. SUBJECT = Centered pause-menu panel,
Cobalt deep backdrop with subtle pixel-art smoke pattern, Parchment cream
inner panel with iron border, small Oxblood "PAUSED" placeholder area at
top (no actual text). Below: stacked button placeholders (3-4 button-shaped
zones).
```

---

## Dialogue and narrative UI

### Dialogue box — standard

```
[STYLE SUFFIX] RESOLUTION = 640×120. SUBJECT = Dialogue box frame at bottom
of screen. 9-slice ready. Parchment cream background (#FCEFC8), iron-bound
border with rivets, small portrait socket on the left (a 64×64 cutout area
with an Aged paper inner border for the speaker's portrait). Right side
of box reserved for text (rendered by engine, no text in image). Small
"continue" arrow indicator in bottom-right corner: a tiny Old gold
downward triangle, designed to animate by blinking on/off.
```

### Dialogue box — Borrow Checker variant

```
[STYLE SUFFIX] RESOLUTION = 640×120. SUBJECT = Same dialogue box, but with
border glowing Main teal (#2A8482) and small "&" rune accents in each corner.
Used when the Borrow Checker is speaking — visually distinct from generic NPCs.
```

### Dialogue box — Cliphy hint variant (Act 6+)

```
[STYLE SUFFIX] RESOLUTION = 480×100. SUBJECT = Smaller, friendlier hint
bubble. Parchment cream with a small tail/pointer in the bottom-left edge
(suggesting the hint emerges from somewhere off-screen). Border is softer,
Old gold (#D2A53F) with small star sparkle accents. No text.
```

### Speaker portrait socket — generic NPC

```
[STYLE SUFFIX] RESOLUTION = 64×64 transparent. SUBJECT = Empty portrait
socket frame: Cobalt deep background, iron-bound rectangular border with
ornate corners. Designed to have NPC portrait art composited into it
at runtime.
```

### Quest/encounter intro banner

```
[STYLE SUFFIX] RESOLUTION = 480×64. SUBJECT = Pixel-art banner that slides
in from the top of the screen at the start of a major encounter. Oxblood
(#6B1F35) with Crypt outline, ornate end-caps suggesting hanging tassels,
center area reserved for engine-rendered text.
```

---

## Compiler-output panel content stamps

These are the small visual stamps that appear inside the COMPILER OUTPUT
panel during specific events. Engine composites these into the panel.

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = "ERROR" stamp
icon: Alarm scarlet circle (#E63946) with Specular white "X" inside, slight
rough-stamped edge texture (palette only).
```

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = "WARNING" stamp
icon: Old gold triangle (#D2A53F) with Cobalt deep exclamation mark, rough-
stamped edge.
```

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = "PASSED" stamp
icon: Forest checkmark (#487E40) with Coalblack outline, rough-stamped
edge, slight glow.
```

```
[STYLE SUFFIX] RESOLUTION = 32×32 transparent. SUBJECT = "ELEGANT" stamp
icon: Old gold star (#D2A53F) with Parchment cream highlight, ornate edges.
Awarded for clippy-clean, idiomatic code.
```

---

## Title screen and end-of-demo

### Title wordmark

See `design/04-art-handoff-prompts.md` REF-10. The title screen is one of
the 10 reference images and gates bulk gen.

### End-of-demo screen

```
[STYLE SUFFIX] RESOLUTION = 640×360. SUBJECT = "End of demo" screen background.
Foreground: the player character standing victorious on the far side of the
Borrow Bridge, holding the Borrowstone aloft (it glows Main teal). The Borrow
Checker stands behind them, having bowed slightly. Mid-ground: the bridge
recedes into morning mist. Background: distant silhouettes of Acts 3-10 zones
(Guildhall towers, the Trait Mage's spire, distant Forge smoke, etc.) hinting
at the journey ahead. Email-signup placeholder area is bottom-third — leave
that area visually clean for engine-rendered form. Mood: triumphant, hopeful,
"there is so much more to come."
```

---

## Production checklist per UI element

- [ ] All states generated (idle / hover / pressed / disabled where applicable)
- [ ] Verified at base resolution AND at 2× / 3× integer scaling
- [ ] Verified in-engine (composited into actual UI, not just inspected as PNG)
- [ ] Filed under `assets/sprites/ui/<area>/<element>_<state>.png`
- [ ] Logged in `assets/sprites/ui/MANIFEST.md`
