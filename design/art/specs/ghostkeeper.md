# The Ghostkeeper — visual spec

**REF-65**, 32x32, idle, transparent bg.
**Mission:** `weak_ref` — hold a non-owning `Weak` reference.

## Curriculum cue

`Weak<T>` is a non-owning reference: it points at a value but does not keep it alive, and you must `upgrade()` it to use it — a call that may return `None`, because the value the strong owners held may already be gone. The Ghostkeeper embodies this: a faded, half-there figure who holds a thin dashed tether out to a hoard that has gone faint, its outline broken, half-vanished. He can point at the hoard and ask whether it is still there, but he does not own it and cannot keep it alive — when the last real owner drops it, his tether goes slack and the hoard is simply gone. When the player writes `weak.upgrade()`, they are doing the Ghostkeeper's work — pulling on the tether to see whether the hoard is still there, or already `None`.

## Visual brief

A deliberately faded, translucent figure — the only ghostly member of the cast. Rendered almost entirely in low-contrast **Aged paper** and **Stone grey** with only sparing **Crypt** to hint at the house tunic; **no hard Coalblack outline**, so the silhouette reads as half-dissolved. Skin is faint **Pink quartz** / **Dusty rose** with a barely-there **Basalt** brow. One hand opens to frame-left.

From the figure's hand a thin **Cobalt** dashed tether runs out to frame-right — broken into dashes (`>` highlight, `<` shadow) so it reads as ghostly and non-binding. At the end of the tether, a hoard that has half-vanished: a broken **Cobalt deep** outline with gaps where walls should be, and inside it a single faded **Old gold** coin — the ghost of the value the tether points at. The cobalt pair is used precisely *because* it is not the warm burgundy/gold of the owning cast: it marks this figure and its link as cold, faint, non-owning.

Boots and form fade out in **Bronze** and **Aged paper** with no solid soles — the figure does not fully touch the ground.

## Palette compliance

Faded form: `S V` (low-contrast, dominant) with sparing `Y` (crypt) and faint `R P` skin, `B` brow. Cobalt counterweight (tether, vanished-hoard outline): `< >`. One faded **Old gold** `G` coin in the hoard. Sparse `X` outline only; faint `Z` boots.

No magic violet, no alarm scarlet, no specular white. The cobalt and the low-contrast V/S treatment are deliberate and on-brief: this figure must read as *not owning*, visually set apart from the solid burgundy-and-gold owning cast.

## Lighting

Flat and faint by design — the Ghostkeeper has no strong top-left key light because he is barely there. The one note of saturation is the **Cobalt** dashed tether, and the one warm pixel is the faded **Old gold** coin in the half-vanished hoard. The broken hoard outline and the gapped tether together say it plainly: this reference points, but does not own, and `upgrade()` may already return `None`.
