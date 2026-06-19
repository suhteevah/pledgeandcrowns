# The Bondsmith — visual spec

**REF-53**, 32x32, idle, transparent bg.
**Mission:** `closure_move` — take ownership of captured values into a closure via `move`.

## Curriculum cue

A `move` closure pulls a captured value out of the surrounding scope and *into* the closure, which now owns it — `move || { use(captured) }`. The value no longer lives outside; it has been taken in. The Bondsmith embodies this: he holds a locket-vessel close against his chest and seals a glowing item down into it with cupped hands. Once sealed, the item is his — it lives inside the vessel he carries, gone from the world outside. He does not borrow the thing; he takes it into himself and carries it away. When the player writes `move || use(data)`, they are doing the Bondsmith's move — capture the value by ownership, seal it into the closure, carry it where the borrow could not follow.

## Visual brief

A figure standing, both hands cupped close against the chest around a vessel. **Crypt** keeper tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, centered, **Inkblood** seaming the deep shadow and the widening hem.

At chest center, a **Bronze locket-vessel** with an **Antique brass** band, held close. Inside it, a sealed glow in **Old gold** brightening to a **Brass leaf** lit core — the captured value, now owned. Both **Pink quartz** / **Dusty rose** hands cup over the vessel, closing the seal — ownership moved in, not lent out. Below the glow the vessel band closes over with **Antique brass**, the item sealed inside.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band below the vessel, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, cupped hands): `P R`. Neutrals (hair, outline, vessel body): `X B S V`. Gold (sealed glow, vessel band, belt): `Z N G L`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. The glow is deliberately warm **Old gold**, not arcane violet — this is ownership taken in, not a spell. The sealed **Old gold → Brass leaf** glow is the one bright mass — the captured value the closure now owns.

## Lighting

Top-left 45°. The **Brass leaf** core of the sealed glow is the brightest point, at chest center inside the cupped hands; the **Bronze** vessel and **Antique brass** band catch the light around it. The glow is contained — held close, owned, carried — the value moved into the closure rather than borrowed from the scope outside.
