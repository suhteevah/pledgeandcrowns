// SPDX-License-Identifier: MIT
// REF-156 - Coin pile, Act 8 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A heap of old-gold coins with brass-leaf glints, two loose coins and
// a small crown with two wine stones lying on top. One specular-white
// pixel. Bottom-centre anchor; solid over the heap.
//
// Legend: X coalblack  G old gold  L brass leaf  N antique brass
//         C parchment cream  W wineflesh  # specular white

const REF156_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '.............X.X.X..............', //  4 crown
  '............XGXLXGX.............', //  5
  '............XGGGGGX....XXX......', //  6
  '............XNWNWNX...XGLGX.....', //  7
  '............XXXXXXX...XGGNX.....', //  8
  '.......................XXX......', //  9
  '................................', // 10
  '..............XXXX..............', // 11
  '..............XLNX..............', // 12 coins
  '..............XGXX..............', // 13
  '.............XLGGNX.............', // 14
  '............X#GNGLGX............', // 15
  '...........XGLGGGGNXX...........', // 16
  '..........XNGGXGLGGGGX..........', // 17
  '..........XGLGNGGGXLGX..........', // 18
  '.........XGGNXGLGNGGGGX.........', // 19
  '........XGNLGGGNGXLGNGGX........', // 20
  '.......XNGGGXNLGGGNGGLGNX.......', // 21
  '......XXGGLNGGGGNLGGCNGGLX......', // 22
  '......XGGNGXGLNGGGGNLGGGNX......', // 23
  '....XXXNGLGGNGGXLNGGGGNLXGX.....', // 24
  '...XGLGXGCNGLGGNGGGLNGGGGNLX....', // 25
  '...XGGNXLGGGGNXLGGNGGGLNGGGGX...', // 26
  '..XGXXXGGXGLGGGGNGLGGNGGGLNXGX..', // 27
  '.XGGNGGLGNGGGXLGGGGNGLXGNGGGLNX.', // 28
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 29
  '................................', // 30
  '................................', // 31
];

window.REF156_GRID = REF156_GRID;
