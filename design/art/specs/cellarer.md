# The Cellarer — visual spec

**REF-47**, 32x32, idle, transparent bg.
**Mission:** `unwrap_or_else` — lazy default via `.unwrap_or_else(|| ...)`.

## Curriculum cue

`.unwrap_or_else` takes the value if it is present, and otherwise *calls a closure* to produce a fresh default — and crucially, that closure runs only when it is actually needed, not before. The Cellarer embodies this lazy default: if the cask still holds brew, he ladles from it; if the cask is empty, he does not error — he brews a fresh batch on the spot. The work of making the default is done only when the cask runs dry. When the player writes `cask.unwrap_or_else(|| brew_fresh())`, they are doing the Cellarer's move — use what's there, else do the work to make a new one, and never do that work unless you have to.

## Visual brief

A figure standing beside a cask, ladle in hand. **Crypt** cellar tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, drawn slightly off-center so the cask sits to his frame-left. **Inkblood** seams the deep shadow down the sides and the widening hem.

To the frame-left, a wooden **cask/barrel**: **Bronze** staves bound by **Antique brass** hoops top and bottom, with a band of **Old gold** brew level visible near the top (a single **Brass leaf** specular pixel on it) and a **Bog umber** shadow at its base. The cask is the source you draw the default from.

In the frame-right hand, a **Bronze** ladle: a **Bronze** bowl on a **Bronze** handle, **Coalblack**-outlined, raised as if mid-scoop — the tool that either draws from the cask or serves the fresh-brewed default. The **Pink quartz** / **Dusty rose** hand grips the handle.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band with an **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, hand): `P R`. Neutrals (hair, outline, cask staves, ladle): `X B S V`. Gold (cask hoops, brew level, ladle, belt, buckle): `Z N G L`. Bog umber `U` (cask base shadow, boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. The **Old gold** brew level is the one bright accent — the value already in the cask, taken when present.

## Lighting

Top-left 45°. The **Brass leaf** spec on the cask's brew level is the brightest point on the frame-left; the **Bronze** ladle catches the light on the right. The cask's **Bog umber** base falls to shadow — the empty bottom that, when reached, sends the Cellarer to brew the lazy default rather than to error.
