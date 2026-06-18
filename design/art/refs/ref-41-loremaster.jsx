// SPDX-License-Identifier: MIT
// REF-41 - The Loremaster, 32x32, idle, transparent bg.
// A scholar holding an open tome whose Parchment-cream pages show a small
// inscription: a mapping, one input glyph bound to one output glyph
// (the Coalblack rule marks). Each trait has ONE associated type bound
// to it - `type Output` - the way each entry in the Loremaster's book
// names exactly one result. Crypt scholar's robe, Old-gold tome trim,
// no magic accent (lore, not spellcraft).
// Mission: assoc_type - an associated type (`type Output`) on a trait.
//
// Legend:
//   X coalblack (outline, page rule, inscription)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (tome clasp)   Z bronze (tome spine, boots)   G old gold (tome cover trim)
//   C parchment cream (tome pages)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF41_GRID = [
  '................................', //  0
  '..........XXXXXXXX..............', //  1 head top
  '.........XSSSVVSSSX.............', //  2 hair
  '........XSSVVVVVVSSX............', //  3
  '........XSPPPPPPPPSX............', //  4 forehead
  '........XPPPPPPPPRRX............', //  5
  '........XPBPPPPBPRWX............', //  6 brow/eyes
  '........XPPRRRRRRRPX............', //  7
  '.......XXPRRRRRRPXX.............', //  8 jaw
  '.......XYPPPRRRRPYX.............', //  9 collar
  '......XYYYYYYYYYYYYX............', // 10 shoulders
  '.....XYYYOOOOOOOOYYYX...........', // 11 robe upper
  '....XYYYOOOWWWWOOOOYYX..........', // 12 robe lit
  '....XYYOOOWWWWWWOOOYYX..........', // 13
  '...PXYOOOOOOOOOOOOOYXP..........', // 14 hands brace the tome edges (P)
  '..PRPYOOOWWWWWWOOOOYPRP.........', // 15 hands (P/R)
  '..PPPGGGGGGGZGGGGGGGPPP.........', // 16 tome cover trim (G) + spine (Z)
  '..XXGCCCCCCGZGCCCCCCGXX.........', // 17 open pages (C), gold border
  '....GCXXCCCGZGCXXCCCG...........', // 18 left page glyph | right page glyph
  '....GCCCCCCGZGCCXXCCG...........', // 19 page rules (X inscription)
  '....GCXXCCCGZGCCCCCCG...........', // 20 input glyph -> output glyph mapping
  '....GCCCCCCGNGCCCCCCG...........', // 21 clasp (N) at spine
  '....GGGGGGGGZGGGGGGGG...........', // 22 tome bottom trim
  '......XYYOOOOOOOOYYX............', // 23 robe lower under tome
  '......XYYOOOWWWWOOYYX...........', // 24
  '......XYFFFFFFFFFFYYX...........', // 25 hem
  '......XFFFFFXX.XFFFFFX..........', // 26 leg split
  '......XZZZZX...XZZZZX...........', // 27 boots
  '......XZUUZX...XZUUZX...........', // 28
  '......XXXXXX...XXXXXX...........', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF41 = function REF41({ scale = 8 }) {
  return <PixelArt grid={REF41_GRID} scale={scale} title="REF-41 The Loremaster" />;
};
window.REF41_GRID = REF41_GRID;
window.REF41_ROLES = ['X','Y','O','W','R','P','B','S','V','N','Z','G','C','U','F'];
