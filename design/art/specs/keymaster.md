# The Keymaster — visual spec

**REF-48**, 32x32, idle, transparent bg.
**Mission:** `hashmap_basic` — insert and get values by key in a `HashMap`.

## Curriculum cue

A `HashMap<K, V>` parks each value in a slot named by its key, and lets you fetch it back in one move — you do not scan every entry, you carry the *key* and the key takes you straight to the cubby. The Keymaster embodies this: behind him stands a rack of labeled pigeonhole slots, each a keyed cubby; in his raised hand a ring of keys, one key per slot. When the player writes `map.insert(key, value)` and later `map.get(&key)`, they are doing what the Keymaster does — file each thing under its key, then walk straight back to it by key, never searching the whole rack.

## Visual brief

A figure standing before a rack of slots, key ring in his raised frame-right hand. **Crypt** keeper's tunic with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, drawn slightly off-center so the rack sits to his frame-left. **Inkblood** seams the deep shadow down the sides and the widening hem.

To the frame-left/back, a **rack of pigeonhole slots**: **Antique brass** frame bands stacked in rows, each slot a **Bronze** interior dropping to a **Bog umber** shadow at its mouth — the named cubbies. This is the keyed storage the map represents.

In the frame-right hand, a **ring of keys**: an **Old gold** ring with hanging key bows, **Coalblack**-outlined teeth, and a single **Brass leaf** specular pixel on the bow that catches the light — the key the player holds, the one bright gold mass on the figure's side. The **Pink quartz** / **Dusty rose** hand grips the ring.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (tunic): `K Y O W`. Skin (face, hand): `P R`. Neutrals (hair, outline): `X B S V`. Gold (key ring, keys, slot frames, slot interiors, belt): `Z N G L`. Bog umber `U` (slot/boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. The **Old gold** key ring is the one bright accent — the key that unlocks its named slot in a single lookup.

## Lighting

Top-left 45°. The **Brass leaf** spec on the key bow is the brightest point on the frame-right; the **Antique brass** rack frames catch the light on the left while the slot mouths fall to **Bog umber** shadow — the cubbies waiting for their keyed value.
