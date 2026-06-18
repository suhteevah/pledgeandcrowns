// REF-26 - The Quartermaster, 32x32, idle, transparent bg.
// Stout supply officer in a Crypt gambeson, clean-shaven, holding a
// tally rod toward a rack of gold grain sacks. A bracket frames three
// sacks - the borrowed window (a slice &[T] = pointer + length).
// Mission: slice_basic - sum a borrowed slice you do not own.

// Legend:
//   X coalblack (outline, tally rod, bracket)
//   Y crypt (gambeson deep)   O oxblood (gambeson primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)        B basalt (brow/eyes)
//   S stone grey (hair)       V aged paper (hair hi)
//   N antique brass (collar stud, sack ties)  Z bronze (rack, sack tie shadow)
//   G old gold (sacks)        L brass leaf (sack highlight, bracket tip)
//   U bog umber (boots)       F pine (hem)
//   . transparent

const REF26_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..........XXXXXXXX..............', //  3 head top
  '.........XSSSVVSSSX.............', //  4 hair
  '........XSSVVVVVVSSX............', //  5
  '........XSPPPPPPPPSX............', //  6
  '........XPPPPPPPPRRX............', //  7 forehead
  '........XPBPPPPBPRWX............', //  8 brow / eyes
  '........XPPRRRRRRRPX............', //  9
  '........XXPRRRRRRPXX............', // 10 jaw (clean-shaven)
  '........XNPPPRRRRPNX............', // 11 collar studs (N)
  '.......XYYYYYYYYYYYYX...........', // 12 shoulders
  '......XYYYOOOOOOOOYYYX..........', // 13 robe upper lit
  '......XYOOOWWWWOOOOYOX.ZZZZZZ...', // 14 robe + rack top (Z)
  '......XYOOWWWWWWOOOYOX.ZGGGGGZ..', // 15 sack row 1 (G)
  '......XYOOOWWWWOOOOYOX.ZGLLGGZ..', // 16 sack highlight (L)
  '....XPPPYOOOOOOOOOOYOX.ZNGGGNZ..', // 17 left hand + sack ties (N)
  '....XPRPYOOOOOOOOOYYOXLZZZZZZZ..', // 18 right hand grips rod; bracket tip L
  '....XPPPYOOOOOOOOOYYOXXZGGGGGZ..', // 19 rod down (X) + sack row 2
  '......XXYYOOOOOOOOOYYX.ZGLLGGZ..', // 20
  '......XYYOOOOOOOOOOYYX.ZNGGGNZ..', // 21
  '......XXXNGGGGGGGGNXXX.ZZZZZZZ..', // 22 belt buckle (G)
  '......XYYOOOOOOOOOOYYX..........', // 23 robe lower
  '......XYYOOWWWWWOOOYYX..........', // 24
  '......XYYOOOOOOOOOOYYX..........', // 25
  '......XYYOOOOOOOOOOYYX..........', // 26
  '......XYFFFFFFFFFFFFYX..........', // 27 hem
  '......XFFFFXX..XXFFFFX..........', // 28 leg split
  '......XZZZZX....XZZZZX..........', // 29 boots
  '......XZUUZX....XZUUZX..........', // 30
  '......XXXXXX....XXXXXX..........', // 31 soles
];

window.REF26 = function REF26({ scale = 8 }) {
  return <PixelArt grid={REF26_GRID} scale={scale} title="REF-26 The Quartermaster" />;
};
window.REF26_GRID = REF26_GRID;
window.REF26_ROLES = ['X','Y','O','W','R','P','B','S','V','N','Z','G','L','U','F'];
