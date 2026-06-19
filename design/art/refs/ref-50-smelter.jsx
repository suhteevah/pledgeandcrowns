// SPDX-License-Identifier: MIT
// REF-50 - The Smelter, 32x32, idle, transparent bg.
// At a Bronze crucible he tips a stream of molten metal (Old-gold/amber
// glow, one warm Teal spark of heat-shimmer) down into a single ingot mold
// below. Many scraps went into the crucible; ONE ingot comes out. He
// carries an accumulator - the growing pool - and folds every piece into
// it, melt by melt, until all that's left is the single result. That is
// `.fold(init, |acc, x| ...)`: start with a seed, combine each item into
// it, end with one value. Crypt forge tunic, no magic, no scarlet.
// Mission: iter_fold - reduce many items to one via `.fold`.
//
// Legend:
//   X coalblack (outline, crucible, mold)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (crucible, mold, boots)   N antique brass (crucible band, belt)
//   G old gold (molten pour, ingot)   L brass leaf (molten hi)
//   T main teal (heat-shimmer accent)   U bog umber (mold shadow)   F pine (hem)
//   . transparent
//
// The molten ramp G->L is the pour and the gathered ingot - the one value
// the fold builds. A single Teal pixel is heat-shimmer over the crucible.

const REF50_GRID = [
  '................................', //  0
  '........XXXXXXXX................', //  1 head top
  '.......XSSSVVSSSX...............', //  2 hair
  '......XSSVVVVVVSSX..............', //  3
  '......XSPPPPPPPPSX..............', //  4 forehead
  '......XPPPPPPPPRRX..............', //  5
  '......XPBPPPPBPRWX..............', //  6 brow/eyes
  '......XPPRRRRRRRPX....T.........', //  7 jaw | heat shimmer (T)
  '.....XXPRRRRRRPXX...XZZX........', //  8 | crucible lip
  '.....XYPPPRRRRPYX..XZNNZX.......', //  9 collar | crucible band (N)
  '.....XYYYYYYYYYYYX.XZGGGZX......', // 10 shoulders | crucible molten (G)
  '....XYYOOOOOOOOYYX.XZGLGZX......', // 11 robe upper | molten hi (L)
  '...XYYYOOOWWWOOOYYX.XZGGZ.......', // 12 robe lit | crucible tips
  '..PXYYOOOWWWWWOOOYYXPXGG........', // 13 hand grips (P) | pour starts (G)
  '.PXYOOOOOOOOOOOOOOYXPP.GG.......', // 14 hand (P) | pour stream
  '.PXYOOOWWWWWWWOOOOYXP...GG......', // 15 | stream falls
  '..XXNGGGGGGGGGGNXX.......GL.....', // 16 belt (N) | stream
  '..XKYYOOOOOOOOYYKX......XGGX....', // 17 | mold lip (X)
  '.XKYYOOOWWWWWOOYYKX....XZGGZX...', // 18 | ingot mold (Z) molten in (G)
  '.XKKYOOOOOOOOOOYYKKX...XZGLGZX..', // 19 robe widens | ingot fills (L)
  'XKYYOOOWWWWWWWOOOYYKX..XZGGGGZX.', // 20 | full ingot
  'XKYYOOOOOOOOOOOOOYYKX..XNNNNNNX.', // 21 | mold base band (N)
  'XKYYOOOOOOOOOOOOOYYKX..XUUUUUUX.', // 22 | mold shadow (U)
  'XKKYYOOOOOOOOOOOOYYKKX.XXXXXXXX.', // 23 widest | mold bottom
  'XKYYFFFFFFFFFFFFFFFYKX..........', // 24 hem
  'XKFFFFFXXXX....XXXXFFFFKX.......', // 25 leg split
  'XXZZZZZXX........XXZZZZX........', // 26 boots
  '.XZUUUZX..........XZUUZX........', // 27
  '.XXXXXXX..........XXXXXX........', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF50 = function REF50({ scale = 8 }) {
  return <PixelArt grid={REF50_GRID} scale={scale} title="REF-50 The Smelter" />;
};
window.REF50_GRID = REF50_GRID;
window.REF50_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','T','U','F'];
