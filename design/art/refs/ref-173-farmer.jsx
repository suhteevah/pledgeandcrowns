// SPDX-License-Identifier: MIT
// REF-173 - Townsfolk: farmer, 32x32. Ambient villager, gives no mission.
// APPROVED by Matt 2026-11-19. Wired as game/assets/sprites/townsfolk/farmer_idle_0.png.
//
// A hayfield straw hat with a wide brim, bronze hair and a short bronze
// beard, a forest tunic with a spring-meadow lit edge, bronze
// trousers. Third drawing: the head is kept, the body is new. He
// stands with his fists on his hips and his feet apart. For the
// village, the market and the Commons.
//
// Legend: X coalblack  Q hayfield  L brass leaf  P pink quartz
//         R dusty rose  W wineflesh  E forest  F pine
//         H spring meadow  Z bronze  U bog umber

const REF173_GRID = [
  '...........XXXXXXXXXX...........', //  0 straw hat
  '..........XQQQQQQQQQLX..........', //  1
  '.......XQQQQQQQQQQQQQQQLX.......', //  2
  '.......XXXXXXXXXXXXXXXXXX.......', //  3
  '..........XZZPPPPPPZUX..........', //  4
  '..........XZPPPPPPPPUX..........', //  5
  '..........XPXXPPPPXXRX..........', //  6
  '..........XPPXPPPPXPRX..........', //  7
  '..........XPPPPRRPPPWX..........', //  8
  '..........XPZZRRRRZZWX..........', //  9 beard
  '..........XPZZZWWZZZRX..........', // 10
  '...........XXPRRRRWXX...........', // 11
  '........XXXHEEEEEEEEFXXX........', // 12 tunic
  '......XXHEEEEEEEEEEEEEFFXX......', // 13
  '.....XHEEXXEEEEEEEEEEXXEFFX.....', // 14
  '....XHEEX.XEEEEEEEEEEX.XEFFX....', // 15
  '....XHEEX.XEEEEEEEEEFX.XEFFX....', // 16
  '.....XEEXXXZZZZLLZZZZXXXEFX.....', // 17
  '......XXPPXEEEEEEEEEFXPRXX......', // 18 fists on hips
  '........XXXEEEEEEEEEFXXX........', // 19
  '..........XEEEEEEEEEFX..........', // 20
  '..........XXXXXXXXXXXX..........', // 21
  '..........XZZZZZZZZZZX..........', // 22 trousers
  '..........XZZZZXXZZZZX..........', // 23
  '.........XZZZZX..XZZZZX.........', // 24
  '.........XZZUUX..XZZUUX.........', // 25
  '.........XZZUUX..XZZUUX.........', // 26
  '.........XUUUUX..XUUUUX.........', // 27
  '........XUUUUUX..XUUUUUX........', // 28
  '........XXXXXXX..XXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF173_GRID = REF173_GRID;
