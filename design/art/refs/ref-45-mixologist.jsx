// SPDX-License-Identifier: MIT
// REF-45 - The Mixologist, 32x32, idle, transparent bg.
// In the frame-left hand a Bronze cocktail shaker, in the frame-right hand
// a glass with a Teal drink in it. She is mid-pour, transforming what's in
// the glass - but only IF there is something to pour. If the glass holds a
// drink (Some), the shake changes it; if the glass is empty (None), the
// shake changes nothing and passes through. That is `Option::map`: apply
// the transform inside the Some, leave None untouched. Crypt tunic, the
// teal drink is the single accent, no magic violet.
// Mission: option_map - `.map()` a closure over the inside of an `Option`.
//
// Legend:
//   X coalblack (outline, shaker/glass edges)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (shaker bands, belt)   Z bronze (shaker body)
//   G old gold (shaker cap, buckle)   L brass leaf (cap spec)
//   T main teal (drink)   I bright teal (drink hi)   M mist teal (pour stream)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF45_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX...XGLX.....', //  8 jaw | shaker cap (G/L)
  '.XGLX...XYPPPRRRRPYX...XGGX.....', //  9 shaker cap (frame-left low) | cap
  '.XNNX..XYYYYYYYYYYYYX..XNNX.....', // 10 shaker band | shaker band
  '.XZZX.XYYYOOOOOOOOYYYX.XZZX.....', // 11 shaker body | shaker body
  '.XZZXPXYYYOOOWWWWOOOYXP.M.X.....', // 12 hands grip (P) | pour stream M
  '.XNNPRPYOOOWWWWWWOOOYPRPM.X.....', // 13 band | pour into glass
  '.XZZPPPYOOOOOOOOOOOOYPPXTIX.....', // 14 hands | glass with drink (T/I)
  '.XXX..XYYOOOWWWWWWOOYYXXTTX.....', // 15 | glass body
  '......XXNGGGGGGGGGGNXX.XTTX.....', // 16 belt band | glass
  '......XKYYOOOOOOOOYYKX.XXXX.....', // 17 robe | glass base
  '......XKYYOOOWWWWOOYYKX.........', // 18
  '.....XKKYYOOOOOOOOOOYYKKX.......', // 19 robe widens
  '.....XKYYYOOOWWWWWWOOYYKX.......', // 20
  '.....XKYYYOOOOOOOOOOOYYKX.......', // 21
  '.....XKYYYOOOOOOOOOOOYYKX.......', // 22
  '....XKKYYYOOOOOOOOOOOYYKKX......', // 23 widest
  '....XKYYFFFFFFFFFFFFFFYKX.......', // 24 hem
  '....XKFFFFFFXXX.XXXFFFFFKX......', // 25 leg split
  '....XXZZZZZXX.....XXZZZZX.......', // 26 boots
  '.....XZUUUZX.......XZUUZX.......', // 27
  '.....XXXXXXX.......XXXXXX.......', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF45 = function REF45({ scale = 8 }) {
  return <PixelArt grid={REF45_GRID} scale={scale} title="REF-45 The Mixologist" />;
};
window.REF45_GRID = REF45_GRID;
window.REF45_ROLES = ['X','Y','O','W','R','P','B','S','V','N','Z','G','L','T','I','M','U','F'];
