// SPDX-License-Identifier: MIT
// REF-153 - Signal mast, Act 7 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A bronze mast with a gold truck and two strings of four signal
// pennants running down and out from the masthead: wine, cobalt, gold
// and forest on the left, the same in reverse on the right. Messages
// passed between ships: Act 7 teaches channels. Second drawing; the
// first had square flags that read as hanging boxes. Bottom-centre
// anchor; solid at the foot.
//
// Legend: X coalblack  Z bronze  N antique brass  S stone grey
//         V aged paper  W wineflesh  R dusty rose  > cobalt
//         M mist teal  G old gold  L brass leaf  E forest
//         H spring meadow

const REF153_GRID = [
  '..............XLGX..............', //  0 truck
  '..............LGGG..............', //  1
  '..............GGGN..............', //  2
  '..............XGNX..............', //  3
  '.........XXXXXXNZXXXXXX.........', //  4 pennants
  '.........XRWWXXNZXXHEEX.........', //  5
  '.........XWWWXXNZXXEEEX.........', //  6
  '..........XWX.XNZX.XEX..........', //  7
  '...........X..XNZX..X...........', //  8
  '......XXXXX...XNZX...XXXXX......', //  9
  '......XM>>X...XNZX...XLGGX......', // 10
  '......X>>>X...XNZX...XGGGX......', // 11
  '.......X>X....XNZX....XGX.......', // 12
  '........X.....XNZX.....X........', // 13
  '...XXXXX......XNZX......XXXXX...', // 14
  '...XLGGX......XNZX......XM>>X...', // 15
  '...XGGGX......XNZX......X>>>X...', // 16
  '....XGX.......XNZX.......X>X....', // 17
  '.....X........XNZX........X.....', // 18
  'XXXXX.........XNZX.........XXXXX', // 19
  'XHEEX.........XNZX.........XRWWX', // 20
  'XEEEX.........XNZX.........XWWWX', // 21
  '.XEX..........XNZX..........XWX.', // 22
  '..X...........XNZX...........X..', // 23
  '..............XNZX..............', // 24
  '..............XNZX..............', // 25
  '..............XNZX..............', // 26
  '..........XXXXXXXXXXXX..........', // 27 foot
  '..........XVVVVVVVVVVX..........', // 28
  '..........XSSSSSSSSSSX..........', // 29
  '..........XXXXXXXXXXXX..........', // 30
  '................................', // 31
];

window.REF153_GRID = REF153_GRID;
