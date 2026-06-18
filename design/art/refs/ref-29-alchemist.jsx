// REF-29 - The Alchemist, 32x32, idle, transparent bg.
// Behind a workbench: a row of input vials (Deep teal) on the left, one
// vial mid-transmutation over a brass still (Bright teal, doubled), and
// a large collecting flask (Main teal) on the right. iter -> map -> collect.
// Mission: iter_map_collect - map each element, gather into a new Vec.
// Teal is the alchemical reagent only - the one accent that sets the
// Alchemist apart from the gold-and-burgundy cast.

// Legend:
//   X coalblack (outline, burner)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin)   P pink quartz (skin hi)     B basalt (brow)
//   S stone grey (hair)
//   U bog umber (apron)   Z bronze (apron trim, still)   N antique brass (loupe)
//   G old gold (flask rim)
//   D deep teal (input vials)   T main teal (collected pool)
//   I bright teal (mid-transform)   M mist teal (vapour)
//   . transparent

const REF29_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSSSSSSSX.............', //  3 hair
  '.........XSPPPPPPSXN............', //  4 loupe on brow (N)
  '.........XPPPPPPPRX.............', //  5 forehead
  '.........XPBPPPBPRX.............', //  6 brow / eyes
  '.........XPPRRRRRPX.............', //  7
  '.........XXPRRRRPXX.............', //  8 jaw
  '.........XIPPRRPPIX.............', //  9 teal under-light (I) on jaw
  '........XYYYYYYYYYYX............', // 10 collar
  '.......XYYOOOOOOOOYYX...........', // 11 shoulders
  '......XYYOOOWWWWOOOYYX..........', // 12 robe upper
  '......XYOOOWWWWWWOOYOX..........', // 13
  '......XUUUUUUUUUUUUUUX..........', // 14 apron top (U)
  '....XPPUUZZZZZZZZZZUUPPX........', // 15 hands + apron trim (Z)
  '....XPRUUUUUUUUUUUUUURPX........', // 16 right hand tips the vial
  '...DDD.UUUUUUUUUUUUU.GGGGG......', // 17 input vials (D) | flask rim (G)
  '..DDDDD.XUUUUUUUUUX.GTTTTTG.....', // 18 vials + collecting flask (T)
  '..DDDDDX.XZIZX....X.GTTTTTG.....', // 19 still + mid-transform (I)
  '..DDDDDX..XIIX..MM..GTTTTTG.....', // 20 burner; vapour (M) rising
  '..DDDDDX..XIIX.M.M..GTTTTTG.....', // 21
  '..XXXXXX..XZZX..MM..GTTTTTG.....', // 22 bench edge; flask body
  '......XYYOOOOOOOOOOYYX.GTTTTG...', // 23 robe lower behind bench
  '......XYYOOWWWWWOOOYYX..GGGGG...', // 24 flask base
  '......XYYOOOOOOOOOOYYX..........', // 25
  '......XYYOOOOOOOOOOYYX..........', // 26
  '......XYYYOOOOOOOOYYYX..........', // 27 hem
  '......XXZZZX....XZZZXX..........', // 28 boots
  '.......XUUZX....XZUUX...........', // 29
  '.......XUUZX....XZUUX...........', // 30
  '.......XXXXX....XXXXX...........', // 31 soles
];

window.REF29 = function REF29({ scale = 8 }) {
  return <PixelArt grid={REF29_GRID} scale={scale} title="REF-29 The Alchemist" />;
};
window.REF29_GRID = REF29_GRID;
window.REF29_ROLES = ['X','Y','O','W','R','P','B','S','U','Z','N','G','D','T','I','M'];
