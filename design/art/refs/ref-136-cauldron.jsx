// SPDX-License-Identifier: MIT
// REF-136 - Cauldron, Act 4 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/cauldron.png.
//
// A basalt iron cauldron on three legs, a teal brew with bright bubbles
// and a few rising motes, a small gold fire under it with one
// alarm-scarlet pixel. Bottom-centre anchor; solid over the cauldron.
//
// Legend: X coalblack  B basalt  S stone grey  T main teal
//         D deep teal  I bright teal  M mist teal
//         N antique brass  G old gold  L brass leaf  ! alarm scarlet

const REF136_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...............I................', //  3 motes
  '................................', //  4
  '...............M...I............', //  5
  '............I...................', //  6
  '...................M............', //  7
  '...........I....................', //  8
  '............M.....I.............', //  9
  '................................', // 10
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 11 rim and brew
  '...XSSXTTITTTTITTTTTITTTTXSSX...', // 12
  '...XBBXDDDDDDDDDDDDDDDDDDXBBX...', // 13
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 14
  '.....XSSSSSBBBBBBBBBBBBBBBX.....', // 15
  '....XXSSSSSBBBBBBBBBBBBBBBXX....', // 16
  '....XSSSSSSBBBBBBBBBBBBBBBBX....', // 17
  '....XSSSSSSBBBBBBBBBBBBBBBBX....', // 18
  '....XSSSSSSBBBBBBBBBBBBBBBBX....', // 19
  '....XXSSSSSBBBBBBBBBBBBBBBXX....', // 20
  '.....XSSSSSBBBBBBBBBBBBBBBX.....', // 21
  '.....XXSSSSBBBBBBBBBBBBBBXX.....', // 22
  '......XXSSSBBBBBBBBBBBBBXX......', // 23
  '.......XXSSBBBBBBBBBBBBXX.......', // 24
  '.......XXXXBBBBBBBBBBXXXX.......', // 25 legs
  '.......XBX.XXXXBBXXXX.XBX.......', // 26
  '.......XBX............XBX.......', // 27
  '.......XBX............XBX.......', // 28
  '.......XBX...N!N......XBX.......', // 29 fire
  '.......XXX..NGLGN.....XXX.......', // 30
  '.............NGN................', // 31
];

window.REF136_GRID = REF136_GRID;
