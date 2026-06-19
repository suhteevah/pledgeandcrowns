// SPDX-License-Identifier: MIT
// REF-62 - The Warden, 32x32, idle, transparent bg.
// Guards a Bronze lockbox latched in one arm, an Old-gold key turned in
// the lock with the other hand, the lid lifting so he can change what is
// inside - even though the box itself was handed to him sealed, "shared,
// look-but-don't-touch". A small ledger at his hip records who is borrowing
// the contents right now, so two hands never reach in at once. That is
// `RefCell`: interior mutability, the borrow rules moved from compile time
// to a runtime check the Warden enforces. Crypt tunic, Old-gold key/lock,
// parchment borrow-ledger, no magic.
// Mission: refcell - mutate through a shared handle with RefCell.
//
// Legend:
//   X coalblack (outline, lock seam, ledger ticks)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi, key hand)
//   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi, ledger line)
//   Z bronze (lockbox body, boots)   N antique brass (lockbox band, belt)
//   G old gold (key, lock plate)   L brass leaf (key spec)
//   U bog umber (boots)   F pine (hem)
//   C parchment cream (borrow-ledger page)
//   . transparent
//
// The Old-gold key (G->L) turned in the lock of the Bronze box is the
// runtime borrow check; the lifting lid is the interior mutation. The
// parchment ledger (C) at the hip is the borrow flag - one borrower at a
// time, checked when you reach in, not when you compile.

const REF62_GRID = [
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
  '...PRPYOOOOOOOOOOOOYOX..........', // 13 key arm (P) reaches across
  '..PPRPGGGGNNNNNNGGOOYX.CCCC.....', // 14 lid lifting (N) | ledger (C)
  '..PPXGLLGXNGGGGNXZZOYX.CXVC.....', // 15 key (G) spec (L) | ledger tick
  '...PXGGGGXGGLLGGXZZOYX.CXVC.....', // 16 key in lock plate (G/L) | tick
  '....XZNNNXGGGGGGXNNZX..CXVC.....', // 17 lockbox band (N) lock (G)
  '....XZZZZXNNNNNNXZZZX..CCCC.....', // 18 lockbox body (Z) | ledger foot
  '....XZZZZZZZZZZZZZZZX...........', // 19 lockbox body
  '....XNNNNNNNNNNNNNNNX...........', // 20 lower band (N)
  '....XUUUUUUUUUUUUUUUX...........', // 21 box base (U)
  '...XKYYOOOOOOOOOOYYKX...........', // 22 tunic lower
  '...XKYYOOOWWWWWOOYYKKX..........', // 23
  '..XKKYOOOOOOOOOOOYYKKX..........', // 24 widest
  '..XKYYFFFFFFFFFFFFYYKX..........', // 25 hem
  '..XKFFFFFXXX..XXXFFFFKX.........', // 26 leg split
  '..XXZZZZZXX....XXZZZZX..........', // 27 boots
  '...XZUUUZX......XZUUZX..........', // 28
  '...XXXXXXX......XXXXXX..........', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF62 = function REF62({ scale = 8 }) {
  return <PixelArt grid={REF62_GRID} scale={scale} title="REF-62 The Warden" />;
};
window.REF62_GRID = REF62_GRID;
window.REF62_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','C'];
