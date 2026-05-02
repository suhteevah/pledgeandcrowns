# The Bellringer — visual spec

**REF-14**, 32×32, idle, transparent bg.
**Mission:** `loop_break` — teaches `loop { ... break value; }`, the only Rust loop that returns.

## Curriculum cue

Bells repeat. They keep ringing until something says stop. That's the lesson: `loop` is the *unconditional repeat*; you exit it deliberately with `break value`, which is also how the loop yields a result. The Bellringer holds the rope of a bell that hangs above his head; the rope is the loop body, the bell is the threshold, the sound is the break.

## Visual brief

Wiry human male, simple village dress: a **Forest**-green tunic over **Pine** trousers (he belongs to Hearthstone Village's Act-1 forest/spring-meadow palette). Brown leather belt with **Old gold** buckle, **Bronze**/**Bog umber** boots. Sleeves rolled (he works with his hands).

Both arms reach up to grip a single thick rope that descends from above the canvas; the rope is **Antique brass** core with **Bronze** shadow, two visible knots at handhold height. Above the rope, in the top 8 pixels of the canvas, hangs the lower curve of a **Old gold** bell with **Bog umber** shadow inside its mouth — players see only the bottom of the bell, suggesting the rest extends beyond the sprite's frame. A faint **Brass leaf** highlight pixel on the bell's rim.

Skin uses the standard Pink quartz / Dusty rose / Wineflesh ramp. Hair short, **Bronze**/**Antique brass** (the gold ramp doubles as warm hair across the cast). A single **Mist teal** dot on each side of his head suggests the *ring* — these are decorative, not magic, just the visual idiom of "sound waves." Use no more than two pixels each side.

## Palette compliance

Burgundy (skin): `R W P`. Gold (bell, buckle, hair, rope): `U Z N G L`. Forest (tunic): `F E`. Neutrals (outline, trouser shadow, rope shadow): `X B S`. Teal (sound-wave hint): `M` (≤4 pixels total).

No magic violet, no scarlet, no specular white, no Cobalt.

## Lighting

Top-left 45°. The bell catches the highlight on its left rim. Tunic shadow falls to the right. The rope is uniform tone (it's vertical and doesn't have a clear lit side at this scale).
