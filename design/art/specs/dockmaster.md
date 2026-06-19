# The Dockmaster — visual spec

**REF-54**, 32x32, idle, transparent bg.
**Mission:** `thread_spawn` — launch a worker that runs concurrently via `thread::spawn`.

## Curriculum cue

The Dockmaster never hauls the cargo himself. He stands at the head of the pier, points down the quay, and blows his whistle — and a crew peels off to work its own berth while he keeps dispatching the next. Each crew labors at the same time, on its own task, the moment he sends it. That is `thread::spawn`: you call it, a new thread of work peels off and runs concurrently, and you carry on. When the player writes `thread::spawn(move || { ... })`, they are doing what the Dockmaster does — pointing, whistling, and letting a worker run.

## Visual brief

A coastal dispatcher, centered and standing, mid-gesture. The house **Crypt**/**Oxblood** coat with **Wineflesh** lit folds — burgundy uniform, but cut as a working harbor coat rather than the inland guild's formal robe. **Inkblood** for the deepest seams.

Across the shoulders, a **Main teal** harbor sash with **Deep teal** shadow and **Bright teal** highlight — the coastal house mark that sets the Concurrent Coast cast apart from the inland guild. This is the one teal accent on his body and it should read as a sash band, not a robe.

In his right hand, raised to his lips, a dispatcher's whistle: a **Bronze** body with an **Antique brass** band and an **Old gold** bell, a single **Brass leaf** specular pixel on the bell. His left arm is flung out to frame-left, **Pink quartz**/**Dusty rose** hand pointing down the pier — the spawn gesture, a worker launched. At his right hip a clipboard manifest: **Parchment cream** pages with **Aged paper** ruled lines under an **Old gold** clip.

Head: short **Stone grey** hair with **Aged paper** highlights, **Basalt** brow/eye line, skin in **Pink quartz**/**Dusty rose**. **Antique brass** belt with an **Old gold** buckle, **Pine** hem, **Bronze**/**Bog umber** boots.

## Palette compliance

Burgundy (coat): `K Y O W`. Skin: `P R`. Neutrals (hair, outline): `X B S V`. Gold (whistle, clip, buckle): `Z N G L`. Teal (sash only): `D T I`. Parchment `C` (manifest), Bog umber `U` (boot shadow), Pine `F` (hem).

No magic violet, no alarm scarlet, no specular white beyond the one whistle-bell pixel, no cobalt, no forest beyond the pine hem.

## Lighting

Top-left 45°. The coat's left-facing folds catch **Wineflesh**; the right side falls to **Crypt**. The two brightest accents are the **Brass leaf** spec on the whistle bell and the **Bright teal** highlight on the sash — authority signaled by the whistle at the lips, coast signaled by the teal.
