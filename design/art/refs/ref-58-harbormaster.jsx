// SPDX-License-Identifier: MIT
// REF-58 - The Harbormaster, 32x32, idle, transparent bg.
// The authority of the whole harbor. A great Old-gold harbor bell hangs at
// his right (frame-right); a Bronze spyglass rides in his left hand. He
// rings the bell to OPEN the harbor, sends every boat out to its work, and
// will not ring it CLOSED until he has counted every last hull back in.
// The harbor does not shut while a boat is still out. That is `thread::scope`:
// a bounded region that spawns its workers and GUARANTEES they all finish
// and rejoin before the scope ends - no thread outlives the harbor's open
// hours. Crypt coat of office, Teal harbor sash, Old-gold bell, no magic.
// Mission: thread_scope - spawn scoped threads that all join before scope end.
//
// Legend:
//   X coalblack (outline, spyglass body, bell clapper)
//   K inkblood (coat deep shadow)
//   Y crypt (coat deep)   O oxblood (coat primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair, beard)   V aged paper (hair/beard hi)
//   Z bronze (spyglass, bell yoke, boots)   N antique brass (belt, bell rim)
//   G old gold (bell body)   L brass leaf (bell spec)
//   U bog umber (boot shadow)   F pine (hem)
//   D deep teal (sash shadow)   T main teal (harbor sash)   I bright teal (sash hi)
//   . transparent
//
// The Old-gold bell (G with L spec) is the scope's bound - rung to open,
// rung to close, never closed early. The Bronze spyglass (Z) counts the
// boats home. The Teal sash marks the harbor authority.

const REF58_GRID = [
  '................................', //  0
  '..........XXXXXXXX....XZZX......', //  1 head top | bell yoke (Z)
  '.........XSSSVVSSSX..XZGGZX.....', //  2 hair | bell crown
  '........XSSVVVVVVSSX.XGGGGX.....', //  3 | bell body (G)
  '........XSPPPPPPPPSX.XGLLGX.....', //  4 forehead | bell spec (L)
  '........XPPPPPPPPRRX.XGGGGGX....', //  5 | bell widens
  '........XPBPPPPBPRWX.NGGGGGN....', //  6 brow/eyes | bell rim (N)
  '........XPPRRRRRRRPX.NNGGNNN....', //  7 | bell mouth
  '........XSPRRRRRRPSX..XXXX......', //  8 beard begins | bell clapper (X)
  '........XSSRRRRRRSSX.PP.........', //  9 beard | spyglass hand (P)
  '.......XVSSSSSSSSVXXPRPX........', // 10 beard tip | spyglass start
  '......XYDTTTTTTTTDYXPPXZX.......', // 11 teal sash (T) | spyglass body (Z)
  '......XYDTIIIIIIITDYX.XZXX......', // 12 sash lit (I) | spyglass
  '......XYDTTTTTTTTTDYX..XZX......', // 13 | spyglass tube
  '......XYOOOOOOOOOOOYX...XZX.....', // 14 coat | spyglass end
  '......XYOOOWWWWWWOOYX....XX.....', // 15 coat lit | lens
  '......XXNGGGGGGGGNXXX...........', // 16 belt (N) buckle (G)
  '......XYYOOOOOOOOYYKX...........', // 17 coat lower
  '......XYYOOOWWWWOOYYKX..........', // 18
  '.....XKYYOOOOOOOOOYYKX..........', // 19
  '.....XKYYOOOWWWWWOOYYKX.........', // 20
  '.....XKKYOOOOOOOOOOYYKKX........', // 21 widest
  '.....XKYYFFFFFFFFFFFFYKX........', // 22 hem
  '.....XKFFFFFXXX..XXXFFFFKX......', // 23 leg split
  '.....XXZZZZXX....XXZZZZZX.......', // 24 boots
  '......XZUUZX......XZUUUZX.......', // 25
  '......XXXXXX......XXXXXXX.......', // 26 soles
  '................................', // 27
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF58 = function REF58({ scale = 8 }) {
  return <PixelArt grid={REF58_GRID} scale={scale} title="REF-58 The Harbormaster" />;
};
window.REF58_GRID = REF58_GRID;
window.REF58_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','D','T','I'];
