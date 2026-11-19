// SPDX-License-Identifier: MIT
// REF-171 - Guard statue, Act 10 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/guard_statue.png.
//
// A broad stone knight on a plinth: closed helm with a visor slit,
// square pauldrons, both gauntlets on a sword set point-down in front
// of him. The sword's guard and grip are old gold, the only gold on
// the figure. Second drawing; the first was spindly. Stand them in
// pairs at the plaza's street mouth. Bottom-centre anchor; solid over
// the plinth.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         G old gold  L brass leaf

const REF171_GRID = [
  '..............XSSX..............', //  0 helm
  '...........XXXXXXXXXX...........', //  1
  '...........XVVVVVVVVX...........', //  2
  '...........XVSSSSSSBX...........', //  3
  '...........XVSSSSSSBX...........', //  4
  '...........XVXXXXXXBX...........', //  5
  '...........XVSSXXSSBX...........', //  6
  '...........XVSSXXSSBX...........', //  7
  '....XXXXXXXXVSSXXSSBXXXXXXXX....', //  8 pauldrons
  '....XVVVVVXXXXXXXXXXXXVVVVVX....', //  9
  '....XSSSSSXSSSXGGXSSSXSSSSSX....', // 10
  '....XSSSSSXSSSXGGXSSSXSSSSSX....', // 11
  '....XSSSSSXXXXXXXXXXXXSSSSBX....', // 12 sword guard
  '....XXXXXXXLGGGGGGGGGXXXXXXX....', // 13
  '.....XVSSXXXXXXXXXXXXXXSSSX.....', // 14
  '.....XSSSXSSSSXVSXSSSSXSSSX.....', // 15 robe
  '.....XSSSXXXXXXVSXXXXXXSSSX.....', // 16
  '.....XSSSXXXXXXVSXXXXXXSSSX.....', // 17
  '.....XSSSXVVVVXVSXVVVVXSSBX.....', // 18
  '.....XXXXXSSSSXVSXSSSSXXXXX.....', // 19
  '.........XXXXXXVSXXXXXX.........', // 20
  '.........XVVSSXVSXSSBBX.........', // 21
  '.........XVVSSXVSXSSBBX.........', // 22
  '.........XVVSSXVSXSSBBX.........', // 23
  '.........XVVSSXVSXSSBBX.........', // 24
  '.........XVVSSXVSXSSBBX.........', // 25
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 26 plinth
  '....XVVVVVVVVVVVVVVVVVVVVVVX....', // 27
  '....XSSSSSSSSSSSSSSSSSSSSSSX....', // 28
  '....XBBBBBBBBBBBBBBBBBBBBBBX....', // 29
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 30
  '................................', // 31
];

window.REF171_GRID = REF171_GRID;
