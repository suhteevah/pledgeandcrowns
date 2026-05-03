# The Herald — visual spec

**REF-17**, 32×32, idle, transparent bg.
**Mission:** `struct_basic` — teaches struct definition with named fields.

## Curriculum cue

A `struct` is a *named tuple of named things*. The Herald's job in any medieval court is the ceremonial reading-out of titles: "Sir Aldric, of House Vale, bearing arms argent on field gules..." — every announcement is a flat record of named fields. His tabard is literally a struct laid flat across his chest: three vertically-stacked color-blocked panels, each in a different palette ramp, each one a "field" the player can read off at a glance.

## Visual brief

Trim, formal posture — feet together, shoulders square. Wears a **tabard** over a **Pine** undertunic. The tabard is the focal point: three horizontally-banded blocks down the front, each a distinct heraldic field — top band **Oxblood** (the burgundy field), middle band **Old gold** (the gold field), bottom band **Forest** (the forest field). One thin **Coalblack** rule between each band, like a struct definition's separators. The tabard hem is **Brass leaf** trimmed.

In his right hand he holds a small **rolled scroll** at his side — pure **Parchment cream** core with **Antique brass** seal on the visible end. The scroll is the call site that reads the struct's fields aloud. Belt is **Bronze** with a small **Old gold** buckle.

Head: short Bronze hair, square-cut. Pink quartz / Dusty rose / Wineflesh skin ramp matching the rest of the village cast. One pixel of Coalblack each for eyes; mouth a single Wineflesh pixel set in a neutral line.

## Palette compliance

Burgundy (top tabard band, skin): `K Y O W R P`. Gold (middle band, hair, scroll seal): `Z N G L`. Forest (bottom band, undertunic): `J F E`. Neutrals (outline, scroll core, belt accents): `X B S V C`. The Parchment-cream scroll is allowed; `C` is already in the gold ramp.

No magic violet, no alarm scarlet, no specular white, no teal.

## Lighting

Top-left 45°. Each tabard band gets a one-pixel brighter row at its top edge (Wineflesh on Oxblood, Brass leaf on Old gold, Spring meadow on Forest) — sells the layered-fields read.
