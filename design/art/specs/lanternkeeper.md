# The Lanternkeeper — visual spec

**REF-40**, 32x32, idle, transparent bg.
**Mission:** `lifetimes` — a borrow kept alive for `'a`.

## Curriculum cue

A lifetime `'a` is the span over which a borrow stays valid. The borrowed value must outlive the reference to it: `fn keep<'a>(&'a self) -> &'a Flame`. The Lanternkeeper holds the Lifetime Lantern aloft, a small steady flame burning inside it — and the flame must stay lit as long as the keeper carries it. The glass lantern is the borrow checker's guarantee: the flame (the borrowed value) lives exactly as long as the keeper holds the lantern. Let go too early and the borrow is dangling; the lantern keeps it honest.

## Visual brief

A figure offset toward frame-left, the right arm raised to hold a lantern aloft in the upper-right of the frame. The lantern hangs from an **Antique brass** ring on a short **Coalblack** chain. The body is an **Old gold** glass case, **Coalblack**-framed, with an **Antique brass** cap and base. Inside, a small steady **Mage glow** flame with a **Royal arcane** base shadow — contained, never escaping its glass, the one magic accent and well under the 5% cap. The raised hand (**Pink quartz** / **Dusty rose**) cups the handle.

The figure wears a **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds.

Head: short **Stone grey** hair with **Aged paper** highlight, **Basalt** brow and eyes, skin in **Pink quartz** / **Dusty rose**. An **Antique brass** belt band with **Old gold**, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `Y O W`. Skin: `P R`. Neutrals (hair, outline, lantern frame/chain): `X B S V`. Gold (lantern body/cap/base, belt): `N Z G`. Magic (contained flame only): `% *` — a few pixels glassed inside the lantern, well under the 5% cap. Bog umber `U` (boots), Pine `F` (hem).

No alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The robe's left folds catch **Wineflesh**. The lantern is the bright focal point in the upper right; the **Mage glow** flame is the brightest mass but it is fully enclosed in **Old gold** glass — the borrow kept alive, but contained to its lifetime, never spilling onto the rest of the figure.
