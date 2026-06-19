// SPDX-License-Identifier: MIT
// REF-46 - The Tabkeeper, 32x32, idle, transparent bg.
// Holds a long Parchment-cream tab strip that hangs from his hand down the
// frame-right side: a CHAIN of orders, one entry under the next, each
// Coalblack rule mark a link. Every order must succeed for the next to be
// served; the first failure ends the strip - the chain stops there. That
// is `.and_then`: each fallible step feeds the next only on success, and
// any Err short-circuits the rest. Crypt clerk's robe, Old-gold tab clip,
// no magic.
// Mission: and_then - chain fallible steps with `.and_then` (short-circuit).
//
// Legend:
//   X coalblack (outline, tab rule marks, chain links)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (tab clip)   G old gold (tab clip face, buckle)   L brass leaf (clip spec)
//   C parchment cream (tab strip)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF46_GRID = [
  '................................', //  0
  '..........XXXXXXXX.............L', //  1 head top | clip spec
  '.........XSSSVVSSSX..........NGN', //  2 hair | tab clip (N/G)
  '........XSSVVVVVVSSX.........XCX', //  3 | tab strip top
  '........XSPPPPPPPPSX.........XCX', //  4 forehead | strip
  '........XPPPPPPPPRRX.........XCX', //  5 | strip
  '........XPBPPPPBPRWX.........CCC', //  6 brow/eyes | strip widens
  '........XPPRRRRRRRPX.......XCXCX', //  7 | order 1
  '.......XXPRRRRRRPXX........XCCCX', //  8 jaw | order 1 rule
  '......XYPPPRRRRPYYX........XCXCX', //  9 collar | order 2
  '.....XYYYYYYYYYYYYYX.......XCCCX', // 10 shoulders | order 2 rule
  '....XYYYOOOOOOOOOYYYX......XCXCX', // 11 robe upper | order 3
  '...XYYYOOOWWWWWOOOOYYX.....XCCCX', // 12 robe lit | order 3 rule
  '...XYYOOOWWWWWWWOOOYYXP....XCXCX', // 13 hand reaches strip (P) | order 4
  '..PXYOOOOOOOOOOOOOOYXPRP...XCCCX', // 14 hand grips clip (P/R) | order 4 rule
  '.PRPYOOOWWWWWWWWOOOYPPP....XXXXX', // 15 hand | chain STOPS here (Err)
  '.PPPXXNGGGGGGGGGGNXX............', // 16 belt band (N/G)
  '....XKYYOOOOOOOOOOYYKX..........', // 17 robe lower
  '....XKYYOOOWWWWWWOOYYKX.........', // 18
  '...XKKYYOOOOOOOOOOOOYYKKX.......', // 19 robe widens
  '...XKYYYOOOWWWWWWWWOOYYKX.......', // 20
  '...XKYYYOOOOOOOOOOOOYYKX........', // 21
  '...XKYYYOOOOOOOOOOOOYYKX........', // 22
  '..XKKYYYOOOOOOOOOOOOYYKKX.......', // 23 widest
  '..XKYYFFFFFFFFFFFFFFYKX.........', // 24 hem
  '..XKFFFFFFXXX..XXXFFFFKX........', // 25 leg split
  '..XXZZZZZXX......XXZZZZX........', // 26 boots
  '...XZUUUZX........XZUUZX........', // 27
  '...XXXXXXX........XXXXXX........', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF46 = function REF46({ scale = 8 }) {
  return <PixelArt grid={REF46_GRID} scale={scale} title="REF-46 The Tabkeeper" />;
};
window.REF46_GRID = REF46_GRID;
window.REF46_ROLES = ['X','K','Y','O','W','R','P','B','S','V','N','G','L','C','U','F'];
