# The Harbormaster — visual spec

**REF-58**, 32x32, idle, transparent bg.
**Mission:** `thread_scope` — spawn scoped threads that all join before the scope ends.

## Curriculum cue

The Harbormaster is the authority of the whole harbor. He rings the great bell to *open* the harbor, sends every boat out to its work — and will not ring it *closed* until he has counted every last hull back in. The harbor does not shut while a boat is still out. That is `thread::scope`: a bounded region that spawns its workers and guarantees they all finish and rejoin before the scope ends — no thread outlives the harbor's open hours, and borrowed data stays valid because nothing escapes the scope. When the player writes `thread::scope(|s| { s.spawn(...); })`, they are doing what the Harbormaster does — opening a bounded region that cannot close until every boat is home.

## Visual brief

A tall, senior authority figure, centered and standing — the most formal of the coastal cast. The house **Crypt**/**Oxblood** coat of office with **Wineflesh** lit folds; **Inkblood** for the deepest seams. Across the chest, a **Main teal** harbor sash (**Deep teal** shadow, **Bright teal** highlight) — the mark of harbor authority.

At frame-right, hung at his side, a great harbor bell: an **Old gold** body with an **Antique brass** rim and mouth, a **Bronze** yoke above, a **Coalblack** clapper, and a single **Brass leaf** specular highlight on the bell's upper curve. The bell is the scope's bound — rung to open, rung to close, never closed early. In his left hand, angled across the body, a **Bronze** spyglass (outlined in **Coalblack**, **Antique brass** trim) — counting the boats home.

Head: short **Stone grey** hair and a trimmed **Stone grey** beard with **Aged paper** highlights — older, senior. **Basalt** brow/eye line, skin in **Pink quartz**/**Dusty rose**. **Antique brass** belt with **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (coat): `K Y O W`. Skin: `P R`. Neutrals (hair, beard, outline): `X B S V`. Gold (bell, spyglass, buckle): `Z N G L`. Teal (sash): `D T I`. Bog umber `U` (boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the one bell pixel, no cobalt, no forest beyond the pine hem.

## Lighting

Top-left 45°. The coat's left-facing folds catch **Wineflesh**; the right side falls to **Crypt**. The two brightest accents are the **Brass leaf** spec on the harbor bell and the **Bright teal** highlight on the sash — authority in gold, harbor in teal, the bell that bounds the day.
