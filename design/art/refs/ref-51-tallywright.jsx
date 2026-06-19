// SPDX-License-Identifier: MIT
// REF-51 - The Tallywright, 32x32, idle, transparent bg.
// A row of items runs along a low Bronze rail before him (frame-low). In
// his hand a counter-stamp; as each item passes he brings the stamp down
// and prints its NUMBER on it - 0, 1, 2 - so each carries both its place
// and itself. He does not just hand you the items, he hands you each item
// WITH its index. That is `.enumerate()`: it pairs every element with its
// running count, yielding `(i, item)`. Crypt clerk tunic, Old-gold stamp
// head and numbered tags, Antique-brass rail, no magic.
// Mission: iter_enumerate - pair each item with its index via `.enumerate`.
//
// Legend:
//   X coalblack (outline, stamp, rail edge, tag digits)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (rail, stamp shaft, boots)   N antique brass (rail caps, belt)
//   G old gold (stamp head, numbered tags)   L brass leaf (stamp spec)
//   U bog umber (rail shadow)   F pine (hem)
//   . transparent
//
// Three Old-gold tags on the rail carry stamped X-digit marks (0,1,2) -
// the indices. The Old-gold stamp head with its L spec is the numbering
// tool the enumerate adapter wields.

const REF51_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX............', //  8 jaw
  '........XYPPPRRRRPYX....GG......', //  9 collar | stamp head top (G)
  '........XYYYYYYYYYYYX..XGLGX....', // 10 shoulders | stamp head (L spec)
  '.......XYYOOOOOOOOYYX..XGGGGX...', // 11 robe upper | stamp head
  '......XYYYOOOWWWOOOYYX..XZZX....', // 12 robe lit | stamp shaft (Z)
  '.....XYYOOOWWWWWOOOYYX..XZZX....', // 13 | shaft
  '.....XYOOOOOOOOOOOOYXPP.XZX.....', // 14 hand reaches (P) | shaft
  '.....XYOOOWWWWWOOOYXPRPXZX......', // 15 hand grips stamp (P)
  '.....XXNGGGGGGGGNXXPPP.XX.......', // 16 belt (N) | hand
  '.....XKYYOOOOOOYYKX.............', // 17
  '....XKYYOOOWWWWOOYYKX...........', // 18
  '....XKKYOOOOOOOOOYYKKX..........', // 19 robe widens
  '...XKYYOOOWWWWWOOOYYKX..........', // 20
  '...XKYYOOOOOOOOOOOYYKX..........', // 21
  '...XKKYYOOOOOOOOOYYKKX..........', // 22 widest
  '...XKYYFFFFFFFFFFFYKX...........', // 23 hem
  '...XKFFFFFXXX.XXFFFFKX..........', // 24 leg split
  '...XXZZZZXX...XXZZZZX...........', // 25 boots
  '....XZUUZX.....XZUUZX...........', // 26
  '....XXXXXX.....XXXXXX...........', // 27 soles
  '..XNGGXGGXGGXNX.................', // 28 rail w/ tags (G), digits (X)
  '..XNGXGGGXGGGXNX................', // 29 stamped numbers row
  '..XZUUUUUUUUUUZX................', // 30 rail body (Z) shadow (U)
  '..XXXXXXXXXXXXXX................', // 31 rail base
];

window.REF51 = function REF51({ scale = 8 }) {
  return <PixelArt grid={REF51_GRID} scale={scale} title="REF-51 The Tallywright" />;
};
window.REF51_GRID = REF51_GRID;
window.REF51_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
