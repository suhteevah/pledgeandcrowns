// SPDX-License-Identifier: MIT
// REF-64 - The Strongbox keeper, 32x32, idle, transparent bg.
// Stands behind a heavy Bronze strongbox with MULTIPLE keyholes across its
// face, several Old-gold keys ringed and shared out, and the lid cracked
// open mid-turn so the contents can be changed. Many owners each hold a key
// to the same box, AND any of them can open it to mutate what is inside.
// That is `Rc<RefCell<T>>`: shared ownership (the many keys, the ref count)
// wrapped around interior mutability (the lid that opens to a runtime-checked
// borrow). Crypt keeper tunic, Old-gold keys and lock plates, no magic.
// Mission: rc_refcell - share AND mutate with Rc<RefCell<T>>.
//
// Legend:
//   X coalblack (outline, box seams, keyholes)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi, key hand)
//   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (strongbox body, boots)   N antique brass (box bands, belt)
//   G old gold (keys, lock plates, lid)   L brass leaf (key/lock spec)
//   U bog umber (boots, box base)   F pine (hem)
//   . transparent
//
// The heavy Bronze strongbox with several Old-gold keys (G, L spec) on a
// shared ring and MULTIPLE keyholes (X) under brass lock plates is shared
// ownership; the lid cracked open mid-turn is the interior mutation - many
// owners, each able to open and change, borrow-checked at runtime.

const REF64_GRID = [
  '................................', //  0
  '..........XXXXXXXX..............', //  1 head top
  '.........XSSSVVSSSX.............', //  2 hair
  '........XSSVVVVVVSSX............', //  3
  '........XSPPPPPPPPSX............', //  4 forehead
  '........XPPPPPPPPRRX............', //  5
  '........XPBPPPPBPRWX............', //  6 brow/eyes
  '........XPPRRRRRRRPX............', //  7
  '.......XXPRRRRRRPXX.............', //  8 jaw
  '.......XYPPPRRRPYYX.............', //  9 collar
  '......XYYYYYYYYYYYYX............', // 10 shoulders
  '.....XYYOOOOOOOOOYYYX...........', // 11 tunic upper
  '....XYYOOOWWWWWOOOYYYX..........', // 12 tunic lit
  '...PRPOOOOOOOOOOOOOYOX..........', // 13 key arm (P) reaches to ring
  '..PPRPGLGGGLGGGLGOOYOX..........', // 14 key ring lifts (G keys) lid line
  '..PPXGGLGGGLGGGLGGGYOX..........', // 15 several keys (G/L spec)
  '...PXNNNNNNNNNNNNNNNNX..........', // 16 box top band (N) lid cracked
  '....XGGGGXGGGGXGGGGGGX..........', // 17 lock plates (G) row
  '....XGLGGXGLGGXGLGGGGX..........', // 18 plate specs (L)
  '....XZXZZXZXZZXZXZZGGX..........', // 19 keyholes (X) across face
  '....XZZZZZZZZZZZZZZZZX..........', // 20 strongbox body (Z)
  '....XNNNNXNNNNXNNNNNNX..........', // 21 mid bands (N)
  '....XZZZZZZZZZZZZZZZZX..........', // 22 strongbox body
  '....XUUUUUUUUUUUUUUUUX..........', // 23 box base (U)
  '...XKYYOOOOOOOOOOOOYKX..........', // 24 tunic lower behind box
  '...XKYYFFFFFFFFFFFFYKX..........', // 25 hem
  '...XKFFFFFXXX..XXXFFFFX.........', // 26 leg split
  '...XXZZZZZXX....XXZZZZX.........', // 27 boots
  '....XZUUUZX......XZUUZX.........', // 28
  '....XXXXXXX......XXXXXX.........', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF64 = function REF64({ scale = 8 }) {
  return <PixelArt grid={REF64_GRID} scale={scale} title="REF-64 The Strongbox" />;
};
window.REF64_GRID = REF64_GRID;
window.REF64_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
