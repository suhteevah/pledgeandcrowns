# The Quartermaster — visual spec

**REF-26**, 32×32, idle, transparent bg.
**Mission:** `slice_basic` — `fn sum_slice(xs: &[i32]) -> i32`.

> Status: **spec drafted 2026-06-18, JSX/PNG pending.** This is one of the
> four remaining `SPRITE_PLAYER` placeholders (Quartermaster, Auditor,
> Chronicler, Alchemist). The brief below is art *direction*; the JSX ref
> + rendered PNG are generated through `claude.ai/design` and approved by
> Matt per the locked art process (CLAUDE.md hard-rule #5).

## Curriculum cue

A slice `&[T]` is a borrowed *view* into a contiguous run of values — a
pointer plus a length. It owns nothing; the buffer lives elsewhere (a
`Vec`, an array). You read across it and hand back what you borrowed. The
Quartermaster is the supply officer who counts a row of stores he does
**not** own: he tallies a contiguous window of the rack, sums it, and
takes nothing off the shelf.

## Visual brief

A stout, square-shouldered quartermaster standing three-quarter-front,
filling the centre of the frame. Knee-length **Crypt** burgundy gambeson
with **Inkblood** shadow and **Wineflesh** mid-tones, a plain **Bronze**
buckle at a wide leather belt — practical, no sash, no sigil (he is staff,
not gentry).

Behind and beside him, spanning the lower-right third, a **Bog umber**
supply rack holding a row of five identical grain sacks in **Old gold**
with **Brass leaf** highlights and **Bronze** ties — the contiguous run.
The load-bearing metaphor: a slim **Antique brass** bracket/caliper frames
exactly three of the five sacks (a sub-run), the *borrowed window* — its
two arms are the pointer and the end, the gap between them the length.

His right hand holds a **Coalblack** notched tally rod (the length count),
raised to the framed sacks but not touching them — reading, not taking.
Left hand rests open at his side, empty: nothing is moved, nothing is
owned. A small **Aged paper** chit (the returned sum) is tucked under the
belt.

Head: broad, clean-shaven, **Stone grey** cropped hair with an **Aged
paper** highlight, weathered **Dusty rose** skin (`P R W`), a level
appraising gaze toward the bracketed sacks. A single **Antique brass**
collar stud, quartermaster's mark.

## Palette compliance

Burgundy (gambeson, skin): `K Y O W R P`. Gold (sacks, rack, bracket,
buckle, ties, collar stud): `U Z N G L`. Neutrals (hair, tally rod,
outline, chit): `X B S V` (`C` parchment cream optional for the chit).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal,
no forest.

## Lighting

Top-left 45° onto the rack: the three bracketed sacks catch `L` brass-leaf
highlight (the borrowed window is what's lit and counted); the two unframed
sacks fall a step toward `Z` bronze shadow. Brightest pixel is the caliper
arm tip on the lit sack — the slice's start pointer, where the count
begins.
