# The Chronicler — visual spec

**REF-28**, 32×32, idle, transparent bg.
**Mission:** `derive_debug` — `#[derive(Debug)]`, print with `{:?}`.

> Status: **spec drafted 2026-06-18, JSX/PNG pending.** One of the four
> remaining `SPRITE_PLAYER` placeholders. Art *direction* only; JSX +
> rendered PNG go through `claude.ai/design` + Matt approval per the locked
> art process.

## Curriculum cue

`#[derive(Debug)]` *generates* the formatting impl mechanically from a
struct's fields — you don't write it by hand. The lesson is the
machine-made record: you annotate, and the representation is produced for
you. The Chronicler is the opposite of the Reckoner (who writes by hand,
mid-stroke): the Chronicler's record writes **itself**, driven by
clockwork, not a living hand.

## Visual brief

A tall, still robed historian standing left-of-centre, hands clasped behind
the back — emphatically *not* writing. Floor-length **Crypt** burgundy
scholar's robe, **Inkblood** shadow, **Wineflesh** mid, a plain **Bronze**
clasp at the throat over an **Aged paper** undercollar.

To his right, a lectern (**Bog umber** post, **Bronze** trim) bearing a
large open tome: two **Parchment cream** pages already filling with neat
**Coalblack** rows. The load-bearing metaphor — *derived, not authored*: a
small **Antique brass** clockwork armature rises from the lectern and holds
the quill, gear teeth (**Old gold** / **Brass leaf** specular) meshed at
its elbow. The quill is upright and writing **on its own**; no hand touches
it. A faint **Brass leaf** line of fresh ink trails behind the nib — the
auto-generated `{:?}` output appearing a field at a time.

On the open left page, three short **Coalblack** field-labels with a tiny
**Bronze** bracket around them = the struct's fields the derive reads from.
The Chronicler merely watches them become a record.

Head: long **Stone grey** hair to the collar with an **Aged paper**
highlight, lined **Dusty rose** skin (`P R W`), eyes on the self-writing
quill. A single **Antique brass** temple-piece (a clerk's eyeglass arm,
folded).

## Palette compliance

Burgundy (robe, skin): `K Y O W R P`. Gold (clockwork armature, gears,
quill specular, tome edge, undercollar hi, clasp): `U Z N G L C`. Neutrals
(hair, ink rows, lectern outline): `X B S V`.

No magic violet (the auto-writing is **clockwork**, not magic — keep it off
the Trait Mage's violet), no alarm scarlet, no specular white, no cobalt,
no teal, no forest.

## Lighting

Top-left 45° onto the open tome: the right page (the one the armature is
actively writing) catches `C` parchment-cream highlight; the left page
(already recorded) sits a step toward `V`. Brightest pixel is the **Brass
leaf** specular on the clockwork gear at the quill's elbow — the derive
*mechanism*, the thing doing the work the player never wrote.
