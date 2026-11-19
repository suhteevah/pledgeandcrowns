// SPDX-License-Identifier: MIT
// REF-174 - Townsfolk: goodwife, 32x32. Ambient villager, gives no mission.
// APPROVED by Matt 2026-11-19. Wired as game/assets/sprites/townsfolk/goodwife_idle_0.png.
//
// A wine kerchief tied low over brass-brown hair and down past her
// ears, a cream blouse, a long teal skirt to the ankle. Third
// drawing: the head is kept, the body is new. She holds a brass
// basket in front of her with both hands. For the well, the market
// stall and the benches.
//
// Legend: X coalblack  W wineflesh  R dusty rose  O oxblood
//         P pink quartz  C parchment cream  V aged paper
//         T main teal  I bright teal  D deep teal
//         N antique brass  G old gold  Z bronze  U bog umber

const REF174_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX...........', //  2 kerchief
  '..........XRWWWWWWWWOX..........', //  3
  '..........XWWWWWWWWWOX..........', //  4
  '..........XWNNNPPNNNOX..........', //  5
  '.........XXPXXPPPPXXROX.........', //  6
  '.........XWPPXPPPPXPROX.........', //  7
  '.........XWPPPPRRPPPWOX.........', //  8
  '..........XPPPRRRRPPWX..........', //  9
  '..........XPPPRWWRPPRX..........', // 10
  '...........XXPRRRRWXX...........', // 11
  '........XXXCCCCCCCCCVXXX........', // 12 blouse
  '.......XCCCCCCCCCCCCCCVVX.......', // 13
  '.......XCCCXCCCCCCCCXCVVX.......', // 14
  '.......XCCCXCCCCCCCCXCVVX.......', // 15
  '.......XCCCCXCCCCCCXCVVVX.......', // 16
  '........XXCCCPXXXXRCVVXX........', // 17
  '.........XTTXNGGGGNXTDX.........', // 18 basket
  '.........XTTXZNNNNZXTDX.........', // 19
  '........XTTTXZNZZNZXTTDX........', // 20
  '........XTTTXXXXXXXXTTDX........', // 21
  '........XTTTTTTTTTTTTTDX........', // 22 skirt
  '.......XTTTTTTTTTTTTTTDDX.......', // 23
  '.......XTTTTTTTTTTTTTTDDX.......', // 24
  '.......XTITTTTTTTTTTTTDDX.......', // 25
  '.......XDDDDDDDDDDDDDDDDX.......', // 26
  '.......XXXXXXXXXXXXXXXXXX.......', // 27
  '..........XUUX....XUUX..........', // 28
  '..........XXXX....XXXX..........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF174_GRID = REF174_GRID;
