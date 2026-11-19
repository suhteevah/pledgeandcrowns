// SPDX-License-Identifier: MIT
// REF-120 - Workbench, Act 6 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A brass-wood bench with a shelf, a stone-grey vice on the left and a
// bar of hot metal cooling on the right: gold to parchment cream at its
// tip, with one alarm-scarlet pixel. Bottom-centre anchor; solid over
// the bench.
//
// Legend: X coalblack  N antique brass  G old gold  L brass leaf
//         Z bronze  S stone grey  B basalt  C parchment cream
//         ! alarm scarlet

const REF120_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '................................', //  7
  '................................', //  8
  '....XXXX........................', //  9 vice
  '....XSBX........................', // 10
  '....XSBXXXX...XXXXXXXXXXXXXXX...', // 11 hot bar
  '....XSSSSBX...XLLLLLLLLCCCCCX...', // 12
  '....XSSSSBX...XGGGGGGGGGGGL!X...', // 13
  '....XXXXXXX...XXXXXXXXXXXXXXX...', // 14
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 15 bench top
  '.XLGGGGGGGGGGGGGGGGGGGGGGGGGGGX.', // 16
  '.XNNNNNNNNNNNNNNNNNNNNNNNNNNNNX.', // 17
  '.XZZZZZZZZZZZZZZZZZZZZZZZZZZZZX.', // 18
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 19
  '...XZZX..................XZZX...', // 20
  '...XZZX..................XZZX...', // 21
  '...XZZX..................XZZX...', // 22
  '...XZZXXXXXXXXXXXXXXXXXXXXZZX...', // 23 shelf
  '...XZZXNNNNNNNNNNNNNNNNNNXZZX...', // 24
  '...XZZXZZZZZZZZZZZZZZZZZZXZZX...', // 25
  '...XZZXXXXXXXXXXXXXXXXXXXXZZX...', // 26
  '...XZZX..................XZZX...', // 27
  '...XZZX..................XZZX...', // 28
  '...XXXX..................XXXX...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF120_GRID = REF120_GRID;
