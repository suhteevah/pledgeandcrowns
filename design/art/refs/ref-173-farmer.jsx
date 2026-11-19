// SPDX-License-Identifier: MIT
// REF-173 - Townsfolk: farmer, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A hayfield straw hat with a wide brim, bronze hair and a short bronze
// beard, a forest tunic with a spring-meadow lit edge, bronze
// trousers. Second drawing: his own head. For the village, the
// market and the Commons.
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
  '........XXXEEEFFFFEEEFXXX.......', // 12 tunic
  '......XXEEEEEEEEEEEEEEEFFXX.....', // 13
  '.....XEEEEFXHHEEEEEEXEEEFFX.....', // 14
  '.....XEEEFFXHEEEEEEEXEEFFFX.....', // 15
  '.....XPPRXXXHEEEEEEEXXXPPRX.....', // 16
  '.....XPRRX.XHEEEEEEFX.XPRRX.....', // 17
  '......XXX..XHEEEEEEFX..XXX......', // 18
  '..........XXHEEEEEEFXX..........', // 19
  '.........XZXHEELLEEFXZX.........', // 20
  '.........XZXHEEEEEEFXZX.........', // 21
  '.........XZXHEEEEEEFXZX.........', // 22
  '.........XZXXXXXXXXXXZX.........', // 23
  '.........XZZZZZXXZZZZZX.........', // 24 trousers
  '.........XZZZZX..XZZZZX.........', // 25
  '.........XZZZZX..XZZZZX.........', // 26
  '.........XUUUUX..XUUUUX.........', // 27
  '........XUUUUUX..XUUUUUX........', // 28
  '........XXXXXXX..XXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF173_GRID = REF173_GRID;
