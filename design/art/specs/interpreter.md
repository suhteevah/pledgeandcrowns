# The Interpreter — visual spec

**REF-44**, 32x32, idle, transparent bg.
**Mission:** `from_error` — implement `From` to convert one error into another.

## Curriculum cue

`impl From<A> for B` teaches the compiler how to turn an `A` into a `B` — and once it knows, the `?` operator will convert errors automatically as they bubble up. The Interpreter is the canonical translator: a complaint arrives in one tongue (one error type), and he hands back the same complaint in another (a different error type), losslessly. When the player writes `impl From<ParseError> for AppError`, they are teaching the world the Interpreter's trade: take the value of one type in, produce the value of another type out, by a rule that always holds.

## Visual brief

A poised figure, both hands raised, holding a small sign/token in each. **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, a clerk-translator's cut. **Inkblood** seams the deep shadow down the sides and the widening hem.

In the frame-left raised hand, the **source** token: a small **Forest** card with a **Spring meadow** highlight, **Coalblack**-edged — the incoming complaint in its original type.

In the frame-right raised hand, the **target** token: a small **Main teal** card with a **Bright teal** highlight, **Coalblack**-edged — the same complaint translated into the destination type. The two token colors (forest → teal) are the only accents on the figure, and the gesture of the two raised hands reads as the conversion between them.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band with an **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `K Y O W`. Skin (face, raised hands): `P R`. Neutrals (hair, outline, sign edges): `X B S V`. Gold (belt, buckle): `N G`. Forest (source token): `E H`. Teal (target token): `T I`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet (translation is craft, not spellcraft), no alarm scarlet, no specular white, no cobalt. Forest is confined to the source token, teal to the target token — the two type-colors, never the uniform.

## Lighting

Top-left 45°. Both tokens are raised to the same height and lit equally — the forest source on the left, the teal target on the right — because `From` is a faithful conversion: the two sides carry the same meaning, only the type changes. The robe's **Wineflesh** fold catches the light at the chest between the two raised arms.
