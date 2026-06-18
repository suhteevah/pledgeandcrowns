// SPDX-License-Identifier: MIT
// REF-30 - The Guildmaster, 32x32, idle, transparent bg.
// Tall authority figure in a Crypt robe of office. A gold guild medallion
// (seal) hangs on the chest; a gold-topped staff of office stands at his
// right. He is a member with duties - methods bound to the type via impl.
// Mission: impl_method - give the struct behavior with `impl`.
//
// Legend:
//   X coalblack (outline, staff shaft)
//   K inkblood (robe deepest shadow)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair, beard)   V aged paper (hair/beard hi)
//   N antique brass (medallion chain, staff collar)   Z bronze (staff shaft, boots)
//   G old gold (medallion disc, staff finial)   L brass leaf (medallion/finial spec)
//   U bog umber (boot shadow)   F pine (hem under-robe)
//   . transparent

const REF30_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSSVVSSSX.............', //  3 hair
  '........XSSVVVVVVSSX............', //  4
  '........XSPPPPPPPPSX............', //  5 forehead band
  '........XPPPPPPPPRRX............', //  6
  '........XPBPPPPBPRWX............', //  7 brow / eyes
  '........XPPRRRRRRRPX............', //  8
  '........XSPRRRRRRPSX............', //  9 beard begins at jaw
  '........XSSRRRRRRSSX............', // 10
  '........XVSSSSSSSSVX............', // 11 beard tip (V hi)
  '.......XYYYYYYYYYYYYX...........', // 12 shoulders
  '......XYYYOOOOOOOOYYYX..........', // 13 robe upper lit
  '......XYOOONLLNOOOOYOX..GG......', // 14 chain start + staff finial top (G)
  '......XYOOONGGNOOOOYOX.GLLG.....', // 15 medallion chain (N) + finial spec (L)
  '......XYOOOGLLGOOOOYOX.GLLG.....', // 16 medallion disc (G) w/ L spec
  '......XYOOOGGGGOOOOYOX..GG......', // 17 medallion lower
  '......XYOOONGGNOOOOYOX..NN......', // 18 staff collar (N)
  '......XYOOOOOOOOOOOYOX..ZZ......', // 19 staff shaft (Z)
  '......XXYYOOOOOOOOYYXX..XZX.....', // 20
  '......XYYOOOWWWWOOOYYX..XZX.....', // 21 robe mid lit
  '......XXXNGGGGGGGGNXXX..XZX.....', // 22 belt buckle (G)
  '......XYYOOOOOOOOOOYYX..XZX.....', // 23 robe lower
  '......XYYOOOWWWWOOOYYX..XZX.....', // 24
  '......XYYOOOOOOOOOOYYX..XZX.....', // 25
  '......XYYOOOOOOOOOOYYX..ZZZ.....', // 26 staff foot
  '......XYFFFFFFFFFFFFYX..........', // 27 hem
  '......XFFFFXX..XXFFFFX..........', // 28 leg split
  '......XZZZZX....XZZZZX..........', // 29 boots
  '......XZUUZX....XZUUZX..........', // 30
  '......XXXXXX....XXXXXX..........', // 31 soles
];

window.REF30 = function REF30({ scale = 8 }) {
  return <PixelArt grid={REF30_GRID} scale={scale} title="REF-30 The Guildmaster" />;
};
window.REF30_GRID = REF30_GRID;
window.REF30_ROLES = ['X','K','Y','O','W','R','P','B','S','V','N','Z','G','L','U','F'];
