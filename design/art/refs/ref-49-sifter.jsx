// SPDX-License-Identifier: MIT
// REF-49 - The Sifter, 32x32, idle, transparent bg.
// Holds a round sieve/screen out before him, shaking grain through it. The
// fine grain that meets the test falls through the Antique-brass mesh and
// is kept; the chaff that fails stays on top and is let go. He does not
// stop the stream - he passes the whole stream through ONE test and keeps
// only what passes. That is `.filter(|x| pred(x))`: a predicate decides,
// the rest fall away. Crypt miller's tunic, Bronze sieve hoop, a few
// Old-gold grains falling, small Forest tint on the mesh, no magic.
// Mission: iter_filter - keep items that pass a predicate via `.filter`.
//
// Legend:
//   X coalblack (outline, sieve rim, mesh wires)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (sieve hoop)   N antique brass (mesh frame, belt)
//   G old gold (grain that passes)   L brass leaf (grain spec)
//   E forest (mesh tint)   U bog umber (boots)   F pine (hem)
//   . transparent
//
// Old-gold grains (G) fall THROUGH the mesh below the sieve - the kept
// items. The Bronze hoop with Forest-tinted mesh is the predicate screen.

const REF49_GRID = [
  '................................', //  0
  '.............XXXXXXXX...........', //  1 head top
  '............XSSSVVSSSX..........', //  2 hair
  '...........XSSVVVVVVSSX.........', //  3
  '...........XSPPPPPPPPSX.........', //  4 forehead
  '...........XPPPPPPPPRRX.........', //  5
  '...........XPBPPPPBPRWX.........', //  6 brow/eyes
  '...........XPPRRRRRRRPX.........', //  7
  '..........XXPRRRRRRPXX..........', //  8 jaw
  '..........XYPPPRRRRPYX..........', //  9 collar
  '..........XYYYYYYYYYYYX.........', // 10 shoulders
  '.........XYYOOOOOOOOYYX.........', // 11 robe upper
  '........XYYYOOOWWWOOOYYX........', // 12 robe lit
  '.....PPXYYOOOWWWWWOOOYYXPP......', // 13 both hands raise sieve (P)
  '....PXZXYOOOOOOOOOOOOYXPXZP.....', // 14 sieve rim (Z) | hands grip
  '....XZNNNNNNNNNNNNNNNNNNNZX.....', // 15 sieve hoop + mesh frame (N)
  '....XZNENENENENENENENENENZX.....', // 16 mesh wires (E tint)
  '....XZNNNNNNNNNNNNNNNNNNNZX.....', // 17 mesh frame
  '.....XXKYYOOOOOOOOOOYYKXX.......', // 18 belt under sieve
  '......XKYYOOOWWWWWOOYYKX........', // 19
  '......G.XKYYOOOOOOYYKX.G........', // 20 grain falls through (G)
  '.....G.XKKYOOOWWWOOYKKX.G.......', // 21
  '....G.XKYYOOOOOOOOOOYYKX.L......', // 22 grain (G) + spec (L)
  '...G.XKYYOOOOOOOOOOOOYYKX.G.....', // 23 widest
  '....XKYYFFFFFFFFFFFFFFFYKX......', // 24 hem
  '....XKFFFFFXXX...XXXFFFFFKX.....', // 25 leg split
  '....XXZZZZZXX......XXZZZZX......', // 26 boots
  '.....XZUUUZX........XZUUZX......', // 27
  '.....XXXXXXX........XXXXXX......', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF49 = function REF49({ scale = 8 }) {
  return <PixelArt grid={REF49_GRID} scale={scale} title="REF-49 The Sifter" />;
};
window.REF49_GRID = REF49_GRID;
window.REF49_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','E','U','F'];
