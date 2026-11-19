// SPDX-License-Identifier: MIT
// REF-126 - Guild banner, Act 3 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/guild_banner.png.
//
// An oxblood banner with a gold ring emblem, a gold hem and fringe, on
// a bronze pole with a stone foot. The district's colours on a stick:
// place them in pairs. Bottom-centre anchor; solid only at the foot.
//
// Legend: X coalblack  O oxblood  W wineflesh  Y crypt  G old gold
//         L brass leaf  N antique brass  Z bronze  S stone grey
//         V aged paper

const REF126_GRID = [
  '.............XXXXXX.............', //  0
  '.............XLGGGX.............', //  1
  '.............XXXXXX.............', //  2
  '..............XNZX..............', //  3
  '.....XGGGGGGGGGGGGGGGGGGGGX.....', //  4 crossbar
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', //  5
  '.......XWOOOOOOOOOOOOOOYX.......', //  6
  '.......XWOOOOOOOOOOOOOOYX.......', //  7
  '.......XWOOOOOOOOOOOOOOYX.......', //  8
  '.......XWOOOOOGGGGOOOOOYX.......', //  9 emblem
  '.......XWOOOOGLGGGGOOOOYX.......', // 10
  '.......XWOOOGGGOOGGGOOOYX.......', // 11
  '.......XWOOOGGOOOOGGOOOYX.......', // 12
  '.......XWOOOGGGOOGGGOOOYX.......', // 13
  '.......XWOOOOGGGGGGOOOOYX.......', // 14
  '.......XWOOOOOGGGGOOOOOYX.......', // 15
  '.......XWOOOOOOOOOOOOOOYX.......', // 16
  '.......XWOOOOOOOOOOOOOOYX.......', // 17
  '.......XWOOOOOOOOOOOOOOYX.......', // 18
  '.......XWOOOOOOOOOOOOOOYX.......', // 19
  '.......XGGGGGGGGGGGGGGGGX.......', // 20 hem and fringe
  '.......XNNNNNNNNNNNNNNNNX.......', // 21
  '.......XXXXXXXXXXXXXXXXXX.......', // 22
  '........GXGXGXGXGXGXGXGX........', // 23
  '..............XNZX..............', // 24
  '..............XNZX..............', // 25
  '..............XNZX..............', // 26
  '...........XXXXXXXXXX...........', // 27 foot
  '...........XVVVVVVVVX...........', // 28
  '...........XSSSSSSSSX...........', // 29
  '...........XXXXXXXXXX...........', // 30
  '................................', // 31
];

window.REF126_GRID = REF126_GRID;
