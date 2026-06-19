# The Vaultwright — visual spec

**REF-60**, 32x32, idle, transparent bg.
**Mission:** `box_basic` — put a value on the heap with `Box::new`.

## Curriculum cue

`Box::new(value)` moves a value to the heap and hands you back a single pointer to it — you stop carrying the bulk of the value and start carrying a handle. The Vaultwright embodies this: he cradles a sturdy gold-bound strongbox in both arms, the value lifted off the ground and set down inside, the lid latched. He does not hold the value loose in his hands; he boxes it — puts the thing in a strongbox and from then on carries the box, not the heap of stuff. When the player writes `Box::new(x)`, they are doing the Vaultwright's work: the value moves to the heap, and what they hold is the pointer to the box.

## Visual brief

A figure standing, centered, both arms come round to cradle a strongbox held against the body. **Crypt** keeper tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, **Inkblood** seaming the deep shadow and the widening hem.

The strongbox dominates the lower chest: a **Bronze** body with **Antique brass** bands top and bottom, an **Old gold** latched lid, and a central lock plate in **Old gold** carrying a single **Brass leaf** specular highlight. The keyhole and chest seam are picked out in **Coalblack**. The box base sits in **Bog umber**. Both **Pink quartz** / **Dusty rose** arms wrap around the box from the sides, cradling it.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, cradling arms): `P R`. Neutrals (hair, outline): `X B S V`. Gold (chest lid, bands, lock): `U Z N G L`. Bog umber `U` (boots, chest base), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the single lock spec, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The brightest accent is the **Brass leaf** specular on the lock plate at chest center; the **Old gold** lid catches the light above the **Bronze** body. The whole gold mass of the strongbox is the heap allocation — one bright, solid object cradled close, the value boxed and the handle carried.
