# The Lighthouse Keeper — visual spec

**REF-55**, 32x32, idle, transparent bg.
**Mission:** `arc_mutex` — share state across threads with `Arc<Mutex<T>>`.

## Curriculum cue

The coast's light is shared — every ship relies on it — but only one keeper may tend the lamp at a time. So the Keeper carries the single key to the lantern room and holds it up: to touch the light you must first take the key, and while you hold it no one else can. That is `Arc<Mutex<T>>`: many owners share the same lamp (the `Arc`), but the `Mutex` hands out exactly one key (the lock), so only one thread mutates the shared state at once. When the player writes `let data = Arc::new(Mutex::new(...))` and `data.lock()`, they are doing what the Keeper does — taking the one key before touching the shared light.

## Visual brief

A keeper standing at frame-left beside a small lighthouse that fills the right column of the frame. The house **Crypt**/**Oxblood** coat with **Wineflesh** lit folds; **Inkblood** for the deepest seams.

In his raised hand, the single key to the lantern room: an **Old gold** bow and shaft with **Coalblack** teeth and a **Brass leaf** specular pixel — *one* key, held up, the lock made visible. It is the one bright mass on the figure's body.

To frame-right, the lighthouse: a **Bronze** tower body banded with **Aged paper** courses and **Antique brass** seams, outlined in **Coalblack**, flaring to a wider **Bronze** base at the waterline. At the top, the lantern room — an **Antique brass** frame around an **Old gold** lamp core (**Main teal** glow inside) — casts a single **Bright teal** beam fanning out across the upper-right, fading through **Main teal** to **Deep teal** at its tail. The beam is the shared light every ship depends on.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow/eye line, skin in **Pink quartz**/**Dusty rose**. **Antique brass** belt with **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (coat): `K Y O W`. Skin: `P R`. Neutrals: `X B S V`. Gold (key, lamp core, tower, buckle): `Z N G L`. Teal (lamp glow, beam): `D T I`. Bog umber `U` (boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the one key pixel, no cobalt, no forest beyond the pine hem.

## Lighting

Top-left 45° on the figure; the lamp is its own emissive source at the tower top. The two brightest accents are the **Brass leaf** spec on the held key and the **Bright teal** lamp beam — the single lock in his hand, the single shared light it opens.
