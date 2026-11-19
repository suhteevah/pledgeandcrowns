// SPDX-License-Identifier: MIT
// REF-145 - The tab slate, Act 5 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A framed basalt slate on two legs where the Tavern keeps its tally:
// the top row reads 'Ok' with five teal tally strokes, the bottom row
// 'Er' with two alarm-scarlet strokes. Act 5 is Result: every order
// ends in one column or the other. The only readable text on a prop,
// and it is two letters a row. Bottom-centre anchor; solid at the legs.
//
// Legend: X coalblack  Z bronze  N antique brass  B basalt
//         C parchment cream  I bright teal  ! alarm scarlet

const REF145_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', //  3
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', //  4
  '...XZBBBBBBBBBBBBBBBBBBBBBBZX...', //  5
  '...XZBBBBBBBBBBBBBBBBBBBBBBZX...', //  6
  '...XZBBCCCCBCBCBBIBIBIBIBBBZX...', //  7 Ok, five strokes
  '...XZBBCBBCBCBCBBIBIBIBIBBBZX...', //  8
  '...XZBBCBBCBCCBBIIIIIIIIIBBZX...', //  9
  '...XZBBCBBCBCBCBBIBIBIBIBBBZX...', // 10
  '...XZBBCCCCBCBCBBIBIBIBIBBBZX...', // 11
  '...XZBBBBBBBBBBBBBBBBBBBBBBZX...', // 12
  '...XZBBBBBBBBBBBBBBBBBBBBBBZX...', // 13
  '...XZBBCCCCBBCCBB!B!BBBBBBBZX...', // 14 Er, two strokes
  '...XZBBCBBBBCBBBB!B!BBBBBBBZX...', // 15
  '...XZBBCCCBBCCBBB!B!BBBBBBBZX...', // 16
  '...XZBBCBBBBCBBBB!B!BBBBBBBZX...', // 17
  '...XZBBCCCCBCBBBB!B!BBBBBBBZX...', // 18
  '...XZBBBBBBBBBBBBBBBBBBBBBBZX...', // 19
  '...XZBBBBBBBBBBBBBBBBBBBBBBZX...', // 20
  '...XZBBBBBBBBBBBBBBBBBBBBBBZX...', // 21
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 22
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 23
  '......XZX..............XZX......', // 24
  '......XZX..............XZX......', // 25
  '......XZX..............XZX......', // 26
  '......XZX..............XZX......', // 27
  '......XZX..............XZX......', // 28
  '......XXX..............XXX......', // 29
  '................................', // 30
  '................................', // 31
];

window.REF145_GRID = REF145_GRID;
