// SPDX-License-Identifier: MIT
// REF-179 - Townsfolk: girl, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Child height. Old-gold hair with a brass-leaf lock and two braids
// tied with wine ribbons, a mist-teal smock over a teal skirt.
// For anywhere a parent stands, and beside the boy.
//
// Legend: X coalblack  G old gold  L brass leaf  N antique brass
//         P pink quartz  R dusty rose  W wineflesh  M mist teal
//         I bright teal  T main teal  D deep teal
//         C parchment cream  U bog umber

const REF179_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '..........XXXXXXXXXXXX..........', //  7 head and braids
  '..........XGGGGLGGGGNX..........', //  8
  '..........XGGPPPPPPGNX..........', //  9
  '........XGXPXXPPPPXXRXGX........', // 10
  '........XGXPPXPPPPXPRXGX........', // 11
  '........XWXPPPPRRPPPWXWX........', // 12
  '........XGXPPPRRRRPPWXGX........', // 13
  '..........XPPPRWWRPPRX..........', // 14
  '...........XXPRRRRWXX...........', // 15
  '........XXXMMMIIIIMMMIXXX.......', // 16 smock
  '......XXMMMMTTTTTTTTMMMIIXX.....', // 17
  '.....XMMMMIXIITTTTTTXMMMIIX.....', // 18
  '.....XMMMIIXITTTTTTTXMMIIIX.....', // 19
  '.....XPPRXXXITTTTTTTXXXPPRX.....', // 20
  '.....XPRRX.XITTTTTTDX.XPRRX.....', // 21
  '.........XTXITTTTTTDXTX.........', // 22
  '.........XTXXXXXXXXXXTX.........', // 23
  '.........XTTTTTTTTTTTDX.........', // 24 skirt
  '.........XDDDDDDDDDDDDX.........', // 25
  '.........XXXXXXXXXXXXXX.........', // 26
  '..........XUUX....XUUX..........', // 27
  '..........XXXX....XXXX..........', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF179_GRID = REF179_GRID;
