# The Loremaster — visual spec

**REF-41**, 32x32, idle, transparent bg.
**Mission:** `assoc_type` — an associated type (`type Output`) on a trait.

## Curriculum cue

An associated type binds one output type to a trait per implementor: `trait Spell { type Output; fn cast(&self) -> Self::Output; }`. Each implementor names exactly one `Output` — not a free type parameter the caller picks, but a single result fixed by the impl. The Loremaster holds an open tome whose page shows a mapping: one input glyph bound to one output glyph, the way each entry in the book names exactly one result. The associated type is that binding — one trait, one named `Output`, written into the lore.

## Visual brief

A scholar, centered and standing, holding an open tome forward at waist height with both hands bracing the outer edges. The tome is **Old gold**-trimmed with a **Bronze** spine down the center and an **Antique brass** clasp. The two **Parchment cream** pages each carry a small **Coalblack** inscription: the left page shows an input glyph, the right page its bound output glyph — one mapping, the `type Output` written as lore.

The figure wears a **Crypt** scholar's robe with **Oxblood** primary and **Wineflesh** lit folds. The hands (**Pink quartz** / **Dusty rose**) brace the book.

Head: short **Stone grey** hair with **Aged paper** highlight, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**. **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `Y O W`. Skin/hands: `P R`. Neutrals (hair, outline, inscription): `X B S V`. Gold (tome trim, spine, clasp): `N Z G`. Parchment cream (pages): `C`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet — this is lore, not spellcraft. No alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The robe's left folds catch **Wineflesh**. The open **Parchment cream** pages are the brightest mass, framed in **Old gold** — the lore lit and held forward; the eye lands on the two-glyph mapping, the associated type made readable.
