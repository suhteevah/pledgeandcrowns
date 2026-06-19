// SPDX-License-Identifier: MIT
// REF-60 - The Vaultwright, 32x32, idle, transparent bg.
// Cradles a sturdy Old-gold-bound strongbox in both arms, lid latched, a
// value lifted off the ground and set down onto the heap inside it. He does
// not hold the value loose in his hands - he BOXES it: takes the thing,
// puts it in a strongbox, and from then on carries a single handle to the
// box, not the bulk of the value itself. That is `Box::new`: the value
// moves to the heap, and what you hold is the pointer. Crypt keeper tunic,
// Old-gold chest bands and lock, no magic.
// Mission: box_basic - put a value on the heap with Box::new.
//
// Legend:
//   X coalblack (outline, chest seam, lock keyhole)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (chest body, boots)   N antique brass (chest bands, belt)
//   G old gold (chest lid, lock)   L brass leaf (lock spec)
//   U bog umber (boot shadow, chest base)   F pine (hem)
//   . transparent
//
// The Old-gold strongbox (Z body, N bands, G lid, L lock spec) cradled in
// both Pink-quartz arms is the heap allocation; the box holds the value and
// the Vaultwright carries the handle to it.

const REF60_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX............', //  8 jaw
  '........XYPPPRRRPYYX............', //  9 collar
  '.......XYYYYYYYYYYYYX...........', // 10 shoulders
  '......XYYOOOOOOOOOYYYX..........', // 11 tunic upper
  '.....XYYOOOWWWWWOOOYYYX.........', // 12 tunic lit
  '....PRPYOOOOOOOOOOOOYPRPX.......', // 13 arms come round (P) to cradle
  '...PPRPXNGGGGGGGGGGNXPPRPX......', // 14 chest lid top (G) brass band (N)
  '...PPRXNGGGGGGGGGGGGNXRPPX......', // 15 lid (G)
  '...PPXNGGGXGGLLGGXGGGNXPPX......', // 16 lock plate (G) lock spec (L)
  '...PPXZNNNXGGGGGGXNNNZXPPX......', // 17 band (N) keyhole region (X)
  '....XZZZZZXXNNNNXXZZZZZX........', // 18 chest body (Z) keyhole (X)
  '....XZZZZZZZNNNNZZZZZZZX........', // 19 chest body
  '....XNNNNNNNNNNNNNNNNNNX........', // 20 lower band (N)
  '....XUUUUUUUUUUUUUUUUUUX........', // 21 chest base (U)
  '...XKYYOOOOOOOOOOOOYYKX.........', // 22 tunic lower behind box
  '...XKYYOOOWWWWWWOOYYKKX.........', // 23
  '...XKKYOOOOOOOOOOOYYKKX.........', // 24 widest
  '...XKYYFFFFFFFFFFFFYYKX.........', // 25 hem
  '...XKFFFFFXXX..XXXFFFFKX........', // 26 leg split
  '...XXZZZZZXX....XXZZZZX.........', // 27 boots
  '....XZUUUZX......XZUUZX.........', // 28
  '....XXXXXXX......XXXXXX.........', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF60 = function REF60({ scale = 8 }) {
  return <PixelArt grid={REF60_GRID} scale={scale} title="REF-60 The Vaultwright" />;
};
window.REF60_GRID = REF60_GRID;
window.REF60_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
