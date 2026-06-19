# The Signaler — visual spec

**REF-56**, 32x32, idle, transparent bg.
**Mission:** `mpsc_channel` — pass values between threads over a channel.

## Curriculum cue

The Signaler stands on the seawall with two semaphore flags — one raised high, one lowered — spelling a message across the water to the next station down the line. He holds the sending end; far off, a receiver reads each flag as it arrives, in order, one at a time. He never carries the message over himself — he hands it to the line and the line carries it. That is an `mpsc` channel: the sender drops a value in, the receiver pulls it out the far end, values flowing one way down the wire. When the player writes `let (tx, rx) = mpsc::channel()` and `tx.send(v)`, they are doing what the Signaler does — committing a message to the line.

## Visual brief

A signalman, centered and standing, both arms out — one raised to frame-left, one lowered to frame-right — gripping two flag poles. The house **Crypt**/**Oxblood** coat with **Wineflesh** lit folds; **Inkblood** for the deepest seams.

Two semaphore flags, one per hand: **Coalblack** poles topped with an **Antique brass** finial (a **Brass leaf** specular pixel at the very tip), each flag a triangular field of **Main teal** with a **Deep teal** shadow edge and **Bright teal** lit edge — sea-signal teal, the channel ends. The raised flag (frame-left) reads as the message being *sent* down the line. The hands gripping the poles are **Pink quartz**/**Dusty rose**.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow/eye line, skin in **Pink quartz**/**Dusty rose**. **Antique brass** belt with **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (coat): `K Y O W`. Skin: `P R`. Neutrals: `X B S V`. Gold (finials, buckle): `N G L`. Teal (flags): `D T I`. Bog umber `U` (boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the two finial pixels, no cobalt, no forest beyond the pine hem.

## Lighting

Top-left 45°. The coat's left-facing folds catch **Wineflesh**; the right side falls to **Crypt**. The brightest accents are the **Bright teal** lit edges of the two flags — one channel, two ends, the message streaming frame-left toward the receiver down the line.
