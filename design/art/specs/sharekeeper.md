# The Sharekeeper — visual spec

**REF-61**, 32x32, idle, transparent bg.
**Mission:** `rc_basic` — share ownership of a value with `Rc`.

## Curriculum cue

`Rc::new(value)` creates a reference-counted handle; `Rc::clone(&handle)` hands out another share of the *same* value, bumping the count. No share owns the value alone — the value lives as long as one owner remains, and is freed only when the last share is dropped. The Sharekeeper embodies this: she holds one gold coin-hoard while several hands reach in from the sides, each laying a tally-token claim on the same hoard, and a small count marker beside it ticks the number of claims. When the player writes `Rc::clone(&rc)`, they are doing the Sharekeeper's work — another hand on the same hoard, the count up by one, the value kept alive by shared ownership.

## Visual brief

A figure standing, centered, holding a single shared hoard-chest at chest height. **Crypt** keeper tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, **Inkblood** seaming the deep shadow and hem.

At chest center, one **Bronze**-bodied hoard with **Antique brass** bands, brimming with **Old gold** coins, two of which carry **Brass leaf** specular highlights. Reaching in from both the left and right edges of the frame are several **Pink quartz** / **Dusty rose** hands — multiple owners, each claiming the same hoard. To the upper-right, a small **Old gold** count marker (a two-tile token with a **Coalblack** digit) records the reference count.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, reaching hands): `P R`. Neutrals (hair, outline): `X B S V`. Gold (hoard coins, chest body, count marker): `U Z N G L`. Bog umber `U` (boots, chest base), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the coin specs, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The brightest accents are the **Brass leaf** specs on the hoard coins. The single gold hoard with many hands on it is the shared value; the small **Old gold** count marker is the reference count — when it reaches zero, the hoard is freed. One value, many owners, kept alive by the count.
