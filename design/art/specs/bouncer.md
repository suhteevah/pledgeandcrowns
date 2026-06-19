# The Bouncer — visual spec

**REF-43**, 32x32, idle, transparent bg.
**Mission:** `custom_error` — define your own error `enum` of named variants.

## Curriculum cue

A bare `Result<T, String>` lumps every failure into one fuzzy string; a real program names its failures: `enum TavernError { Underage, NoCoin, Brawling, Banned }`. Each kind of trouble gets its *own* named variant, and a `match` can then handle each by name. The Bouncer embodies this — every sort of trouble that comes through the door has a name, and the name is written on his tally board. He is the figure who refuses to call everything just "an error": he enumerates.

## Visual brief

The widest, heaviest figure in the tavern cast. Broad-shouldered, thick-necked, **arms crossed** over the chest — the crossed forearms *are* the stance, so there is no prop in the hands. **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds, the most massive cut in the cast (no apron). **Inkblood** seams the deep shadow down both sides and across the heavy hem.

Head: short, flat **Stone grey** hair with **Aged paper** highlight, a heavy **Basalt** brow line and **Basalt** eyes, a broad **Pink quartz** / **Dusty rose** face and jaw. The crossed forearms cross the chest in **Pink quartz** / **Dusty rose** skin tones over the burgundy.

To his frame-right, a small **Old gold** tally board on an **Antique brass** frame, with **Coalblack** rule marks down its face — each mark one named variant of the error enum, the roster of every kind of trouble. A single **Brass leaf** specular pixel sits at the board's top corner. This is the one bright gold mass on the sprite: the enumerated list of named failures.

**Antique brass** belt band with an **Old gold** buckle at the waist, **Pine** hem, **Bronze**/**Bog umber** boots on thick legs.

## Palette compliance

Burgundy (robe): `K Y O W`. Skin (face, crossed arms): `P R`. Neutrals (hair, brow, outline): `X B S V`. Gold (tally board, belt, buckle): `N G L`. Bog umber `U` (boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem.

## Lighting

Top-left 45°. The crossed forearms catch the light across the chest; the **Wineflesh** lit fold runs under the arms. The brightest accent is the **Brass leaf** spec on the tally board — authority and enumeration signaled in gold, off to the side where the roster of named troubles hangs.
