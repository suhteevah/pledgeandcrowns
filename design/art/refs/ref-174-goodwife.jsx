// SPDX-License-Identifier: MIT
// REF-174 - Townsfolk: goodwife, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A wine kerchief tied over her hair, a cream blouse, a long teal
// skirt to the ankle, a brass basket on her arm. Built on the approved
// REF-71 villager body. For the well, the market stall and the
// benches.
//
// Legend: X coalblack  W wineflesh  R dusty rose  O oxblood
//         P pink quartz  C parchment cream  V aged paper
//         T main teal  I bright teal  D deep teal  M mist teal
//         N antique brass  G old gold  Z bronze  U bog umber

const REF174_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX...........', //  2 kerchief
  '..........XRWWWWWWWWOX..........', //  3
  '..........XWWPPPPPPWOX..........', //  4
  '..........XWPPPPPPPPOX..........', //  5
  '..........XPXXPPPPXXRX..........', //  6
  '..........XPPXPPPPXPRX..........', //  7
  '..........XPPPPRRPPPWX..........', //  8
  '..........XPPPRRRRPPWX..........', //  9
  '..........XPPPRWWRPPRX..........', // 10
  '...........XXPRRRRWXX...........', // 11
  '........XXXCCCVVVVCCCVXXX.......', // 12 blouse
  '......XXCCCCTTTTTTTTCCCVVXX.....', // 13
  '.....XCCCCVXIITTTTTTXCCCVVX.....', // 14
  '.....XCCCVVXITTTTTTTXCCVVVX.....', // 15
  '.....XPPRXXXITTTTTTTXXXPPRX.....', // 16
  '.....XPRRX.XITTTTTTDX.XPXXXXXX..', // 17 basket
  '......XXX..XITTTTTTDX..XXNGGNX..', // 18
  '..........XXITTTTTTDXX..XZNNZX..', // 19
  '.........XTXITTMMTTDXTX.XXXXXX..', // 20
  '.........XTXITTTTTTDXTX.........', // 21
  '.........XTXITTTTTTDXTX.........', // 22
  '.........XTXXXXXXXXXXTX.........', // 23
  '.........XTTTTTTTTTTTDX.........', // 24 skirt
  '.........XTTITTTTTTTTDX.........', // 25
  '.........XDDDDDDDDDDDDX.........', // 26
  '.........XXXXXXXXXXXXXX.........', // 27
  '..........XUUX....XUUX..........', // 28
  '..........XXXX....XXXX..........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF174_GRID = REF174_GRID;
