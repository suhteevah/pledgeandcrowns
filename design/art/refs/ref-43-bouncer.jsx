// SPDX-License-Identifier: MIT
// REF-43 - The Bouncer, 32x32, idle, transparent bg.
// Burly, broad-shouldered, ARMS CROSSED over the chest. On his belt or
// frame-side a small Old-gold tally board lists the kinds of trouble by
// name: each Coalblack rule mark is one named variant. Every sort of
// trouble has its OWN name - that is a custom error enum. The robe is the
// widest, heaviest cut in the tavern cast; no apron, no prop in the hands
// (the crossed arms ARE the stance). Crypt uniform, gold tally, no magic.
// Mission: custom_error - define your own error `enum` of named variants.
//
// Legend:
//   X coalblack (outline, tally rule marks)
//   K inkblood (robe deep shadow)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (tally board frame, belt band)   G old gold (tally face, buckle)
//   L brass leaf (tally spec)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF43_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair (short, flat)
  '..........XSSVVVVSSX............', //  3
  '..........XPPPPPPPPX............', //  4 forehead (heavy brow)
  '..........XPBBPPBBPX............', //  5 brow line
  '..........XPBPPPPBPX............', //  6 eyes
  '..........XPPRRRRRPX............', //  7
  '..........XXPRRRRPXX............', //  8 jaw (broad)
  '.........XXYPPRRPYXX............', //  9 thick neck/collar
  '.......XXYYYYYYYYYYYYXX.........', // 10 wide shoulders
  '......XYYYOOOOOOOOOOYYYX........', // 11 robe upper
  '.....XYYOOOOWWWWWWOOOOYYX.......', // 12 broad chest lit
  '.....XYYOOOOOOOOOOOOOOYYX.......', // 13
  '.....XYYOPPPRRRRRRPPPOYYX......L', // 14 crossed forearms (P/R) | tally spec
  '.....XYYPRRPOOOOOOPRRPYYX....NGN', // 15 crossed arms over chest | tally top
  '.....XYYOPPPOOOOOOPPPOYYX....NXG', // 16 forearms meet | tally rule
  '.....XKYYOOOOOOOOOOOOYYKX....NGG', // 17 belt above waist | tally rule
  '.....XXNGGGGGGGGGGGGNXX......NXG', // 18 belt band (N) buckle (G) | tally rule
  '.....XKYYOOOOOOOOOOYYKX......NGG', // 19 robe lower | tally rule
  '....XKKYYOOOWWWWWWOOYYKKX....NXG', // 20 robe widens (heavy) | tally rule
  '....XKYYYOOOOOOOOOOOOYYKX....NGN', // 21 | tally bottom
  '....XKYYYOOOWWWWWWOOOYYKX.......', // 22
  '....XKYYYOOOOOOOOOOOOYYKX.......', // 23
  '...XKKYYYOOOOOOOOOOOOYYKKX......', // 24 widest at hem
  '...XKYYFFFFFFFFFFFFFFYYKX.......', // 25 hem
  '...XKFFFFFFXXX..XXXFFFFFKX......', // 26 leg split (thick legs)
  '...XXZZZZZXX......XXZZZZZX......', // 27 boots
  '....XZUUUZX........XZUUUZX......', // 28
  '....XXXXXXX........XXXXXXX......', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF43 = function REF43({ scale = 8 }) {
  return <PixelArt grid={REF43_GRID} scale={scale} title="REF-43 The Bouncer" />;
};
window.REF43_GRID = REF43_GRID;
window.REF43_ROLES = ['X','K','Y','O','W','R','P','B','S','V','N','G','L','U','F'];
