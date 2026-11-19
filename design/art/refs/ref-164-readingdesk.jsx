// SPDX-License-Identifier: MIT
// REF-164 - Reading desk, Act 9 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A brass-wood desk with an open book, a tall candle (one
// alarm-scarlet pixel of flame) and a closed oxblood volume.
// Bottom-centre anchor; solid over the desk.
//
// Legend: X coalblack  N antique brass  G old gold  L brass leaf
//         Z bronze  C parchment cream  S stone grey  O oxblood
//         W wineflesh  ! alarm scarlet

const REF164_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '......................!.........', //  3 candle
  '......................L.........', //  4
  '.....................XLX........', //  5
  '.....................XCX........', //  6
  '.....................XCX........', //  7
  '.....................XCX........', //  8
  '.....XXXXXXXXXXXXX...XCX........', //  9 open book
  '.....XCCCCCXCCCCCX...XCX.XXXXX..', // 10
  '.....XCSSSCXCSSSCX...XCX.XWWWX..', // 11
  '.....XCCCCCXCCCCCX..XNGNXXOOOX..', // 12
  '.....XCSSSCXCCCCCX..XXXXXXOOOX..', // 13
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 14 desk top
  '.XGGGGGGGGGGGGGGGGGGGGGGGGGGGGX.', // 15
  '.XNNNNNNNNNNNNNNNNNNNNNNNNNNNNX.', // 16
  '.XNNNNNNNNNNNNNNNNNNNNNNNNNNNNX.', // 17
  '.XZZZZZZZZZZZZZZZZZZZZZZZZZZZZX.', // 18
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 19
  '....XZZX................XZZX....', // 20
  '....XZZX................XZZX....', // 21
  '....XZZX................XZZX....', // 22
  '....XZZXXXXXXXXXXXXXXXXXXZZX....', // 23
  '....XZZXXZZZZZZZZZZZZZZXXZZX....', // 24
  '....XZZXXXXXXXXXXXXXXXXXXZZX....', // 25
  '....XZZX................XZZX....', // 26
  '....XZZX................XZZX....', // 27
  '....XZZX................XZZX....', // 28
  '....XXXX................XXXX....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF164_GRID = REF164_GRID;
