# The Reckoner — visual spec

**REF-25**, 32×32, idle, transparent bg.
**Mission:** `closure_basic` — `let add = |a, b| a + b`.

## Curriculum cue

A closure is an anonymous function bound to a name with `let`. The Reckoner's voice spec from the editor pass: "inks the ledger, summed in a single stroke". The visual is a desk-clerk hunched over an open ledger, quill in the right hand, mid-stroke — a single pen-line that *is* the closure body, terse and self-contained. The bound name `add` is the ledger's running total; the `|a, b| a + b` is what happens between the columns and the sum.

## Visual brief

Seated-but-readable-as-standing accountant: the figure is bowed forward at the waist over a desk that fills the lower third of the frame. Long **Crypt** robe with **Inkblood** shadow and **Wineflesh** mid — clerical burgundy, the same family as the Sage but plainer (no sash, no sigils). **Aged paper** undercollar showing at the throat.

The desk is a **Bog umber** bench-top spanning rows 24–30 across the lower frame, with **Bronze** trim along the leading edge. On the desk lies an open ledger: two pages of **Parchment cream**, with three short **Coalblack** tally rows on the left page and a single **Coalblack** sum-line at the bottom of the right page — `a + b` and the result. A **Old gold** ribbon bookmark trails over the edge.

Right hand grips a **Coalblack** quill with a **Brass leaf** specular pixel on the nib — mid-stroke, hovering just above the right page. The pen-line is the closure body. Left hand rests flat on the left page, fingers spread, anchoring the binding.

Head bowed: short **Stone grey** hair with **Aged paper** highlight, the face foreshortened — only forehead, brow, and the bridge of the nose visible. No beard. A single **Antique brass** earring on the visible side, period-appropriate clerical detail.

## Palette compliance

Burgundy (robe): `K Y O W R P`. Gold (desk trim, ribbon, quill nib spec, earring, undercollar hi): `Z N G L`. Neutrals (hair, outline, ledger, quill): `X B S V C` (parchment cream `C` is in the gold ramp). Skin: `P R W`. Bog umber `U` for the desk.

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest.

## Lighting

Top-left 45° onto the open ledger — the right page (sum side) is in `C` cream highlight, the left page (input side) shades a row toward `V`. The single brightest pixel in the sprite is the **Brass leaf** quill-nib: the closure is being written *now*.
