# The Conjurer — visual spec

**REF-38**, 32x32, idle, transparent bg.
**Mission:** `generic_struct` — a struct generic over `T` (`Pair<T>`).

## Curriculum cue

A generic struct holds values of a type parameter: `struct Pair<T> { a: T, b: T }`. Both fields are the *same* `T` — a `Pair<i32>` holds two ints, never an int and a string. The Conjurer conjures exactly this: one orb in each hand, and because they are two fields of the same `Pair<T>`, they must read as identical — same shape, same color, same glow. If the two orbs looked different, the picture would be lying about the type.

## Visual brief

A centered, standing mage in a **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds, **Inkblood** seam shadow at the widening hem. The hands come forward at waist height, each cupping a small conjured orb. The two orbs are deliberately, pixel-for-pixel identical: a **Mage glow** core with a thin **Royal arcane** halo, one at frame-left, one at frame-right, at the same height — the two fields of `Pair<T>`, the one type appearing twice.

Head: short **Stone grey** hair with **Aged paper** highlight, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**, hands in **Pink quartz** / **Dusty rose**. An **Antique brass** belt band with an **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `K Y O W`. Skin/hands: `P R`. Neutrals (hair, outline): `X B S V`. Gold (belt band, buckle): `N G`. Magic (the two orbs only): `* %` — matched cores and thin halos, together just under the 5% cap. Bog umber `U` (boots), Pine `F` (hem).

No alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The robe's left folds catch **Wineflesh**; the right falls to **Crypt**/**Inkblood**. The two **Mage glow** orbs are lit equally and identically — the point of the piece is that they match, because `Pair<T>` binds both fields to the one `T`.
