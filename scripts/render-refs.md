# Reference-art PNG export

The 10 reference images for the bible v2.0 milestone live in `design/art/refs/*.jsx`
as palette-mapped ASCII grids (REF-01..REF-08) or procedural canvas-draw functions
(REF-06, REF-09, REF-10). Per `design/04b-art-deliverables.md` §2, the contractor
deliverable contract is **PNG-32 single-frame**, which means we need a JSX→PNG step.

## Why this is a two-step manual flow

Babel-standalone's external-`src` JSX loader doesn't work over `file://` (Chrome
treats sibling `.jsx` files as cross-origin under the file: scheme). The
self-contained `render-refs-inline.html` harness sidesteps that by inlining all
JSX source into one HTML doc.

CI does NOT run this. The PNGs in `design/art/refs/png/` are committed artifacts;
the harness only needs to run when a `.jsx` ref source changes.

## Workflow

1. Open `scripts/render-refs-inline.html` in Chrome (or any modern browser).
2. Wait for the top-right status badge to turn green and read `done: 10/10`.
3. Open devtools, run in the console:
   ```js
   copy(JSON.stringify(Object.fromEntries(
     Object.entries(window.__refs).map(([k, v]) => [k, v.dataUrl || null])
   )))
   ```
4. Paste clipboard into `scripts/.refs-snapshot.json` (gitignored).
5. Run `python scripts/render-refs.py`.
6. Commit the changed PNGs in `design/art/refs/png/`.

## Native pixel sizes (matching bible v2.0 §"Reference image set")

| Ref | Subject              | Native px | File             |
|-----|----------------------|-----------|------------------|
| 01  | Player default       | 32×32     | REF01.png        |
| 02  | Ferris guide         | 32×32     | REF02.png        |
| 03  | Borrow Checker NPC   | 64×64     | REF03.png        |
| 04  | Village tile sheet   | 64×64     | REF04.png        |
| 05  | Tower interior tiles | 64×64     | REF05.png        |
| 06  | Code editor frame    | 640×360   | REF06.png        |
| 07  | Goblin enemy         | 32×32     | REF07.png        |
| 08  | Healing potion icon  | 16×16     | REF08.png        |
| 09  | World-map cinematic  | 320×180   | REF09.png        |
| 10  | Title screen         | 640×360   | REF10.png        |
