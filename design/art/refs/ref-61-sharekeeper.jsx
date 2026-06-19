// SPDX-License-Identifier: MIT
// REF-61 - The Sharekeeper, 32x32, idle, transparent bg.
// Holds one Old-gold coin-hoard chest while several hands reach in from
// the sides, each laying a tally-token claim on the SAME hoard - and a
// small count marker beside it ticks the number of claims. No hand owns
// the hoard alone; each holds a share, and the hoard lives until the last
// share is dropped. That is `Rc::new` + `Rc::clone`: shared ownership by
// reference count, the value kept alive as long as one owner remains.
// Crypt keeper tunic, Old-gold shared hoard and count marker, no magic.
// Mission: rc_basic - share ownership of a value with Rc.
//
// Legend:
//   X coalblack (outline, chest seam, count digit)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid, reaching hands)   P pink quartz (skin hi, hands)
//   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (hoard chest body, boots)   N antique brass (chest band, belt)
//   G old gold (coins, count marker)   L brass leaf (coin spec)
//   U bog umber (boots, chest base)   F pine (hem)
//   . transparent
//
// The single Old-gold hoard (Z chest, G coins, L spec) with many Pink-quartz
// hands reaching in from both sides is the shared value; the small G count
// marker (count tile, X digit) is the reference count - drop to zero and the
// hoard is freed.

const REF61_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX............', //  8 jaw
  '........XYPPPRRRPYYX............', //  9 collar
  '.......XYYYYYYYYYYYYX...........', // 10 shoulders
  '......XYYOOOOOOOOOYYYX..........', // 11 tunic upper
  '.....XYYOOOWWWWWOOOYYYX.........', // 12 tunic lit
  '....XYYOOOOOOOOOOOOOYYX.GXGX....', // 13 count marker (G tiles, X digit)
  'PRP.XYOOOOOOOOOOOOOOOYX.GGGG....', // 14 reaching hand left | count tile
  'PPRPXNGGGGGGGGGGGGGGNXPRP.......', // 15 chest band (N), hand right (P)
  'PPRPXGGLLGGGGGGLLGGGGXPPRP......', // 16 coins (G) spec (L) | hand right
  '.PPXGGGGGGGGGGGGGGGGGGXPP.......', // 17 hoard coins
  '....XZGGGGGGGGGGGGGGZX..........', // 18 chest body top (Z)
  'PRP.XZZGGGGGGGGGGGGZZX.PRP......', // 19 more hands reach | chest body
  'PPRPXZZZZGGGGGGGGZZZZX.PPRP.....', // 20 coins settle into chest
  '.PPXNNNNNNNNNNNNNNNNNNX.PP......', // 21 lower band (N)
  '....XUUUUUUUUUUUUUUUUUX.........', // 22 chest base (U)
  '...XKYYOOOOOOOOOOOOYYKX.........', // 23 tunic lower behind chest
  '...XKKYOOOWWWWWWOOYYKKX.........', // 24 widest
  '...XKYYFFFFFFFFFFFFYYKX.........', // 25 hem
  '...XKFFFFFXXX..XXXFFFFKX........', // 26 leg split
  '...XXZZZZZXX....XXZZZZX.........', // 27 boots
  '....XZUUUZX......XZUUZX.........', // 28
  '....XXXXXXX......XXXXXX.........', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF61 = function REF61({ scale = 8 }) {
  return <PixelArt grid={REF61_GRID} scale={scale} title="REF-61 The Sharekeeper" />;
};
window.REF61_GRID = REF61_GRID;
window.REF61_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
