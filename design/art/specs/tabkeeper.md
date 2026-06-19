# The Tabkeeper — visual spec

**REF-46**, 32x32, idle, transparent bg.
**Mission:** `and_then` — chain fallible steps with `.and_then` (short-circuit).

## Curriculum cue

`.and_then` chains fallible operations: each step runs only if the previous one succeeded, and the first failure short-circuits the rest of the chain. The Tabkeeper embodies this — he holds a long tab strip of orders, one entry feeding into the next, every order required to succeed for the next to be served. The moment one order fails, the chain ends: the strip stops there. When the player writes `parse(x).and_then(validate).and_then(serve)`, they are running the Tabkeeper's strip — each link depends on the last, and one `Err` cuts the rest off.

## Visual brief

A clerk-figure holding a long tab. **Crypt** robe with **Oxblood** primary and **Wineflesh** lit folds — the house burgundy, a tab-keeper's cut, drawn slightly off-center so the tab can hang down the frame-right side. **Inkblood** seams the deep shadow down the sides and the widening hem.

From an **Antique brass** / **Old gold** clip at the top-right (with a single **Brass leaf** specular pixel), a long **Parchment cream** tab strip hangs down the frame-right edge. Down its length, a **chain of order-entries**: small **Coalblack** rule marks, one entry under the next — each link in the `.and_then` chain. Near the bottom the strip ends in a solid **Coalblack** bar: the chain *stops* there, the first failure that short-circuits the rest. His frame-right hand reaches up to grip the clip; the **Pink quartz** / **Dusty rose** hands brace the tab.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow and eye line, skin in **Pink quartz** / **Dusty rose**. **Antique brass** belt band with an **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (robe): `K Y O W`. Skin (face, hands): `P R`. Neutrals (hair, outline, tab rule marks): `X B S V`. Gold (tab clip, buckle): `N G L`. Parchment cream (tab strip): `C`. Bog umber `U` (boots), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white, no cobalt, no teal, no forest beyond the pine hem. Parchment cream is confined to the tab strip — the chain of orders.

## Lighting

Top-left 45°. The **Brass leaf** spec on the tab clip is the brightest point top-right; the cream tab strip runs bright down the right edge with the **Coalblack** rule marks reading as the chain's links. The solid **Coalblack** bar at the strip's foot is where the eye stops — the short-circuit, the failed link that ends the chain.
