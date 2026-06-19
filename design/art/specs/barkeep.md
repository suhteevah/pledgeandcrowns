# The Barkeep — visual spec

**REF-42**, 32x32, idle, transparent bg.
**Mission:** `result_match` — `match` on a `Result`, handle both `Ok` and `Err`.

## Curriculum cue

A `Result<T, E>` is either `Ok(value)` or `Err(error)`, and a `match` on it must handle *both* arms — you cannot use the value until you have accounted for the failure. The Barkeep is the canonical "two outcomes, both handled" figure: one mug comes back full (`Ok`), one comes back spilled (`Err`). The player who writes `match drink { Ok(ale) => ..., Err(e) => ... }` is doing exactly what the Barkeep does every shift — serving the good pour and mopping up the bad one, never pretending the spill didn't happen.

## Visual brief

A genial tradesman standing behind a **Bronze** bar/counter that fills the lower third of frame, **Bog umber** front face with **Bronze** plank seams and a **Bronze** base trim. **Crypt** apron-tunic over the body with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy uniform in a working apron cut, with an **Antique brass** waistband. **Inkblood** seams the apron's deepest shadow at the sides.

In the frame-left hand, a mug standing **upright and full**: a **Bronze** body with **Antique brass** bands, brimming with **Old gold** ale and topped by a **Brass leaf** foam head — this is the `Ok(value)`, the good pour.

In the frame-right hand, a mug **tipped and spilling**: rim-down, its ale gone, **Antique brass** bands and a **Bronze** body now empty, with a single **Alarm scarlet** drip falling from it — this is the `Err`. The lone scarlet pixel is the only alarm on the whole sprite (well under 1%), the one thing the player must not forget to handle.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Pine** hem below the bar, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (apron-tunic): `K Y O W`. Skin: `P R`. Neutrals (hair, outline): `X B S V`. Gold (bar, mug bodies, bands, ale, foam): `U Z N G L`. Bog umber `U` (bar front, boot shadow), Pine `F` (hem). One **Alarm scarlet** `!` pixel on the spilled mug — the `Err` accent, ≤1%.

No magic violet, no specular white, no cobalt, no teal, no forest beyond the pine hem. The single `!` drip is the only alarm pixel.

## Lighting

Top-left 45°. The full mug's **Old gold** ale and **Brass leaf** foam are the brightest mass on the frame-left; the spilled mug on the right reads darker and broken, ending in the lone scarlet drip — the two outcomes lit unequally on purpose, because `Ok` and `Err` are not the same and the `match` must tell them apart.
