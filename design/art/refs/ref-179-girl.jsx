// SPDX-License-Identifier: MIT
// REF-179 - Townsfolk: girl, 32x32. Ambient villager, gives no mission.
// APPROVED by Matt 2026-11-19. Wired as game/assets/sprites/townsfolk/girl_idle_0.png.
//
// Child height. Old-gold hair with a brass-leaf lock and two braids
// tied with wine ribbons, a mist-teal smock over a teal skirt.
// Third drawing: the head is kept, the body is new. Her hands are
// clasped in front of her. For anywhere a parent stands, and
// beside the boy.
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
  '........XXXCMMMMMMMMIXXX........', // 16 smock
  '.......XCMMMMMMMMMMMMMIIX.......', // 17
  '.......XCMMXMMMMMMMMXMIIX.......', // 18
  '.......XCMMMXMMMMMMXMIIIX.......', // 19
  '........XXMMMMMPPRMIIIXX........', // 20 clasped hands
  '.........XXXXXXXXXXXXXX.........', // 21
  '.........XTTTTTTTTTTTDX.........', // 22 skirt
  '........XTTTTTTTTTTTTTDX........', // 23
  '........XTTTTTTTTTTTTTDX........', // 24
  '........XTITTTTTTTTTTTDX........', // 25
  '........XDDDDDDDDDDDDDDX........', // 26
  '........XXXXXXXXXXXXXXXX........', // 27
  '..........XUUX....XUUX..........', // 28
  '..........XXXX....XXXX..........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF179_GRID = REF179_GRID;
