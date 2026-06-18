// SPDX-License-Identifier: MIT
// REF-31 - The Recruiter, 32x32, idle, transparent bg.
// A Crypt-robed officer holds an open recruit-roll (parchment-cream scroll)
// in the left hand and a quill in the right, mid-stamp - forging a brand-new
// member from a name. The scroll IS the constructor output.
// Mission: assoc_new - `Self::new` returns a fresh instance.
//
// Legend:
//   X coalblack (outline, quill shaft, roll text lines)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi, scroll shadow)
//   C parchment cream (recruit-roll page)
//   N antique brass (collar stud, roll spindle)   Z bronze (roll rod)
//   G old gold (wax seal on roll)   L brass leaf (quill nib spec)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF31_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSSVVSSSX.............', //  3 hair
  '........XSSVVVVVVSSX............', //  4
  '........XSPPPPPPPPSX............', //  5 forehead
  '........XPPPPPPPPRRX............', //  6
  '........XPBPPPPBPRWX............', //  7 brow / eyes
  '........XPPRRRRRRRPX............', //  8
  '........XXPRRRRRRPXX............', //  9 jaw (clean-shaven)
  '........XNPPPRRRRPNX............', // 10 collar studs (N)
  '.......XYYYYYYYYYYYYX...........', // 11 shoulders
  '......XYYYOOOOOOOOYYYX..........', // 12 robe upper lit
  '......XYOOOWWWWOOOOYOX.....XLX..', // 13 quill tip (L spec) upper right
  '....ZCCCCCCCCCY.OOOYOX....XXLX..', // 14 roll top edge + spindle (Z) | quill (X)
  '...ZCXXXXXCCCCV.OOOYOX...XXX....', // 15 roll text line 1 | quill shaft (X)
  '...PCXXXCCCCCCC.OOOOOX..PXX.....', // 16 left hand grips roll | right hand + quill
  '...PRCCXXXXXCCC.OOOYOXXPRPX.....', // 17 roll text line 2 | right hand stamps
  '...PPCCCGGCCCCC.OOOYOXXPPPX.....', // 18 wax seal (G) on roll | hand
  '...ZCCCGGGGCCCCV.OOYOX..........', // 19 seal lower
  '...ZCXXXCCCCCCCY.OYOXX..........', // 20 roll text line 3
  '....ZCCCCCCCCCY.XXYXX...........', // 21 roll bottom edge
  '......XXXNGGGGGGGGNXXX..........', // 22 belt buckle (G)
  '......XYYOOOOOOOOOOYYX..........', // 23 robe lower
  '......XYYOOOWWWWOOOYYX..........', // 24
  '......XYYOOOOOOOOOOYYX..........', // 25
  '......XYYOOOOOOOOOOYYX..........', // 26
  '......XYFFFFFFFFFFFFYX..........', // 27 hem
  '......XFFFFXX..XXFFFFX..........', // 28 leg split
  '......XZZZZX....XZZZZX..........', // 29 boots
  '......XZUUZX....XZUUZX..........', // 30
  '......XXXXXX....XXXXXX..........', // 31 soles
];

window.REF31 = function REF31({ scale = 8 }) {
  return <PixelArt grid={REF31_GRID} scale={scale} title="REF-31 The Recruiter" />;
};
window.REF31_GRID = REF31_GRID;
window.REF31_ROLES = ['X','Y','O','W','R','P','B','S','V','C','N','Z','G','L','U','F'];
