// SPDX-License-Identifier: MIT
// REF-102 - Tool rack, Act 6 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/tool_rack.png.
//
// A bronze-wood frame with a brass-lit top beam. A hammer (brass handle,
// stone-grey head) and a pair of coalblack tongs hang from coalblack pins.
// Open between the posts, so the ground shows through. Bottom-centre
// anchor; solid only at the two feet.
//
// Legend: X coalblack  N antique brass  Z bronze  U bog umber
//         S stone grey  B basalt (hammer cheek)

const REF102_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', //  4 top beam
  '....XNNNNNNNNNNNNNNNNNNNNNNX....', //  5
  '....XZZZZZZZZZZZZZZZZZZZZZZX....', //  6
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', //  7
  '.....XZX....X......X.X..XZX.....', //  8 pins
  '.....XZX....N......X.X..XZX.....', //  9
  '.....XZX....N......X.X..XZX.....', // 10
  '.....XZX....N......X.X..XZX.....', // 11
  '.....XZX....N.......X...XZX.....', // 12
  '.....XZX....N......X.X..XZX.....', // 13
  '.....XZX....N.....X...X.XZX.....', // 14
  '.....XZX..XXXXX...X...X.XZX.....', // 15 hammer head
  '.....XZX..XSSXX....X.X..XZX.....', // 16
  '.....XZX..XSSXX.........XZX.....', // 17
  '.....XZX..XXXXX.........XZX.....', // 18
  '.....XZX................XZX.....', // 19
  '.....XZX................XZX.....', // 20
  '.....XZX................XZX.....', // 21
  '.....XZX................XZX.....', // 22
  '.....XZX................XZX.....', // 23
  '.....XZX................XZX.....', // 24
  '.....XZX................XZX.....', // 25
  '.....XZX................XZX.....', // 26
  '.....XZX................XZX.....', // 27
  '....XXXXX..............XXXXX....', // 28 feet
  '....XUUUX..............XUUUX....', // 29
  '....XXXXX..............XXXXX....', // 30
  '................................', // 31
];

window.REF102_GRID = REF102_GRID;
