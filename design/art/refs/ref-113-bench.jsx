// SPDX-License-Identifier: MIT
// REF-113 - Bench, The Commons district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/bench.png.
//
// A brass-wood bench on bronze legs, seen from the front: a back rail
// and a seat. Bottom-centre anchor; solid over the seat.
//
// Legend: X coalblack  N antique brass  G old gold  L brass leaf
//         Z bronze

const REF113_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '................................', //  7
  '................................', //  8
  '................................', //  9
  '.....XXX................XXX.....', // 10
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 11 back rail
  '...XGGGGGGGGGGGGGGGGGGGGGGGGX...', // 12
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 13
  '...XZZZZZZZZZZZZZZZZZZZZZZZZX...', // 14
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 15
  '.....XZX................XZX.....', // 16
  '.....XZX................XZX.....', // 17
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 18 seat
  '...XLGGGGGGGGGGGGGGGGGGGGGGGX...', // 19
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 20
  '...XZZZZZZZZZZZZZZZZZZZZZZZZX...', // 21
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 22
  '.....XZX................XZX.....', // 23
  '.....XZX................XZX.....', // 24
  '.....XZX................XZX.....', // 25
  '.....XZX................XZX.....', // 26
  '.....XZX................XZX.....', // 27
  '.....XXX................XXX.....', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF113_GRID = REF113_GRID;
