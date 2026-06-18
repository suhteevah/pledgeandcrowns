# The Locksmith — visual spec

**REF-32**, 32x32, idle, transparent bg.
**Mission:** `if_let` — match one variant, ignore the rest.

## Curriculum cue

`if let Some(x) = opt { ... }` is the one key that fits. You have a ring of possibilities; only the matching variant turns the lock, and the rest are passed over without ceremony — no `match` arm for `None`, no exhaustive handling, just the single case you care about. The Locksmith raises one key clear of the ring: that is the `Some` arm. The hanging ring of other keys is everything `if let` quietly ignores.

## Visual brief

A standing tradesman. **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds, **Antique brass** collar studs — workmanlike burgundy uniform.

Raised high in the right hand, held clear above the shoulder, a single key singled out: an **Old gold** bow (ring-head) with a **Brass leaf** specular pixel, a **Coalblack**-cored shaft, and an **Old gold** bit at the bottom — the one key that fits, the brightest gold mass and the highest point in the sprite.

Below the raised key, the hand grips a **Bronze**-banded key-ring from which two more **Old gold** keys hang (their shafts **Coalblack**-cored, an **Antique brass** shadow between them) — the variants `if let` ignores. The hand reads in **Pink quartz** / **Dusty rose**.

Head: short **Stone grey** hair with **Aged paper** highlight, clean-shaven, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**. Standard **Old gold** belt buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `Y O W`. Skin: `P R`. Neutrals (hair, outline, key cores): `X B S V`. Gold (collar stud, key-ring band, keys, raised-key spec, buckle): `N Z G L`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The raised key takes the **Brass leaf** specular on its bow — the matching variant, lit. The hanging ring stays in flatter **Old gold** with **Antique brass** shadow, deliberately less bright: the ignored variants, present but unhandled.
