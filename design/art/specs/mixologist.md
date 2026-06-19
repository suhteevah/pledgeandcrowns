# The Mixologist — visual spec

**REF-45**, 32x32, idle, transparent bg.
**Mission:** `option_map` — `.map()` a closure over the inside of an `Option`.

## Curriculum cue

`Option::map` applies a transform to the value *inside* a `Some`, and quietly does nothing to a `None` — no branch, no panic, the transform just passes through the empty case. The Mixologist embodies this: she shakes and pours to change what is in the glass, but only *if* there is a drink in it. A full glass (`Some`) gets transformed; an empty glass (`None`) stays empty and she does not error. When the player writes `glass.map(|drink| shake(drink))`, they are doing the Mixologist's move — transform the contents when present, leave nothing-ness untouched.

## Visual brief

A deft figure mid-pour, a tool in each hand. **Crypt** tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, a working bartender's cut. **Inkblood** seams the deep shadow down the sides and the widening hem.

In the frame-left hand, a **Bronze** cocktail shaker: a **Bronze** body with **Antique brass** bands, capped by an **Old gold** top with a **Brass leaf** specular pixel — the transform she is about to apply.

In the frame-right hand, a glass holding a **Main teal** drink with a **Bright teal** highlight, **Coalblack**-outlined. A short **Mist teal** pour stream connects the shaker's reach to the glass — the transform landing on the contents. The teal drink is the single accent that marks the `Some` value; an empty glass would simply show no teal at all (the `None` case the map leaves alone).

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band with an **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin: `P R`. Neutrals (hair, outline, shaker/glass edges): `X B S V`. Gold (shaker body, bands, cap, belt, buckle): `Z N G L`. Teal (drink, pour stream): `T I M`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no forest beyond the pine hem. Teal is confined to the drink and the pour stream — the `Some` value being transformed, never the uniform.

## Lighting

Top-left 45°. The **Brass leaf** spec on the shaker cap is the brightest point top-left; the teal drink glows with its **Bright teal** highlight on the right. The pour stream reads as motion between them — the transform in the act of being applied to the contents that *are* there.
