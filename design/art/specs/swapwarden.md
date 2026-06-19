# The Swapwarden — visual spec

**REF-63**, 32x32, idle, transparent bg.
**Mission:** `cell` — set or replace a single value through `Cell`.

## Curriculum cue

`Cell<T>` is interior mutability for one value at a time: `set` drops in a new value, `replace` swaps one out for another, all through a shared handle — no borrow check needed, because the old value simply moves out whole rather than being lent. The Swapwarden embodies this: she stands over a single socket-slot set in a pedestal, lifts the one coin sitting in it with two fingers while the next coin drops in to take its place. One value swapped for another in the same single cell — set and replace, no peeking, no borrowing. When the player writes `cell.set(x)` or `cell.replace(y)`, they are doing the Swapwarden's work — the slot's value changes, the old one moves out whole.

## Visual brief

A figure standing over a pedestal-slot, pinching a coin between two fingers above it. **Crypt** keeper tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, **Inkblood** seaming the deep shadow and hem.

The pedestal fills the lower-center of the frame: a **Bronze** column with a **Bog umber** base, topped by a single socket framed in **Antique brass**, one **Old gold** coin seated in the slot. Above the slot, a second **Old gold** coin — each coin carrying a **Brass leaf** specular highlight — is lifted out by a **Pink quartz** / **Dusty rose** pinch of two fingers, mid-swap. Slot seam and outlines in **Coalblack**.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, swapping fingers): `P R`. Neutrals (hair, outline): `X B S V`. Gold (the two coins, slot rim, pedestal): `U Z N G L`. Bog umber `U` (boots, pedestal base), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the coin specs, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The two brightest accents are the **Brass leaf** specs on the two coins — one lifting out above the slot, one seated in it. The two coins reading as one-replacing-the-other in the same single **Bronze** socket is the swap: `set` and `replace`, mutate-through-shared without a borrow, the old value moving out whole.
