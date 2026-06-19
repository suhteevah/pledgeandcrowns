# The Strongbox — visual spec

**REF-64**, 32x32, idle, transparent bg.
**Mission:** `rc_refcell` — share *and* mutate with `Rc<RefCell<T>>`.

## Curriculum cue

`Rc<RefCell<T>>` is the two combined: shared ownership (the `Rc`, many handles, the reference count) wrapped around interior mutability (the `RefCell`, a lid that opens to a runtime-checked borrow). Many owners hold a handle to the same value, and any of them can open it to mutate the contents. The Strongbox keeper embodies this: he stands behind a heavy strongbox with multiple keyholes across its face, several keys ringed and shared out, the lid cracked open mid-turn so the contents can be changed. When the player writes `Rc::clone(&shared)` then `shared.borrow_mut()`, they are doing the Strongbox keeper's work — another key for another owner, and the lid that opens to change what is inside.

## Visual brief

A figure standing behind a heavy strongbox that fills the chest and waist. **Crypt** keeper tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, **Inkblood** seaming the deep shadow and hem.

The strongbox is a broad **Bronze** body with several **Antique brass** bands across it, a **Bog umber** base, and a row of **Old gold** lock plates — multiple of them — each with a **Coalblack** keyhole, several plates carrying a **Brass leaf** specular highlight. Above the box, a **Pink quartz** / **Dusty rose** hand lifts a shared ring of several **Old gold** keys; the top band reads as a lid cracked open mid-turn. Seams and keyholes in **Coalblack**.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, key hand): `P R`. Neutrals (hair, outline): `X B S V`. Gold (keys, lock plates, bands): `U Z N G L`. Bog umber `U` (boots, box base), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the key/plate specs, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The brightest accents are the **Brass leaf** specs on the shared keys and lock plates. The heavy strongbox with *multiple* keyholes and *several* shared keys is the `Rc` — shared ownership; the lid cracked open mid-turn is the `RefCell` — interior mutation, borrow-checked at runtime. Many owners, each able to open and change.
