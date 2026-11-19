// SPDX-License-Identifier: MIT
// REF-124 - Ledger desk, Act 3 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/ledger_desk.png.
//
// A brass-wood desk with an open ledger (parchment pages, stone-grey
// ruled lines), a cobalt inkpot and a quill. Where records are kept:
// Act 3 is structs. Bottom-centre anchor; solid over the desk.
//
// Legend: X coalblack  N antique brass  G old gold  Z bronze
//         C parchment cream  V aged paper  S stone grey
//         < cobalt deep  > cobalt

const REF124_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '..........................VX....', //  5 quill
  '..........................VX....', //  6
  '.........................VX.....', //  7
  '.........................VX.....', //  8
  '.......XXXXXXXXXXXXXX...VX......', //  9 ledger
  '.......XCCCCCXXCCCCCX...VX......', // 10
  '.......XCSSSCXXCSSSCX.XXXX......', // 11
  '.......XCCCCCXXCCCCCX.X><X......', // 12
  '.......XCSSSCXXCSSSCX.X<<X......', // 13
  '.......XVVVVVXXVVVVVX.X<<X......', // 14
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 15 desk top
  '....XGGGGGGGGGGGGGGGGGGGGGGX....', // 16
  '....XNNNNNNNNNNNNNNNNNNNNNNX....', // 17
  '....XZZZZZZZZZZZZZZZZZZZZZZX....', // 18
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 19
  '......XZX..............XZX......', // 20
  '......XZX..............XZX......', // 21
  '......XZX..............XZX......', // 22
  '......XZXXXXXXXXXXXXXXXXZX......', // 23
  '......XZXZZZZZZZZZZZZZZXZX......', // 24
  '......XZXXXXXXXXXXXXXXXXZX......', // 25
  '......XZX..............XZX......', // 26
  '......XZX..............XZX......', // 27
  '......XZX..............XZX......', // 28
  '......XXX..............XXX......', // 29
  '................................', // 30
  '................................', // 31
];

window.REF124_GRID = REF124_GRID;
