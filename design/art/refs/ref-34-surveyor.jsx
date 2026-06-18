// SPDX-License-Identifier: MIT
// REF-34 - The Surveyor, 32x32, idle, transparent bg.
// A Crypt-robed surveyor stands beside a tall graduated measuring rod held
// in the right hand: a bronze staff ticked in old gold at each unit. A bare
// length, but every tick names it - a number wrapped in a newtype `Meters`.
// Mission: tuple_struct - `struct Meters(f64)`, a named scalar.
//
// Legend:
//   X coalblack (outline, rod core)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (collar stud, rod foot)   Z bronze (rod shaft)
//   G old gold (graduation ticks)   L brass leaf (top tick spec)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF34_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX.....XX.......', //  2 head top | rod top
  '.........XSSSVVSSSX....XLX......', //  3 hair | rod top tick (L)
  '........XSSVVVVVVSSX...XZX......', //  4 | rod
  '........XSPPPPPPPPSX...XGX......', //  5 forehead | tick (G)
  '........XPPPPPPPPRRX...XZX......', //  6
  '........XPBPPPPBPRWX...XZX......', //  7 brow / eyes | rod
  '........XPPRRRRRRRPX...XGX......', //  8 | tick (G)
  '........XXPRRRRRRPXX...XZX......', //  9 jaw
  '........XNPPPRRRRPNX...XZX......', // 10 collar studs | rod
  '.......XYYYYYYYYYYYYX..XGX......', // 11 shoulders | tick (G)
  '......XYYYOOOOOOOOYYYX.XZX......', // 12 robe upper lit | rod
  '......XYOOOWWWWOOOOYOX.XZX......', // 13 | rod
  '......XYOOWWWWWWOOOYOXPXGXP.....', // 14 right hand grips rod | tick (G)
  '......XYOOOWWWWOOOOYPRRZRPX.....', // 15 hand wraps rod shaft
  '....PPPYOOOOOOOOOOYOPPXZXPP.....', // 16 left hand on hip | rod
  '....PRPYOOOOOOOOOYYOX..XGX......', // 17 | tick (G)
  '....PPPYOOOOOOOOOYYOX..XZX......', // 18
  '......XXYYOOOOOOOOYYXX.XZX......', // 19 | rod
  '......XYYOOOWWWWOOOYYX.XGX......', // 20 robe mid | tick (G)
  '......XYYOOOOOOOOOOYYX.XZX......', // 21
  '......XXXNGGGGGGGGNXXX.XZX......', // 22 belt buckle | rod
  '......XYYOOOOOOOOOOYYX.XGX......', // 23 robe lower | tick (G)
  '......XYYOOOWWWWOOOYYX.XZX......', // 24
  '......XYYOOOOOOOOOOYYX.XZX......', // 25
  '......XYYOOOOOOOOOOYYX.NNN......', // 26 rod foot (N)
  '......XYFFFFFFFFFFFFYX..........', // 27 hem
  '......XFFFFXX..XXFFFFX..........', // 28 leg split
  '......XZZZZX....XZZZZX..........', // 29 boots
  '......XZUUZX....XZUUZX..........', // 30
  '......XXXXXX....XXXXXX..........', // 31 soles
];

window.REF34 = function REF34({ scale = 8 }) {
  return <PixelArt grid={REF34_GRID} scale={scale} title="REF-34 The Surveyor" />;
};
window.REF34_GRID = REF34_GRID;
window.REF34_ROLES = ['X','Y','O','W','R','P','B','S','V','N','Z','G','L','U','F'];
