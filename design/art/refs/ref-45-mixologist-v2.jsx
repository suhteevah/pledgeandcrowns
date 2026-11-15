// SPDX-License-Identifier: MIT
// REF-45 v2 - The Mixologist on the Tavern kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/mixologist_idle_0.png.
//
// Kit: Tavern shirtsleeves and bronze, no robe (REF-71). The bartender: dark
// hair in a bun, cream blouse, bronze apron over a long basalt skirt - the
// only skirt silhouette in the tavern. Lesson prop (option_map): a bronze
// shaker raised in the frame-left hand (the closure) and a glass in the
// frame-right hand holding what is inside the Option. Teal = 9 px, drink only.
//
// Legend: X coalblack  P pink quartz  R dusty rose  W wineflesh  B basalt
//         V aged paper  C parchment cream  G old gold  L brass leaf
//         N antique brass  Z bronze  U bog umber  T main teal  I bright teal

const REF45_V2_GRID = [
  '................................', //  0
  '............XXXX................', //  1 bun
  '...........XBBBBX...............', //  2
  '...........XXXXXXXXXX...........', //  3
  '..........XBBBBBBBBBBX..........', //  4 hair
  '..........XBPPPPPPPPBX..........', //  5
  '..........XBPXPPPPXPBX..........', //  6 eyes
  '..........XPPPPRRPPPRX..........', //  7
  '..........XPPRWWWWRPWX..........', //  8
  '...XXX.....XPRRRRRWX............', //  9 shaker cap
  '...XLX......XXRRRXX.............', // 10
  '..XNNNX...XXCCCVVVCCXX..........', // 11 shaker band
  '..XZZZX.XXCCCZZZZZZCCVXX........', // 12 apron bib
  '..XZZZXXCCCCXNZZZZZZXCCVVX......', // 13
  '..XNNNXPPCCVXNZZZZZZXCCVVVX.....', // 14 hand on the shaker
  '...XXX.XXXXXXNZZZZZZXXXPPRX.....', // 15
  '............XNZZGGZZX.XTTTX.....', // 16 glass
  '............XNZZZZZZX.XITTX.....', // 17
  '...........XXNZZZZZZXX.XTX......', // 18
  '..........XBXNZZZZZZXBX.XX......', // 19
  '..........XBXNZZZZZZXBX.........', // 20
  '.........XBBXNZZZZZZXBBX........', // 21
  '.........XBBXXXXXXXXXBBX........', // 22 apron hem
  '........XBBBBBBBBBBBBBBBX.......', // 23 long skirt
  '........XBBBBBBBBBBBBBBBX.......', // 24
  '.......XBBBBBBBBBBBBBBBBBX......', // 25
  '.......XXXXXXXXXXXXXXXXXXX......', // 26
  '..........XUUX.....XUUX.........', // 27 shoes
  '..........XXXX.....XXXX.........', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF45_V2_GRID = REF45_V2_GRID;
