# The Pilgrim — visual spec

**REF-23**, 32×32, idle, transparent bg.
**Mission:** `option_unwrap_or` — `.unwrap_or(default)`.

## Curriculum cue

`Option<T>` is the language's "may or may not be there." `.unwrap_or(d)` is the combinator that *picks the path with the lit lantern* — if the value is `Some`, take it; if `None`, fall back to `d`. The Pilgrim is a road-walker who carries that fallback with him: a lantern, lit, swinging from his right hand. Voice spec: "the absent path lit a default". The lantern flame *is* the default value; it is what burns when the world hands you nothing.

## Visual brief

Lean traveller, hooded, moderate stride implied by a slight lean forward. Long road-cloak in **Pine** (forest mid) with **Mossbed** shadows and a single row of **Spring meadow** highlight along the left edge — green, not burgundy: the Pilgrim is *not* of the village uniform; he comes from elsewhere. Hood up, casting his upper face in **Inkblood** shadow — only the lower jaw and a glint of one eye visible (single `S` pixel). **Aged paper** wrap around the chin like a pilgrim's scarf.

In the right hand, held away from the body at hip height, an iron-frame lantern: **Coalblack** outline cage, **Basalt** body, with the lantern *glass* a 2×2 patch of **Old gold** with one **Brass leaf** specular pixel as the flame core. That flame is the only bright on the sprite. A short **Bronze** chain links the lantern to the hand. Walking staff in the left hand, **Bronze** with a **Stone grey** ferrule.

Boots **Bog umber**, dust on the hem (1 row of `S` flecks at the boot tops). Skin: the visible jaw is **Pink quartz**/**Dusty rose**.

The lantern glow on the cloak is implied with two **Antique brass** pixels on the hood's lower right edge — the default value subtly tinting the surroundings, as fallbacks do.

## Palette compliance

Forest ramp (cloak): `J F E H`. Gold ramp (lantern glass, flame, chain, staff, hood-glow tint): `Z N G L`. Neutrals (lantern frame, hair, outline, scarf, glint): `X B S V`. Skin: `P R W`. Boots: `U`.

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no burgundy (deliberate — the Pilgrim is a wanderer, not of Hearthstone).

## Lighting

Top-left 45° from the lantern itself, not from above. The lantern is the local light source — left edge of the figure (closer to lantern) gets the implied glow; right edge stays in **Mossbed**.
