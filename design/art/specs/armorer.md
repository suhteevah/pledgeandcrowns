# The Armorer — visual spec

**REF-35**, 32x32, idle, transparent bg.
**Mission:** `enum_data_match` — one enum, two payloads, exhaustive match.

## Curriculum cue

An enum can carry data per variant: `enum Item { Weapon(Blade), Potion(Vial) }`. A `match` on it must handle *every* variant, and each arm unpacks that variant's own payload. The Armorer holds both at once — a blade in one hand, a potion in the other — the two variants of the same `Item` enum, each with different data inside. To use an `Item`, the player must `match` both arms; leaving one out won't compile. This is the Armorer, not the Smith: leaner, no anvil, standing by a weapon rack.

## Visual brief

A lean tradesman, working tunic. **Crypt** tunic with **Oxblood** primary and **Wineflesh** lit folds, a **Bronze** belt band — trimmer than the Smith, no apron.

To his left (frame-left), a weapon rack: a **Bronze** top rail with **Antique brass** pegs, spare **Stone grey** blades (one tipped **Aged paper**) hanging in its slots. He stands beside it, not at an anvil.

In the left hand, raised, the **Weapon** variant: a **Stone grey** blade with an **Aged paper** highlight at the tip, a **Coalblack** core, an **Antique brass** hilt guard, a **Bronze** grip, and an **Old gold** pommel — the variant carrying its `Blade` payload.

In the right hand, held out, the **Potion** variant: a small vial of **Main teal** fluid with a **Bright teal** highlight, **Coalblack**-outlined — the variant carrying its `Vial` payload. Teal is the single accent that marks the alchemical item, consistent with the Alchemist's reagent coding, and it appears only on the vial.

Head: short **Stone grey** hair with **Aged paper** highlight, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**. **Old gold** belt buckle area, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `Y O W`. Skin: `P R`. Neutrals (hair, outline, blade, rack blades): `X B S V`. Gold (rack rail, pegs, hilt guard, grip, pommel, belt): `Z N G`. Teal (potion fluid only): `T I`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no forest beyond the pine hem. Teal is confined to the single potion vial — the variant accent, never the uniform.

## Lighting

Top-left 45°. The raised blade catches **Aged paper** at the tip; the **Old gold** pommel anchors the bottom of the weapon. The teal vial glows with a **Bright teal** highlight on its right — the two variants lit equally, because a `match` must honor both.
