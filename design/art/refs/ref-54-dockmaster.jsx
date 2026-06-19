// SPDX-License-Identifier: MIT
// REF-54 - The Dockmaster, 32x32, idle, transparent bg.
// Stands at the head of the pier and DISPATCHES dock crews to work in
// parallel: one arm flung out POINTING down the quay, the other holding
// a Bronze whistle to his lips, a clipboard manifest tucked at his hip.
// He does not haul cargo himself - he spawns the workers and lets them
// run, each crew laboring at once on its own berth. That is `thread::spawn`:
// you point, you blow the whistle, and a new thread of work peels off and
// runs concurrently while you keep dispatching. Crypt keeper's coat with
// a Teal harbor sash (coastal house), Old-gold whistle, no magic.
// Mission: thread_spawn - launch a worker with thread::spawn.
//
// Legend:
//   X coalblack (outline, whistle mouth, clipboard clip)
//   K inkblood (coat deep shadow)
//   Y crypt (coat deep)   O oxblood (coat primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi, manifest line)
//   Z bronze (whistle body, boots)   N antique brass (whistle band, belt)
//   G old gold (whistle bell, clip)   L brass leaf (whistle spec)
//   U bog umber (boot shadow)   F pine (hem)
//   D deep teal (sash shadow)   T main teal (harbor sash)   I bright teal (sash hi)
//   C parchment cream (manifest page)
//   . transparent
//
// The Old-gold whistle (Z->G->L) at his lips is the spawn call; the
// pointing Pink-quartz arm flung to frame-left launches a worker down
// the pier. The Teal sash marks him coastal crew, not inland guild.

const REF54_GRID = [
  '................................', //  0
  '..........XXXXXXXX..............', //  1 head top
  '.........XSSSVVSSSX.............', //  2 hair
  '........XSSVVVVVVSSX............', //  3
  '........XSPPPPPPPPSX............', //  4 forehead
  '........XPPPPPPPPRRX............', //  5
  '........XPBPPPPBPRWX............', //  6 brow / eyes
  '........XPPRRRRRRGGX............', //  7 whistle (G) at lips, frame-right
  '........XXPRRRRRXGLGX...........', //  8 jaw + whistle bell (G/L spec)
  '.......XYPPPRRRPXZGZX...........', //  9 collar + whistle body (Z)
  '.......XYDTTTTTTDYXNX...........', // 10 teal sash (T) over shoulders
  '......XYDTIIIIIITDYYX...........', // 11 sash lit (I)
  '....PRPYDTTTTTTTTDYYX...........', // 12 pointing arm (P) frame-left start
  '...PPRPXYOOOOOOOOOYYX...........', // 13 arm flung out | coat upper
  '..PPRPXYYOOOWWWOOOOYYX..........', // 14 hand points (P) | coat lit
  '...XXXXYOOOOOOOOOOOOYX..........', // 15
  '......XYOOOWWWWWOOOOYX..........', // 16
  '......XXNGGGGGGGGNXXX...........', // 17 belt (N) buckle (G)
  '......XYYOOOOOOOOYYKX.XGX.......', // 18 coat lower | clipboard clip (G)
  '......XYYOOOWWWWOOYYKX.XCX......', // 19 | manifest top (C)
  '.....XKYYOOOOOOOOOYYKX.XVX......', // 20 | manifest line (V)
  '.....XKYYOOOWWWWWOOYYKX.XCX.....', // 21 | manifest page
  '.....XKKYOOOOOOOOOOYYKKX.XVX....', // 22 widest | manifest line
  '.....XKYYFFFFFFFFFFFFYKX.XXX....', // 23 hem | clipboard foot
  '.....XKFFFFFXXX..XXXFFFFKX......', // 24 leg split
  '.....XXZZZZXX....XXZZZZZX.......', // 25 boots
  '......XZUUZX......XZUUUZX.......', // 26
  '......XXXXXX......XXXXXXX.......', // 27 soles
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF54 = function REF54({ scale = 8 }) {
  return <PixelArt grid={REF54_GRID} scale={scale} title="REF-54 The Dockmaster" />;
};
window.REF54_GRID = REF54_GRID;
window.REF54_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','D','T','I','C'];
