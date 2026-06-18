# The Porter — visual spec

**REF-33**, 32x32, idle, transparent bg.
**Mission:** `while_let` — loop popping while a variant remains.

## Curriculum cue

`while let Some(item) = stack.pop() { ... }` drains a collection one element at a time until it runs dry. The Porter unloads a stack of crates: he lifts the top crate off the pile, and he will keep lifting — pop, pop, pop — until the stack returns `None` and there is nothing left to take. The pile to his side is the `Vec`; the crate in his hands is the current `Some`; the loop ends when the floor shows.

## Visual brief

A labourer, working tunic rather than formal robe. **Crypt** tunic with **Oxblood** primary and **Wineflesh** lit folds, a **Bronze** belt band — plainer and shorter than the officer cast, a working garment.

Both arms reach up and to his right, lifting one crate clear off the top of the stack: a **Bog umber** crate with **Antique brass** corner brackets and an **Old gold** carry-strap across its face — the current `Some(crate)`. His hands (**Pink quartz** / **Dusty rose**) grip its underside; the left hand also rests at the hip.

Beside him, a stack of three more **Bog umber** crates rises from the floor, each banded with **Antique brass** corner brackets, **Coalblack**-outlined, sitting on a **Coalblack** floor line — the remaining stack the loop will keep draining.

Head: short **Stone grey** hair with **Aged paper** highlight, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**. **Bronze**/**Bog umber** boots, **Pine** hem at the tunic.

## Palette compliance

Burgundy (tunic): `Y O W`. Skin: `P R`. Neutrals (hair, outline, crate edges): `X B S V`. Gold (crate brackets, carry-strap, boots, belt): `Z N G`. Bog umber `U` (crate bodies, boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The lifted crate catches the most light — **Old gold** strap and **Antique brass** brackets reading bright — because it is the element currently in hand. The stacked crates shade uniformly in **Bog umber**: the queued elements, waiting their turn to be popped.
