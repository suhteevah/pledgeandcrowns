// REF-19 — The Tinker, 32×32, idle, transparent bg.
// Stocky village fix-it artisan; goggles up, mallet at thigh, belt of
// alternating brass/umber pouches (the loop body's repeated cells).
// Small gear on hip chain. Mission: while_loop.

// Legend:
//   X coalblack (outline)        B basalt (mallet head, goggle frame, brow)
//   S stone grey (goggle lenses, mallet head hi, hip chain)
//   Z bronze (hair, mallet haft, boot tops, pouch A)
//   N antique brass (hair mid, vest highlight)
//   G old gold (gear, vest accent)
//   U bog umber (pouch B, undershirt, boot shadow, trousers)
//   F pine (vest shadow)         E forest (vest primary)
//   H spring meadow (vest lit row)
//   R dusty rose (skin mid)      W wineflesh (skin shadow / mouth)
//   P pink quartz (skin hi)
//   . transparent

const REF19_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XZZZZZZZZX.............', //  3 hair shadow row
  '........XZNNNNNNNNZX............', //  4 hair mid
  '........XBSSXXXXSSBX............', //  5 goggles pushed up (lenses + frame)
  '........XBSSXXXXSSBX............', //  6 goggles row 2
  '........XPPPPPPRRRWX............', //  7 forehead under goggles
  '........X.PPRRRRRRWX............', //  8
  '........XX.X.XX.X.XX............', //  9 eyes
  '........X.PPRRRRRRWX............', // 10
  '........XXPPPRRRRWXX............', // 11 chin
  '.........XEEEEEEEEX.............', // 12 vest collar
  '........XEEEEEEEEEEX............', // 13 shoulders
  '.......XEHHEEEEEEFEEX...........', // 14 vest upper (left-edge highlight)
  '.......XEHEEEEEEFFFEX..XGGGX....', // 15 vest mid + gear hangs from chain
  '.......XEEEEEEEFFFEEX..XGGGX....', // 16
  '.......XEEEEEFFFFEEEX...XSX.....', // 17 chain links
  '.......XEEEFFFFFFEEEX...XSX.....', // 18
  '.......XZZZUUZZZUUZZX...........', // 19 belt: pouch A (Z) / pouch B (U) alternating
  '.......XZNZUNUZNZUNZX...........', // 20 belt highlight row
  '.......XZZZUUZZZUUZZX...........', // 21
  '......XBSSXFFEEEEFFXX...........', // 22 mallet head left of body, vest lower
  '......XBBSSXEEEEEEEX............', // 23 mallet head body
  '......XBBSSXUUUUUUUX............', // 24 undershirt under vest
  '.......XZZZXUUUUUUUX............', // 25 mallet haft + trousers
  '.......XZZZXUUUUUUUX............', // 26
  '.......XZZZXUUXXUUUX............', // 27 leg split start
  '.......XZZZXUUX.UUUX............', // 28
  '........XX.XZZZX.ZZZX...........', // 29 boot tops shifted right
  '...........XZUZX.ZUZX...........', // 30 boot mid
  '...........XXXXX.XXXX...........', // 31 soles
];

window.REF19 = function REF19({ scale = 8 }) {
  return <PixelArt grid={REF19_GRID} scale={scale} title="REF-19 The Tinker" />;
};
window.REF19_GRID = REF19_GRID;
window.REF19_ROLES = ['X','B','S','Z','N','G','U','F','E','H','R','W','P'];
