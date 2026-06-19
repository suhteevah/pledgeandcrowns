# The Warden — visual spec

**REF-62**, 32x32, idle, transparent bg.
**Mission:** `refcell` — mutate through a shared handle with `RefCell`.

## Curriculum cue

`RefCell<T>` gives you interior mutability: the box is handed to you shared and sealed — "look, don't touch" by the type system — yet you can still `borrow_mut()` and change what is inside, because the borrow rules are moved from compile time to a check enforced at *runtime*. The Warden embodies this: he guards a lockbox latched in one arm, turns a key in the lock with the other to lift the lid and change the contents, and keeps a small ledger at his hip recording who is borrowing right now, so two hands never reach in at once. When the player writes `cell.borrow_mut()`, they are doing the Warden's work — the runtime check passes, the lid lifts, the contents change through a shared handle.

## Visual brief

A figure standing, one arm latched around a lockbox, the other reaching across to turn a key in its lock. **Crypt** keeper tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, **Inkblood** seaming the deep shadow and hem.

The lockbox sits at chest/waist: a **Bronze** body with **Antique brass** bands and an **Old gold** lock plate, lid cracked open along an **Antique brass** seam. An **Old gold** key, with a **Brass leaf** specular highlight, is turned in the lock by a **Pink quartz** / **Dusty rose** hand. Keyhole and seams in **Coalblack**; box base in **Bog umber**. At the figure's hip, a small **Parchment cream** borrow-ledger with **Coalblack** tick marks and an **Aged paper** ruled line — the record of the current borrower.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, key hand): `P R`. Neutrals (hair, outline): `X B S V`. Gold (key, lock plate, bands): `U Z N G L`. Parchment cream `C` (borrow-ledger). Bog umber `U` (boots, box base), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the key spec, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The brightest accent is the **Brass leaf** spec on the turning key. The **Old gold** lock plate and key are the runtime borrow check; the lifting lid is the interior mutation; the **Parchment cream** ledger is the borrow flag — one borrower at a time, checked when you reach in, not when you compile.
