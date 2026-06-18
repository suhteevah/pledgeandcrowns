// SPDX-License-Identifier: MIT
// REF-37 - The Wandwright, 32x32, idle, transparent bg.
// A craftsman who turns out wands that will work for ANY element: the
// blank he holds is plain (no element bound yet), gold-fitted at the
// collar. A small work-bench with a tool fills the frame-left lower
// corner. The bounds are the gold fittings - the blank fits any type
// that satisfies them. Crypt apron-tunic, no magic accent (an unbound
// wand carries no element).
// Mission: generic_fn - a function generic over T with trait bounds.
//
// Legend:
//   X coalblack (outline, wand core, bench legs)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair, wand blank)   V aged paper (hair hi, blank hi)
//   N antique brass (wand fittings, bench tool head)
//   Z bronze (wand collar, bench top, boots)   G old gold (fitting spec)
//   U bog umber (bench shadow, boots)   F pine (hem)
//   . transparent

const REF37_GRID = [
  '................................', //  0
  '................XXXXXXXX........', //  1 head top
  '...............XSSSVVSSSX.......', //  2 hair
  '..............XSSVVVVVVSSX......', //  3
  '..............XSPPPPPPPPSX......', //  4 forehead
  '..............XPPPPPPPPRRX......', //  5
  '..............XPBPPPPBPRWX......', //  6 brow/eyes
  '..............XPPRRRRRRRPX......', //  7
  '.............XXPRRRRRRPXX.......', //  8 jaw
  '............XYPPPRRRRPYX........', //  9 collar
  '...........XYYYYYYYYYYYYX.......', // 10 shoulders
  '....XVX....XYYYOOOOOOOOYYYX.....', // 11 wand blank tip (V) | robe
  '....XSX...XYYYOOOWWWWOOOOYOX....', // 12 blank shaft (S)
  '...PXSXP..XYOOOWWWWWWOOOOYOX....', // 13 left hand grips blank | robe lit
  '...PRSRPX.XYOOOWWWWWWOOOOYPRP...', // 14 hand | right hand out
  '...PPNPPX.XYOOOOOOOOOOOOYPPPP...', // 15 fitting (N) | right hand open
  '....NGNX..XXYYOOOOOOOOYYXXPPX...', // 16 fitting spec (G)
  '....XZX...XYYOOOWWWWWOOYYX......', // 17 wand collar (Z)
  '..........XXXNZZZZZZNXXXX.......', // 18 belt band
  '..........XYYOOOOOOOOYYX........', // 19 robe lower
  '..ZZZZZZ..XYYOOOWWWWOOYYX.......', // 20 bench top (Z)
  '..ZUUUUZ..XYYOOOOOOOOYYX........', // 21 bench apron (U)
  '..ZNGNZZ..XYYOOOWWWWOOYYX.......', // 22 tool on bench (N/G)
  '..ZXNXZZ..XYFFFFFFFFFFYYX.......', // 23 tool head | hem
  '..XX.XXX..XFFFFFXXFFFFFFX.......', // 24 bench legs | leg split
  '..X...X...XZZZZX..XZZZZX........', // 25 | boots
  '..X...X...XZUUZX..XZUUZX........', // 26
  '..X...X...XXXXXX..XXXXXX........', // 27 soles
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF37 = function REF37({ scale = 8 }) {
  return <PixelArt grid={REF37_GRID} scale={scale} title="REF-37 The Wandwright" />;
};
window.REF37_GRID = REF37_GRID;
window.REF37_ROLES = ['X','Y','O','W','R','P','B','S','V','N','Z','G','U','F'];
