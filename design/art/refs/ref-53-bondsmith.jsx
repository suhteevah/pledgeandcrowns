// SPDX-License-Identifier: MIT
// REF-53 - The Bondsmith, 32x32, idle, transparent bg.
// Holds a Bronze locket-vessel close against his chest and seals a glowing
// Old-gold item down INTO it with cupped hands. Once sealed, the item is
// his - it lives inside the vessel he carries, gone from the world outside.
// He does not borrow the thing; he TAKES it into himself, owns it, carries
// it away. That is a `move` closure: `move || { use(captured) }` pulls the
// captured value out of the surrounding scope and into the closure, which
// now owns it. Crypt keeper tunic, Old-gold sealed glow, no magic violet -
// the glow is warm gold, not arcane.
// Mission: closure_move - take ownership into a closure via `move`.
//
// Legend:
//   X coalblack (outline, vessel rim)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (vessel body, boots)   N antique brass (vessel band, belt)
//   G old gold (sealed glow, item)   L brass leaf (glow hi)
//   U bog umber (boots)   F pine (hem)
//   . transparent
//
// The Old-gold glow (G->L) sealed inside the Bronze vessel is the captured
// value, now owned by the closure that holds it. Cupped Pink-quartz hands
// close over it - ownership moved in, not lent out.

const REF53_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX............', //  8 jaw
  '........XYPPPRRRRPYX............', //  9 collar
  '........XYYYYYYYYYYYX...........', // 10 shoulders
  '.......XYYOOOOOOOOYYX...........', // 11 robe upper
  '......XYYYOOOWWWOOOYYX..........', // 12 robe lit
  '.....XYYOOOWWWWWOOOYYX..........', // 13
  '.....XYOOOOXNNNNXOOOOYX.........', // 14 vessel rim (X) brass band (N)
  '.....XYOOOXNGGGGNXOOOYX.........', // 15 vessel opens, glow in (G)
  '....PRPYOXNGLLGGNXYOPRPX........', // 16 cupped hands (P) | glow hi (L)
  '....PPRPXNGGLLGGNXPPRPX.........', // 17 hands close over glow
  '.....XPPXZNGGGGNZXPPX...........', // 18 vessel body (Z)
  '......XXXXZNNNNZXXXX............', // 19 vessel band (N) sealed
  '.....XXNGGGGGGGGNXX.............', // 20 belt (N)
  '....XKYYOOOOOOOOYYKX............', // 21 robe lower
  '....XKYYOOOWWWWOOYYKX...........', // 22
  '...XKKYOOOOOOOOOOYYKKX..........', // 23 widest
  '...XKYYFFFFFFFFFFFFYKX..........', // 24 hem
  '...XKFFFFFXXX..XXXFFFFKX........', // 25 leg split
  '...XXZZZZZXX....XXZZZZX.........', // 26 boots
  '....XZUUUZX......XZUUZX.........', // 27
  '....XXXXXXX......XXXXXX.........', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF53 = function REF53({ scale = 8 }) {
  return <PixelArt grid={REF53_GRID} scale={scale} title="REF-53 The Bondsmith" />;
};
window.REF53_GRID = REF53_GRID;
window.REF53_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
