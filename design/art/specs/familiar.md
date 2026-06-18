# The Familiar Keeper — visual spec

**REF-39**, 32x32, idle, transparent bg.
**Mission:** `dyn_trait` — `Box<dyn Trait>`, dynamic dispatch over many types.

## Curriculum cue

`Box<dyn Trait>` is one container that can hold *any* concrete type implementing the trait, with the exact type erased: `let f: Box<dyn Familiar> = ...`. You no longer know which element the value is — you only know it speaks the `Familiar` trait, and the call dispatches at runtime. The Keeper carries a polyglot familiar in a cage: the creature is painted in several element colors at once because the cage (the box) hides which concrete element it actually is. The cage is the `Box`; the multi-colored creature is `dyn Familiar`.

## Visual brief

A figure offset toward frame-left, the right arm raised to hold a small cage aloft in the upper-right of the frame. The cage is a **Bronze** frame with **Coalblack** bars and **Antique brass** rings top and bottom. Inside, the familiar reads as many types in one container: a **Main teal** wing, an **Old gold** body, a **Forest** tail, with a single **Basalt** eye. The deliberate clash of teal + gold + forest in one tiny creature is the whole point — concrete types hidden behind one dyn container.

The figure wears a **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds. The raised hand (**Pink quartz** / **Dusty rose**) grips the cage hook.

Head: short **Stone grey** hair with **Aged paper** highlight, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**. An **Antique brass** belt band with **Old gold**, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `Y O W`. Skin: `P R`. Neutrals (hair, outline, cage bars): `X B S V`. Gold (cage frame/rings, creature body, belt): `Z N G`. Teal (creature wing): `T`. Forest (creature tail): `E`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet — this is dynamic dispatch, not spellcraft; the accent is the creature's element clash, not arcane glow. No alarm scarlet, no specular white, no cobalt. Teal and forest are confined to the single caged creature.

## Lighting

Top-left 45°. The robe's left folds catch **Wineflesh**. The cage is the bright focal cluster in the upper right — teal, gold, and forest jammed together — so the eye reads "many types, one box" before it reads the keeper.
