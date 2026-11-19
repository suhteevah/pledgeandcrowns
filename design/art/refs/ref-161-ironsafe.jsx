// SPDX-License-Identifier: MIT
// REF-161 - Iron safe, Act 8 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A squat basalt safe with a stone-grey door, a brass dial, a handle
// and four gold rivets, on two feet. Bottom-centre anchor; solid over
// the safe.
//
// Legend: X coalblack  B basalt  S stone grey  V aged paper
//         N antique brass  G old gold  L brass leaf  Z bronze

const REF161_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', //  5 case
  '....XSSSSSSSSSSSSSSSSSSSSSSX....', //  6
  '....XSBBBBBBBBBBBBBBBBBBBBBX....', //  7
  '....XSBXXXXXXXXXXXXXXXXXXBBX....', //  8 door
  '....XSBXVSSSSSSSSSSSSSSBXBBX....', //  9
  '....XSBXVGSSSSSSSSSSSSGBXBBX....', // 10
  '....XSBXVSSSSSSSSSSSSSSBXBBX....', // 11
  '....XSBXVSSSSSXXXXSSSSSBXBBX....', // 12 dial
  '....XSBXVSSSSXGLNNXSSSSBXBBX....', // 13
  '....XSBXVSSSXGGNNNNXXXSBXBBX....', // 14
  '....XSBXVSSSGGNNNNNNGXSBXBBX....', // 15
  '....XSBXVSSXGNNXXNNZXXSBXBBX....', // 16
  '....XSBXVSSSNNNNNNZZGXSBXBBX....', // 17
  '....XSBXVSSSXNNNNZZXXXSBXBBX....', // 18
  '....XSBXVSSSSXNNZZXSXXSBXBBX....', // 19
  '....XSBXVSSSSSXXXXSSSSSBXBBX....', // 20
  '....XSBXVSSSSSSSSSSSSSSBXBBX....', // 21
  '....XSBXVSSSSSSSSSSSSSSBXBBX....', // 22
  '....XSBXVGSSSSSSSSSSSSGBXBBX....', // 23
  '....XSBXVSSSSSSSSSSSSSSBXBBX....', // 24
  '....XSBXXXXXXXXXXXXXXXXXXBBX....', // 25
  '....XSBBBBBBBBBBBBBBBBBBBBBX....', // 26
  '....XSBBBBBBBBBBBBBBBBBBBBBX....', // 27
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 28
  '.....XBBBX............XBBBX.....', // 29
  '.....XXXXX............XXXXX.....', // 30
  '................................', // 31
];

window.REF161_GRID = REF161_GRID;
