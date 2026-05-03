# The Forgewright — visual spec

**REF-21**, 32×32, idle, transparent bg.
**Mission:** `borrow_mut` — exclusive borrow `&mut T`.

## Curriculum cue

The Forgewright is the village's specialist on the *exclusive* side of borrowing. Where the Smith (REF-11) is the introductory mutator — "name what changes" via `let mut` against your own binding — the Forgewright handles the rarer, stricter case: a single writer, holding tongs to a single piece of iron, while the Borrow Checker stands behind the curtain. Voice spec from the editor pass: "one writer at the forge". The visual contract is one set of tongs gripping one workpiece — never a hammer. The hammer belongs to the Smith; the tongs belong to the exclusive borrow.

## Visual brief

Wiry, focused human, smaller frame than the Smith but with the same village burgundy uniform — apron is **Oxblood** with **Wineflesh** mid and **Crypt** shadow, but cut shorter than the Smith's so you read them apart at a glance. No hammer. Instead the figure grips a long pair of **Basalt**/**Stone grey** iron tongs in front, both hands on the haft, jaw set. The tong-jaws clamp a small workpiece glowing with **Old gold** core and a single **Brass leaf** specular pixel — that pixel is the value being mutated through the deref `*x`. The forge fire itself is off-canvas (no scarlet on the sprite — alarm scarlet is reserved for boss and danger states); the glow on the workpiece comes only from the gold ramp.

Hair is short **Bronze** pulled back; **Antique brass** beard kept tight (one row deeper than the Smith's full beard). Skin is the standard human palette. Belt **Old gold** buckle. Boots **Bronze**/**Bog umber**. Pose: square front, weight balanced, both arms forward in a tight controlled hold — heraldic for "I have it, no one else does."

## Visual distinction from the Smith

- Smith: hammer at right hip (passive ready), full beard, square wide stance, head 9 px wide.
- Forgewright: tongs held in front (active grip), shorter beard, narrower stance, head 8 px wide, shorter apron.

Both wear the burgundy village uniform, which is the *family resemblance* — both are forge-related, both are about mutation, but the props and posture make the borrow-vs-let-mut distinction read in one glance.

## Palette compliance

Burgundy ramp (apron): `K Y O W R P`. Gold ramp (workpiece, buckle, beard): `Z N G L`. Neutrals (tongs, outline): `X B S`. Skin: `P R W`. Boots: `Z U`.

No magic violet, no alarm scarlet, no specular white, no cobalt.

## Lighting

Top-left 45°. The gold workpiece is the only bright on the sprite — single `L` pixel at its top-left edge — to pull the eye to the mutation point.
