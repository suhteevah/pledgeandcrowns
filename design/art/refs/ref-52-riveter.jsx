// SPDX-License-Identifier: MIT
// REF-52 - The Riveter, 32x32, idle, transparent bg.
// Holds two metal strips, one over the other, and drives Old-gold rivets
// down the seam where they meet - rivet by rivet, the top strip's hole
// matched to the bottom strip's hole, pinned into one pair. He does not
// join all of one then all of the other; he takes ONE from each, side by
// side, and binds that pair, then the next. That is `.zip`: walk two
// iterators in step, yielding `(a, b)` pairs, stopping when either runs
// out. Crypt fitter tunic, Bronze strips, Old-gold rivet hammer, no magic.
// Mission: iter_zip - pair two iterators elementwise via `.zip`.
//
// Legend:
//   X coalblack (outline, strips, hammer head)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (metal strips, hammer head, boots)   N antique brass (strip edge, belt)
//   G old gold (rivets)   L brass leaf (rivet/hammer spec)
//   U bog umber (boots)   F pine (hem)
//   . transparent
//
// Two Bronze strips run side by side, pinned by a line of Old-gold rivets
// (G) - the zipped pairs. The Old-gold-faced hammer with its L spec is the
// tool driving each pair together.

const REF52_GRID = [
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
  '........XYYYYYYYYYYYX...XGGGX...', // 10 shoulders | hammer head (G)
  '.......XYYOOOOOOOOYYX..XGLGGX...', // 11 robe upper | hammer (L spec)
  '......XYYYOOOWWWOOOYYX.XGGGGX...', // 12 robe lit | hammer head
  '.....XYYOOOWWWWWOOOYYX..XZZX....', // 13 | hammer haft (Z)
  '.....XYOOOOOOOOOOOOYXPP.XZX.....', // 14 hand reaches (P) | haft
  '.....XYOOOWWWWWOOOYXPRPXZX......', // 15 hand grips hammer (P)
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
  '..XNZZZZZZZZZZZZZZNX............', // 28 top strip (Z) brass edge (N)
  '..XNZGZZGZZGZZGZZNX.............', // 29 rivets (G) along seam
  '..XNZZZZZZZZZZZZZNX.............', // 30 bottom strip (Z)
  '..XXXXXXXXXXXXXXXX..............', // 31 strip base
];

window.REF52 = function REF52({ scale = 8 }) {
  return <PixelArt grid={REF52_GRID} scale={scale} title="REF-52 The Riveter" />;
};
window.REF52_GRID = REF52_GRID;
window.REF52_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
