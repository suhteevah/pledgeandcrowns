// SPDX-License-Identifier: MIT
// REF-144 - Round table and stools, Act 5 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A round brass-wood table on one leg with a lit candle (two
// alarm-scarlet pixels of flame), a tankard and a plate, and a stool
// at either side. Bottom-centre anchor; solid over the table.
//
// Legend: X coalblack  N antique brass  G old gold  L brass leaf
//         Z bronze  C parchment cream  V aged paper  ! alarm scarlet

const REF144_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...............!!...............', //  3 candle
  '...............LL...............', //  4
  '..............XLLX..............', //  5
  '..............XCCX..............', //  6
  '..............XCCX..............', //  7
  '..............XCCX..............', //  8
  '......CCC.....XCCX.....XXXX.....', //  9
  '.....XCCCX...XNGGNX...XVVVVX....', // 10
  '.....XGGGXXXXXXXXXXXXXXXXXXX....', // 11 table top
  '.....XGLGXGGGGGGGNNNNNNXX.......', // 12
  '.....XXXXGGGGGGNNNNNNNNNNXX.....', // 13
  '....XXGGGGGGNNNNNNNNNNNNNNXX....', // 14
  '....XXGGGNNNNNNNNNNNNNNZZZXX....', // 15
  '....XXGNNNNNNNNNNNNNNZZZZZXX....', // 16
  '.....XXNNNNNNNNNNNZZZZZZZXX.....', // 17
  '.......XXNNNNNNZZZZZZZZXX.......', // 18
  '..........XXXXXXXXXXXX..........', // 19
  '..............XZZX..............', // 20
  'XXXXXXX.......XZZX.......XXXXXXX', // 21 stools
  'XNNNNNX.......XZZX.......XNNNNNX', // 22
  'XNNNNNX.......XZZX.......XNNNNNX', // 23
  'XXXXXXX.......XZZX.......XXXXXXX', // 24
  '.XX.XX........XZZX........XX.XX.', // 25
  '.XX.XX...XXXXXXXXXXXXXX...XX.XX.', // 26
  '.XX.XX...XNNNNNNNNNNNNX...XX.XX.', // 27
  '.XX.XX...XZZZZZZZZZZZZX...XX.XX.', // 28
  '.XX.XX...XXXXXXXXXXXXXX...XX.XX.', // 29
  '................................', // 30
  '................................', // 31
];

window.REF144_GRID = REF144_GRID;
