// SPDX-License-Identifier: MIT
// REF-117 - Ingot stack, Act 6 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// Five stone-grey iron bars stacked three and two on a bronze pallet,
// with one old-gold bar on top (one specular-white pixel). Bottom-centre
// anchor; solid over the stack.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         G old gold  L brass leaf  N antique brass  Z bronze
//         # specular white

const REF117_GRID = [
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
  '................................', // 10
  '................................', // 11
  '................................', // 12
  '................................', // 13
  '................................', // 14
  '................................', // 15
  '................................', // 16
  '................................', // 17
  '...........XXXXXXXXX............', // 18 the gold bar
  '...........XL#LGGGNX............', // 19
  '...........XGGGGGGNX............', // 20
  '.......XXXXXXXXXXXXXXXXX........', // 21 iron bars
  '.......XVVVSSSBXVVVSSSBX........', // 22
  '.......XSSSSSSBXSSSSSSBX........', // 23
  '...XXXXXXXXXXXXXXXXXXXXXXXXX....', // 24
  '...XVVVSSSBXVVVSSSBXVVVSSSBX....', // 25
  '...XSSSSSSBXSSSSSSBXSSSSSSBX....', // 26
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 27 pallet
  '..XNNNNNNNNNNNNNNNNNNNNNNNNNNX..', // 28
  '..XZZZZZZZZZZZZZZZZZZZZZZZZZZX..', // 29
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 30
  '................................', // 31
];

window.REF117_GRID = REF117_GRID;
