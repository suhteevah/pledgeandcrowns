# The Surveyor — visual spec

**REF-34**, 32x32, idle, transparent bg.
**Mission:** `tuple_struct` — `struct Meters(f64)`, a named scalar.

## Curriculum cue

A tuple struct wraps a bare value in a name: `struct Meters(f64)` is just a number, but the type now says what the number *means*. The Surveyor holds a graduated measuring rod — a plain length of staff that would be meaningless except that every tick names a unit. The rod is the `f64`; the gold graduation marks are the `Meters` wrapper. You cannot add `Meters` to `Feet` by accident, and you cannot read the rod without knowing its unit.

## Visual brief

A standing surveyor. **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds, **Antique brass** collar studs — field-officer burgundy, neat.

In the right hand, gripped at chest height and planted to the floor, a tall graduated measuring rod that runs nearly the full height of the frame: a **Bronze** shaft (**Coalblack**-outlined) ticked at regular intervals with **Old gold** graduation bands, the topmost tick taking a single **Brass leaf** specular pixel. An **Antique brass** foot caps the rod at the ground. The hand wraps the shaft in **Pink quartz** / **Dusty rose**, with a **Bronze** pixel where fingers cross the rod. Left hand rests on the hip.

Head: short **Stone grey** hair with **Aged paper** highlight, clean-shaven, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**. Standard **Old gold** belt buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `Y O W`. Skin: `P R`. Neutrals (hair, outline, rod core): `X B S V`. Gold (collar stud, rod shaft, graduation ticks, top-tick spec, rod foot, buckle): `N Z G L`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45° along the rod. The graduation ticks read brightest in **Old gold** against the darker **Bronze** shaft — the named units stand out from the raw length. The single **Brass leaf** specular sits on the topmost tick: the unit label that gives the number meaning.
