// SPDX-License-Identifier: MIT
// REF-125 - Notice board, Act 3 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/notice_board.png.
//
// A brass-wood board on two bronze posts under a small wine roof, with
// four pinned notices (parchment, aged paper, one dusty rose). Marks,
// not readable text. Bottom-centre anchor; solid only at the two posts.
//
// Legend: X coalblack  N antique brass  G old gold  Z bronze
//         W wineflesh  O oxblood  R dusty rose  C parchment cream
//         V aged paper  S stone grey  > cobalt

const REF125_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', //  4 roof
  '..XWWWWWWWWWWWWWWWWWWWWWWWWWWX..', //  5
  '..XOOOOOOOOOOOOOOOOOOOOOOOOOOX..', //  6
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', //  7
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', //  8 board
  '....XGGGGGGGGGGGGGGGGGGGGGGX....', //  9
  '....XNXXXWXXXNNNNNNNNXXGXXNX....', // 10
  '....XNXCCCCCXNXX>XXXNXCCCXNX....', // 11
  '....XNXSSSSSXNXVVVVXNXSSSXNX....', // 12
  '....XNXCCCCCXNXSSSSXNXCCCXNX....', // 13
  '....XNXSSSSCXNXVVVVXNXCCCXNX....', // 14
  '....XNXCCCCCXNXSSSSXNXXXXXNX....', // 15
  '....XNXXXXXXXNXVVVVXNNNNNNNX....', // 16
  '....XNNXXWXXXNXSSSVXNNNNNNNX....', // 17
  '....XNNXRRRRXNXVVVVXNNNNNNNX....', // 18
  '....XNNXRRRRXNXXXXXXNNNNNNNX....', // 19
  '....XNNXXXXXXNNNNNNNNNNNNNNX....', // 20
  '....XZZZZZZZZZZZZZZZZZZZZZZX....', // 21
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 22 posts
  '.....XZX................XZX.....', // 23
  '.....XZX................XZX.....', // 24
  '.....XZX................XZX.....', // 25
  '.....XZX................XZX.....', // 26
  '.....XZX................XZX.....', // 27
  '.....XZX................XZX.....', // 28
  '.....XXX................XXX.....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF125_GRID = REF125_GRID;
