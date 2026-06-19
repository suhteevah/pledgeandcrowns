# The Smelter — visual spec

**REF-50**, 32x32, idle, transparent bg.
**Mission:** `iter_fold` — reduce many items to one value via `.fold`.

## Curriculum cue

`.fold(init, |acc, x| ...)` starts with a seed, combines each item into a running accumulator, and ends with a single value. The Smelter embodies this: at his crucible he pours a stream of molten metal down into one ingot mold — many scraps went into the crucible, *one* ingot comes out. He carries the growing pool (the accumulator) and folds every piece into it, melt by melt, until all that remains is the single result. When the player writes `iter.fold(0, |acc, x| acc + x)`, they are doing the Smelter's move — seed the pool, melt each item in, walk away with one ingot.

## Visual brief

A figure standing at a furnace, frame-right hand tipping a crucible. **Crypt** forge tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, drawn off-center so the crucible and mold sit to his frame-right. **Inkblood** seams the deep shadow and the widening hem.

To the frame-right, a **Bronze crucible** bound with an **Antique brass** band, brimming with molten metal in **Old gold** with a **Brass leaf** lit core. From its lip a stream of molten **Old gold** pours down into a single **ingot mold** below — a **Bronze** mold with an **Antique brass** base band and a **Bog umber** shadow, filling with **Old gold** that brightens to **Brass leaf** as the ingot gathers. Many melts, one ingot — the fold's single result.

A single **Main teal** pixel of heat-shimmer rises above the crucible — the only cool accent, reading as forge heat, not magic.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band, **Pine** hem, **Bronze**/**Bog umber** boots. The **Pink quartz** / **Dusty rose** hand grips the crucible.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, hand): `P R`. Neutrals (hair, outline, crucible/mold): `X B S V`. Gold (molten pour, ingot, crucible band, belt): `Z N G L`. Teal `T` (single heat-shimmer pixel only). Bog umber `U` (mold/boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no forest. The lone **Main teal** pixel is heat-shimmer, capped at one. The **Old gold → Brass leaf** molten ramp is the bright mass — the value the fold builds.

## Lighting

Top-left 45°. The brightest points are the **Brass leaf** core of the molten metal in the crucible and the gathering ingot. The molten **Old gold** glow lights the whole frame-right; the mold's **Bog umber** base falls to shadow beneath the finished ingot — the one value left when every item has been folded in.
