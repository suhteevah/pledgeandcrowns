// SPDX-License-Identifier: MIT
// REF-143 - Tavern sign, Act 5 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A bronze post with an iron arm and a hanging brass board showing a
// foaming tankard. For street mouths, so the district announces itself.
// Bottom-centre anchor (the post is at the left); solid at the foot.
//
// Legend: X coalblack  Z bronze  N antique brass  G old gold
//         L brass leaf  B basalt  S stone grey  V aged paper
//         C parchment cream

const REF143_GRID = [
  '................................', //  0
  '................................', //  1
  '.....XXXX.......................', //  2
  '.....XNZX.......................', //  3
  '.....XXXXXXXXXXXXXXXXXXXXXXX....', //  4 iron arm
  '.....XBBBBBBBBBBBBBBBBBBBBBX....', //  5
  '.....XXXXXXXXXXXXXXXXXXXXXXX....', //  6
  '.....XNZX.....B.........B.......', //  7
  '.....XNZX....BB.........B.......', //  8
  '.....XNZX..XXXXXXXXXXXXXXXXX....', //  9 board
  '.....XNZX..XGGGGGGGGGGGGGGGX....', // 10
  '.....XNZX.BXNNNNNNNNNNNNNNNX....', // 11
  '.....XNZXB.XNNNNCCCCCCNNNNNX....', // 12
  '.....XNZX..XNNNXCCCCCCXNNNNX....', // 13
  '.....XNZX..XNNNXGGGGGGXXNNNX....', // 14
  '.....XNZX..XNNNXGLGGGGXNNNNX....', // 15
  '.....XNZX..XNNNXGLGGGGXXNNNX....', // 16
  '.....XNZX..XNNNXGGGGGGXNNNNX....', // 17
  '.....XNZX..XNNNXGGGGGGXNNNNX....', // 18
  '.....XNZX..XNNNNXXXXXXNNNNNX....', // 19
  '.....XNZX..XNNNNNNNNNNNNNNNX....', // 20
  '.....XNZX..XNNNNNNNNNNNNNNNX....', // 21
  '.....XNZX..XZZZZZZZZZZZZZZZX....', // 22
  '.....XNZX..XXXXXXXXXXXXXXXXX....', // 23
  '.....XNZX.......................', // 24
  '.....XNZX.......................', // 25
  '.....XNZX.......................', // 26
  '..XXXXXXXXXX....................', // 27 foot
  '..XVVVVVVVVX....................', // 28
  '..XSSSSSSSSX....................', // 29
  '..XXXXXXXXXX....................', // 30
  '................................', // 31
];

window.REF143_GRID = REF143_GRID;
