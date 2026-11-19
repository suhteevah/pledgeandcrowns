// SPDX-License-Identifier: MIT
// REF-106 - Signpost, Act 1 Hearthstone Village prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/signpost.png.
//
// A bronze post with a brass board pointing right, bog-umber lettering
// marks (not readable text), two tufts of grass at its foot. Meant for
// street corners; mirror it for a left-pointing sign. Bottom-centre
// anchor; solid only at the post.
//
// Legend: X coalblack  Z bronze  N antique brass  G old gold
//         U bog umber  E forest  H spring meadow

const REF106_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '..............XXXX..............', //  6
  '..............XNZX..............', //  7
  '....XXXXXXXXXXXXXXXXXXXXXXX.....', //  8 board
  '....XGGGGGGGGGGGGGGGGGGGGNXX....', //  9
  '....XNNNNNNNNNNNNNNNNNNNNNNXX...', // 10
  '....XNNUUUUUNUUUNUUUUUUNNNNNXX..', // 11
  '....XNNUUUNUUUUUUNNNNNNNNNNNXX..', // 12
  '....XNNNNNNNNNNNNNNNNNNNNNNXX...', // 13
  '....XZZZZZZZZZZZZZZZZZZZZNXX....', // 14
  '....XXXXXXXXXXXXXXXXXXXXXXX.....', // 15
  '..............XNZX..............', // 16
  '..............XNZX..............', // 17
  '..............XNZX..............', // 18
  '..............XNZX..............', // 19
  '..............XNZX..............', // 20
  '..............XNZX..............', // 21
  '..............XNZX..............', // 22
  '..............XNZX..............', // 23
  '..............XNZX..............', // 24
  '..............XNZX..............', // 25
  '..............XNZX..............', // 26
  '..............XNZX..............', // 27
  '............H.XNZXH.............', // 28 grass
  '...........HE.XXXEH.............', // 29
  '..........HEE....EEH............', // 30
  '................................', // 31
];

window.REF106_GRID = REF106_GRID;
