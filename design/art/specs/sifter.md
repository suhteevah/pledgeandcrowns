# The Sifter — visual spec

**REF-49**, 32x32, idle, transparent bg.
**Mission:** `iter_filter` — keep only items that pass a predicate via `.filter`.

## Curriculum cue

`.filter(|x| pred(x))` runs the whole stream past a single test and keeps only what passes; the rest fall away. The Sifter embodies this: he holds a round sieve out before him and shakes grain through it — the fine grain that meets the test falls through the mesh and is kept, the chaff that fails stays on top and is let go. He does not stop the stream, he passes all of it through one screen. When the player writes `iter.filter(|x| x.is_good())`, they are doing the Sifter's move — let everything stream past the predicate, keep what falls through, drop the rest.

## Visual brief

A figure standing, both hands raising a round sieve at chest height. **Crypt** miller's tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, centered, **Inkblood** seaming the deep shadow.

In both **Pink quartz** / **Dusty rose** hands, a **round sieve**: a **Bronze** hoop, an **Antique brass** mesh frame top and bottom, and a row of mesh wires across the middle tinted with a touch of **Forest** (the screen the grain is tested against). The sieve is the predicate.

Below the sieve, a scatter of **Old gold** grains falling through the mesh — the kept items — with a single **Brass leaf** specular grain catching the light. These are the elements that passed the test.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Inkblood** belt seam under the sieve, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, hands): `P R`. Neutrals (hair, outline): `X B S V`. Gold (sieve hoop, mesh frame, falling grain): `Z N G L`. Forest `E` (mesh-wire tint only — a few pixels). Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal. Forest appears only as the mesh tint. The **Old gold** grain falling through is the bright accent — the kept value.

## Lighting

Top-left 45°. The **Brass leaf** spec sits on one falling grain; the **Bronze** sieve hoop catches the light along its upper-left arc. The grain below the mesh is what passed the predicate — the **Forest**-tinted wires are the test it had to clear.
