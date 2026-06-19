# The Tidewatch — visual spec

**REF-57**, 32x32, idle, transparent bg.
**Mission:** `atomic` — increment a shared counter with `AtomicUsize`.

## Curriculum cue

The Tidewatch stands beside a notched tide-gauge post. As the tide rises she reads the gauge and the count climbs — a single mark advancing notch by notch up the post. Many waves push the water; every one of them nudges the *same* mark up by one, and no two pushes ever land on the same notch or lose a count. That is an `AtomicUsize`: a counter many threads can `fetch_add` at once, each increment whole and uninterrupted, the tally always exact. When the player writes `counter.fetch_add(1, Ordering::SeqCst)`, they are doing what the tide does to the gauge — bumping the one shared mark up by one, safely, from anywhere.

## Visual brief

A tidewatcher standing at frame-left, one arm reaching to the gauge post at frame-right. The house **Crypt**/**Oxblood** coat with **Wineflesh** lit folds; **Inkblood** for the deepest seams.

The tide-gauge post fills the right column: a **Bronze** timber outlined in **Coalblack**, capped with an **Antique brass** head, cut with regular **Aged paper** notch ticks up its length. Partway up the post stands the current count — a single **Old gold** tick mark with a **Brass leaf** specular pixel, the atomic counter's value, climbing. Her **Pink quartz**/**Dusty rose** hand reaches across to read it.

At the post's foot, the tide: a band of **Main teal** water with **Deep teal** shadow and **Bright teal** sparkle — the rising water whose every push fetch-adds the mark higher. This is the only teal mass, kept low at the waterline.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow/eye line, skin in **Pink quartz**/**Dusty rose**. **Antique brass** belt with **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (coat): `K Y O W`. Skin: `P R`. Neutrals (hair, notch ticks, outline): `X B S V`. Gold (climbing tick, post, buckle): `Z N G L`. Teal (tide water): `D T I`. Bog umber `U` (boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the one tick pixel, no cobalt, no forest beyond the pine hem.

## Lighting

Top-left 45° on figure and post. The brightest accents are the **Brass leaf** spec on the climbing **Old gold** tick and the **Bright teal** sparkle on the tide — the one shared count, the rising water that drives it.
