// SPDX-License-Identifier: MIT
// REF-39 - The Familiar Keeper, 32x32, idle, transparent bg.
// Accompanied by a polyglot familiar in a bronze cage held aloft: the
// creature is painted in several element colors AT ONCE - a Main-teal
// wing, an Old-gold body, a Forest tail - many concrete types behind one
// container. That is Box<dyn Trait>: the cage (the box) hides which exact
// element the familiar is; you only know it speaks the Familiar trait.
// Crypt robe; the multi-color creature is the accent, no violet.
// Mission: dyn_trait - Box<dyn Trait>, dynamic dispatch over many types.
//
// Legend:
//   X coalblack (outline, cage bars)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes, creature eye)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (cage frame, boots)   N antique brass (cage ring)   G old gold (creature body)
//   T main teal (creature wing)   E forest (creature tail)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF39_GRID = [
  '................................', //  0
  '.............NZZZZN.............', //  1 cage hook + top ring (N/Z)
  '.............Z....Z.............', //  2 cage frame
  '............ZX.TT.XZ............', //  3 cage bars | teal wing peeks
  '...XXXXXXXX.ZXTTGGTXZ...........', //  4 head top | creature: teal wing + gold body
  '..XSSSVVSSSX.XGGGGBGX...........', //  5 hair | gold body + eye (B)
  '.XSSVVVVVVSSXXGGGGGEX...........', //  6 | gold body + forest tail start
  '.XSPPPPPPPPSX.XEEEE.X...........', //  7 forehead | forest tail (E)
  '.XPPPPPPPPRRX.ZX..XZ............', //  8 | cage lower bars
  '.XPBPPPPBPRWX..ZNNZ.............', //  9 brow/eyes | cage base ring
  '.XPPRRRRRRRPX...XX..............', // 10
  'XXPRRRRRRPXXX..PPP..............', // 11 jaw | raised hand holds cage
  'XYPPPRRRRPYX..PRRP..............', // 12 collar | hand (P/R)
  'XYYYYYYYYYYYX.PPPP..............', // 13 shoulders | wrist
  'XYYYOOOOOOOOYYYX................', // 14 robe upper
  'XYYOOOWWWWOOOOYOX...............', // 15 robe lit
  'XYOOOWWWWWWOOOOYOX..............', // 16
  'XYOOOOOOOOOOOOOYOX..............', // 17
  'XYOOOWWWWWWOOOOYOX..............', // 18
  'XXNGGGGGGGGGGGNXXX..............', // 19 belt band (N/G)
  'XYYOOOOOOOOOOYYX................', // 20 robe lower
  'XYYOOOWWWWWOOYYX................', // 21
  'XYYOOOOOOOOOOYYX................', // 22
  'XYYOOOOOOOOOOYYX................', // 23
  'XYFFFFFFFFFFFFYX................', // 24 hem
  'XFFFFFXX..XXFFFFX...............', // 25 leg split
  'XZZZZX....XZZZZX................', // 26 boots
  'XZUUZX....XZUUZX................', // 27
  'XXXXXX....XXXXXX................', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF39 = function REF39({ scale = 8 }) {
  return <PixelArt grid={REF39_GRID} scale={scale} title="REF-39 The Familiar Keeper" />;
};
window.REF39_GRID = REF39_GRID;
window.REF39_ROLES = ['X','Y','O','W','R','P','B','S','V','Z','N','G','T','E','U','F'];
