# The Alchemist — visual spec

**REF-29**, 32×32, idle, transparent bg.
**Mission:** `iter_map_collect` — map a `Vec` through `|x| x * 2`, `collect`.

> Status: **spec drafted 2026-06-18, JSX/PNG pending.** The last of the four
> remaining `SPRITE_PLAYER` placeholders. Art *direction* only; JSX +
> rendered PNG go through `claude.ai/design` + Matt approval per the locked
> art process.

## Curriculum cue

The iterator triple — `iter` / `map` / `collect`: walk a sequence,
transform **each** element with a closure (`|x| x * 2`), then gather the
results into a new container. The Alchemist is transmutation as a line:
identical inputs go in, each is changed the same way, and the changed
results are collected into one new vessel.

## Visual brief

An alchemist standing behind a workbench that fills the lower-third of the
frame, leaning slightly forward over the work. **Crypt** burgundy robe with
rolled sleeves, **Inkblood** shadow, **Oxblood** mid; a scorched **Bog
umber** leather apron over the front, **Bronze** buckle.

The bench tells the whole algorithm, read left to right:
- **Input Vec (left):** a row of three identical small vials, each holding
  a shallow measure of **Deep teal** reagent — the unmapped elements.
- **Map (centre):** a small **Antique brass** still / burner where one vial
  is mid-transmutation — its contents lifted to a brighter, *fuller*
  **Bright teal**, twice the level (`|x| x * 2`). A thin **Mist teal**
  vapour curl rises from it: the per-element transform.
- **Collect (right):** one large round-bottomed **Brass leaf**-rimmed flask,
  already holding a deeper pool of the doubled **Main teal** liquid — the
  new `Vec` the results are gathered into. A glass funnel (**Aged paper**
  glint) feeds the flask: the `collect`.

His right hand tips the mid-transform vial toward the funnel; left hand
steadies the collecting flask. The motion is one element completing the
map→collect hop.

Head: short **Stone grey** hair, **Wineflesh** skin (`P R W`), a leather
strap holding a single **Antique brass** loupe pushed up onto the brow. No
beard; a faint **Bright teal** under-light on the jaw from the glowing
flask.

## Palette compliance

Burgundy (robe, skin): `K Y O W R P`. Gold (apron, still, flask rim,
buckle, loupe): `U Z N G L`. **Teal (the reagent — the distinct accent that
sets the Alchemist apart from the gold-and-burgundy cast):** `A D T I M` —
Deep teal inputs, Bright teal mid-transform, Main teal collected pool, Mist
teal vapour. Neutrals (hair, outline, funnel): `X B S V`.

Teal stays the *alchemical fluid only* and well under a quarter of the
canvas (the split-complement is an accent ramp, not the body). No magic
violet, no alarm scarlet, no specular white, no cobalt, no forest.

## Lighting

Top-left 45°, but with a secondary **Bright teal** glow rising from the
collecting flask (the only non-top light in the sprite) catching the jaw
and the funnel underside. Brightest pixel is the **Mist teal** vapour curl
at the burner — the instant of transformation, the `* 2` happening to one
element.
