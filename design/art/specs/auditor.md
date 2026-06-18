# The Auditor — visual spec

**REF-27**, 32×32, idle, transparent bg.
**Mission:** `result_question_mark` — parse + propagate the error with `?`.

> Status: **spec drafted 2026-06-18, JSX/PNG pending.** One of the four
> remaining `SPRITE_PLAYER` placeholders. Art *direction* only; JSX +
> rendered PNG go through `claude.ai/design` + Matt approval per the locked
> art process.

## Curriculum cue

`Result<T, E>` is `Ok` or `Err`. The `?` operator unwraps the `Ok` and
keeps going, or short-circuits the whole function the instant it meets an
`Err`. The Auditor embodies the short-circuit: she runs her eye down the
column and stops dead at the **first** discrepancy — no further rows are
read, the error is handed straight back up the chain.

## Visual brief

A severe, upright auditor, narrow-shouldered, standing front-on, still as a
plumb line. Floor-length **Crypt** burgundy robe, **Inkblood** shadow,
**Oxblood** mid, buttoned to a high **Aged paper** collar — austere,
joyless, exact.

In her left hand, held chest-high, an open audit ledger: two **Parchment
cream** pages with a column of short **Coalblack** tally rows. Her right
hand holds a brass-rimmed loupe (**Old gold** ring, **Brass leaf**
specular on the glass) to one eye. The eye and the loupe are aimed at one
specific row partway down — the first failure.

The load-bearing metaphor — the propagated `Err`: a single small **Alarm
scarlet** wax seal / flag pinned on that exact row, the only warm-danger
pixel in the sprite, and everything below it on the page left blank
(unread — the function already returned). This is a deliberate, documented
bible exception: ~0.6% of canvas in **Alarm scarlet** `!`, under the 1%
alarm cap, justified the way the Heraldic Sage's cobalt sigil is — the
Auditor is the one NPC whose entire lesson *is* the error, so the error
gets the realm's one alarm color. Cross-ref the Sage spec.

Head: hair scraped back into a tight **Stone grey** bun, sharp **Wineflesh**
skin (`P R W`), thin pressed mouth. An **Antique brass** chain holds the
loupe.

## Palette compliance

Burgundy (robe, skin): `K Y O W R P`. Gold (loupe ring + specular, collar
hi, chain, ledger edge): `N G L C`. Neutrals (hair, tally marks, outline):
`X B S V`. **Alarm:** `!` Alarm scarlet — the flagged row's seal ONLY,
≤1% canvas (documented exception).

No magic violet, no specular white, no cobalt, no teal, no forest.

## Lighting

Top-left 45°. The lit half of the open ledger is the **rows above** the
flag (the `Ok` values already read, in `C` cream); the rows below the
**Alarm scarlet** seal sit in `B` basalt shadow — unreached, because `?`
bailed. Brightest non-alarm pixel is the **Brass leaf** loupe specular: the
instant of inspection where the `Err` is caught.
