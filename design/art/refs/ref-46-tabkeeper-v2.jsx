// SPDX-License-Identifier: MIT
// REF-46 v2 - The Tabkeeper on the Tavern kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: Tavern shirtsleeves and bronze apron, no robe (REF-71). The tab clerk:
// a brass eyeshade and grey hair. Lesson prop (and_then): a long cream tab
// hanging from a gold clip down the whole frame-right edge, one order entry
// under the next. Each entry depends on the one above; the strip ends at the
// first one that fails.
//
// Legend: X coalblack  P pink quartz  R dusty rose  W wineflesh  B basalt
//         S stone grey  V aged paper  C parchment cream  G old gold
//         L brass leaf  N antique brass  Z bronze  U bog umber

const REF46_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX...........', //  2
  '..........XSSVVVVVVSSX...XNGX...', //  3 hair, tab clip
  '.........XNNNNNNNNNNNNX..XLGX...', //  4 brass eyeshade
  '..........XPPPPPPPPPRX...XCCX...', //  5 tab
  '..........XPXXPPPPXXRX...XXCX...', //  6
  '..........XPPXPPPPXPRX...XCCX...', //  7
  '..........XPPPPRRPPPWX...XCXX...', //  8
  '...........XPRRRRRRWX....XCCX...', //  9
  '............XXRRRRXX.....XXCX...', // 10
  '..........XXCCCVVVCCXX...XCCX...', // 11
  '........XXCCCZZZZZZCCVXXPXCXX...', // 12 hand holds the tab
  '.......XCCCCXNZZZZZZXCCCXXCCX...', // 13
  '.......XCCCVXNZZZZZZXCCVXXXCX...', // 14
  '.......XPPRXXNZZZZZZXXXX.XCCX...', // 15
  '.......XPRRXXNZZGGZZX....XCXX...', // 16
  '........XXX.XNZZZZZZX....XCCX...', // 17
  '...........XXNZZZZZZXX...XXCX...', // 18
  '..........XBXNZZZZZZXBX..XCCX...', // 19
  '..........XBXNZZZZZZXBX..XCXX...', // 20
  '..........XBXXXXXXXXXBX..XCCX...', // 21 apron hem
  '..........XBBBBBXBBBBBX..XXCX...', // 22 trousers
  '..........XBBBBX.XBBBBX..XCCX...', // 23
  '..........XBBBBX.XBBBBX..XCXX...', // 24
  '..........XBBBBX.XBBBBX..XCCX...', // 25
  '..........XUUUUX.XUUUUX..XXXX...', // 26 boots, tab end
  '.........XUUUUUX.XUUUUUX........', // 27
  '.........XXXXXXX.XXXXXXX........', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF46_V2_GRID = REF46_V2_GRID;
