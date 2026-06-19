// SPDX-License-Identifier: MIT
// REF-47 - The Cellarer, 32x32, idle, transparent bg.
// Stands beside a wooden cask/barrel (frame-left), a Bronze ladle in the
// frame-right hand. If the cask still holds brew he ladles from it; if the
// cask is EMPTY he does not error - he brews a fresh default batch on the
// spot. The work of making the default is only done WHEN it is needed, not
// before. That is `.unwrap_or_else`: take the value if present, else call
// a closure to produce a fresh one - the default computed lazily. Crypt
// cellar tunic, Old-gold cask bands, a faint Old-gold brew level, no magic.
// Mission: unwrap_or_else - lazy default via `.unwrap_or_else(|| ...)`.
//
// Legend:
//   X coalblack (outline, cask staves, ladle)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (cask staves, ladle bowl)   N antique brass (cask hoops, belt)
//   G old gold (cask brew level, buckle)   L brass leaf (brew spec)
//   U bog umber (cask shadow, boots)   F pine (hem)
//   . transparent

const REF47_GRID = [
  '................................', //  0
  '.............XXXXXXXX...........', //  1 head top
  '............XSSSVVSSSX..........', //  2 hair
  '...........XSSVVVVVVSSX.........', //  3
  '...........XSPPPPPPPPSX.........', //  4 forehead
  '...........XPPPPPPPPRRX.........', //  5
  '...........XPBPPPPBPRWX.........', //  6 brow/eyes
  '...........XPPRRRRRRRPX.........', //  7
  '..XNNNNNNX.XXPRRRRRRPXX.........', //  8 cask top hoop | jaw
  '..XZZZZZZX.XYPPPRRRRPYX.........', //  9 cask staves | collar
  '..XGGGGGGX.XYYYYYYYYYYYX........', // 10 brew level (G) | shoulders
  '..XZGGGGLX.XYYOOOOOOOOYYX.......', // 11 brew hi (L) | robe upper
  '..XNNNNNNXXYYYOOOWWWOOOYYX......', // 12 hoop | robe lit
  '..XZZZZZZXYYOOOWWWWWOOOYYXP.....', // 13 staves | hand reaches out (P)
  '..XZZZZZZXYOOOOOOOOOOOOYXPZP....', // 14 | hand grips ladle (P) ladle bowl Z
  '..XNNNNNNXYOOOWWWWWOOOYXPZZZP...', // 15 hoop | ladle bowl (Z)
  '..XZZZZZZXXNGGGGGGGGNXX.PZZP....', // 16 staves | belt band | ladle
  '..XZZZZZZXKYYOOOOOOYYKX..XZX....', // 17 | ladle handle
  '..XUUUUUUXKYYOOWWWWOYYKX.XZX....', // 18 cask base shadow | handle
  '..XXXXXXXXKKYOOOOOOOYYKKX.XX....', // 19 cask bottom | robe widens | handle end
  '.....XKYYYYOOOWWWWWOOOYYKX......', // 20
  '.....XKYYYYOOOOOOOOOOOYYKX......', // 21
  '.....XKYYYYOOOOOOOOOOOYYKX......', // 22
  '....XKKYYYYOOOOOOOOOOOYYKKX.....', // 23 widest
  '....XKYYFFFFFFFFFFFFFFFYKX......', // 24 hem
  '....XKFFFFFFXXX..XXXFFFFFKX.....', // 25 leg split
  '....XXZZZZZXX......XXZZZZX......', // 26 boots
  '.....XZUUUZX........XZUUZX......', // 27
  '.....XXXXXXX........XXXXXX......', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF47 = function REF47({ scale = 8 }) {
  return <PixelArt grid={REF47_GRID} scale={scale} title="REF-47 The Cellarer" />;
};
window.REF47_GRID = REF47_GRID;
window.REF47_ROLES = ['X','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
