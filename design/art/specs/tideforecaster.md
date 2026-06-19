# The Tideforecaster — visual spec

**REF-59**, 32x32, idle, transparent bg.
**Mission:** `async_fn` — write an `async fn` that returns a future you `await`.

## Curriculum cue

The Tideforecaster stands at the sea's edge, spyglass raised, a tide-almanac under her arm, gazing at the horizon — waiting for a tide the almanac promises but that has not yet arrived. She does not hold the water in her hands; she holds the *promise* of it. The tide is coming; when it does she will read it, but until then she watches, ready, not blocking the harbor. That is an `async fn`: it returns a future — a value that is not here yet but will be — to be `await`ed rather than blocked-for. When the player writes `async fn tide() -> Tide { ... }` and `tide().await`, they are doing what the Forecaster does — declaring a result that will arrive and awaiting it.

## Visual brief

A forecaster standing at the shoreline, centered, spyglass raised to one eye. The house **Crypt**/**Oxblood** coat with **Wineflesh** lit folds; **Inkblood** for the deepest seams.

In her raised hand, a spyglass to the eye: a **Bronze** body (**Coalblack** outline, **Antique brass** trim) with an **Old gold** end cap and a **Brass leaf** specular pixel at the lens — the `await`, poised on the promised tide. Under her other arm, a tide-almanac: **Parchment cream** pages with **Aged paper** ruled lines and **Coalblack** binding — the `fn` signature that declares the future will come.

The setting carries the metaphor. A thin **Cobalt deep** band runs across the very top — the night sky, a small cool accent. Across the bottom rows, the sea: **Main teal** water with **Deep teal** shadow and a **Bright teal** horizon line and sparkle — the future not yet arrived, the tide she awaits.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow/eye line, skin in **Pink quartz**/**Dusty rose**. **Antique brass** belt with **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (coat): `K Y O W`. Skin: `P R`. Neutrals: `X B S V`. Gold (spyglass, buckle): `Z N G L`. Teal (sea, horizon): `D T I`. Cobalt (night-sky band): `<`. Parchment `C` (almanac), Bog umber `U` (boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the one lens pixel; cobalt limited to the single night-sky band at top.

## Lighting

Top-left 45° on the figure. The two brightest accents are the **Brass leaf** spec on the raised spyglass lens and the **Bright teal** horizon line of the sea — the await, and the tide that has not yet come in.
