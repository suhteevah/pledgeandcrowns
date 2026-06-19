// SPDX-License-Identifier: MIT
// REF-63 - The Swapwarden, 32x32, idle, transparent bg.
// Stands over a single Bronze socket-slot set in a pedestal. One Old-gold
// coin sits in the slot; with two fingers she lifts it out while the next
// coin drops in to take its place - one value swapped for another in the
// same single cell, no borrowing, no peeking at what was there, just set
// and replace. That is `Cell`: interior mutability for one value at a time,
// `set` and `replace`, mutate-through-shared without a borrow check because
// the old value simply moves out whole. Crypt tunic, Old-gold coins and
// slot rim, no magic.
// Mission: cell - set or replace a single value through Cell.
//
// Legend:
//   X coalblack (outline, slot seam)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi, swapping fingers)
//   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (pedestal, slot rim, boots)   N antique brass (slot band, belt)
//   G old gold (the two coins)   L brass leaf (coin spec)
//   U bog umber (boots, pedestal base)   F pine (hem)
//   . transparent
//
// Two Old-gold coins (G, L spec) - one lifting out above the slot, one
// dropping into the single Bronze socket (Z rim, N band) - is the swap: one
// value replaces another in the same cell, set and replace, no borrow.

const REF63_GRID = [
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
  '.....XYYOOOOOOOOOYYYX.GLG.......', // 11 tunic | coin lifting out (G/L)
  '....XYYOOOWWWWWOOOYYYXGGGG......', // 12 tunic lit | lifted coin
  '...XYYOOOOOOOOOOOOOYOXXPPX......', // 13 swap fingers (P) above slot
  '...XYOOOOWWWWWWOOOYYOXPRPX......', // 14 fingers pinch the coin
  '...XYOOOOOOOOOOOOOYYOX.PX.......', // 15
  '...XKYYOOOOOOOOOOOYYKX..........', // 16 tunic lower
  '...XKYYOOOWWWWWWOOYYKX..........', // 17
  '...XKKYOOOOOOOOOOYYKKX..........', // 18 widest
  '...XKYYFFFFFFFFFFFFYKX..........', // 19 hem
  '...XKFFFFFXXX..XXXFFFFX.........', // 20 leg split
  '...XXZZZZXX....XXZZZZX..........', // 21 boots
  '....XZUUZX......XZUUZX..........', // 22
  '....XXXXXX......XXXXXX..........', // 23 soles
  '.........XNNNNNNNNX.............', // 24 slot band top (N)
  '........XNGGLLGGGGNX............', // 25 coin dropping in (G/L)
  '........XZGGGGGGGGZX............', // 26 coin seated in slot (G)
  '........XZNNNNNNNNZX............', // 27 slot band (N) rim (Z)
  '........XZZZZZZZZZZX............', // 28 pedestal (Z)
  '........XUUUUUUUUUUX............', // 29 pedestal base (U)
  '........XXUUUUUUUUXX............', // 30
  '.........XXXXXXXXXX.............', // 31 base
];

window.REF63 = function REF63({ scale = 8 }) {
  return <PixelArt grid={REF63_GRID} scale={scale} title="REF-63 The Swapwarden" />;
};
window.REF63_GRID = REF63_GRID;
window.REF63_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
