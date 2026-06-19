# The Riveter — visual spec

**REF-52**, 32x32, idle, transparent bg.
**Mission:** `iter_zip` — pair two iterators elementwise via `.zip`.

## Curriculum cue

`.zip` walks two iterators in step, yielding `(a, b)` pairs and stopping when either runs out — it takes one from each, side by side, and binds that pair, then the next. The Riveter embodies this: he holds two metal strips, one over the other, and drives rivets down the seam where they meet — rivet by rivet, the top strip's hole matched to the bottom strip's hole, pinned into one pair. He does not join all of one then all of the other. When the player writes `a.iter().zip(b.iter())`, they are doing the Riveter's move — pull one from each side, pin them together, advance, stop when a side runs dry.

## Visual brief

A figure standing, frame-right hand raising a rivet hammer mid-stroke. **Crypt** fitter tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, drawn slightly off-center so the two riveted strips run along the frame's lower edge. **Inkblood** seams the deep shadow.

In the raised frame-right hand, a **rivet hammer**: an **Old gold**-faced head with a **Brass leaf** specular pixel, on a **Bronze** haft, **Coalblack**-outlined — the tool driving each pair together. The **Pink quartz** / **Dusty rose** hand grips it.

Along the bottom of the frame, **two Bronze metal strips** run side by side, one over the other, edged in **Antique brass**, pinned along their seam by a line of **Old gold** rivets — the zipped pairs, one rivet binding each matched hole. The strips meet and stop together.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, hand): `P R`. Neutrals (hair, outline, hammer haft, strips): `X B S V`. Gold (hammer face, rivets, strip edges, belt): `Z N G L`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. The **Old gold** hammer face and the line of **Old gold** rivets are the bright accents — each rivet a zipped pair, two strips pinned into one.

## Lighting

Top-left 45°. The **Brass leaf** spec on the hammer face is the brightest point, top-center where the tool is raised; the **Old gold** rivets catch the light in a row along the seam below — the pairs the zip has bound, two iterators walked in step.
