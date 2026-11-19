// SPDX-License-Identifier: MIT
// REF-159 - Scales, Act 8 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A heavy gold balance: a thick beam across the full width, two deep
// pans, three stacked coins in the left and an iron weight in the
// right, on a broad foot. Counting what is held: reference counts.
// Second drawing; the first was too thin to read. Bottom-centre
// anchor; solid at the foot.
//
// Legend: X coalblack  G old gold  L brass leaf  N antique brass
//         V aged paper  S stone grey  B basalt

const REF159_GRID = [
  '................................', //  0
  '................................', //  1
  '..............XXXX..............', //  2 pivot
  '.............XLLGGX.............', //  3
  '.............XLGGGX.............', //  4
  '.............XGGGNX.............', //  5
  '.............XGGNNX.............', //  6
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', //  7 beam
  '.XLLLLLLLLLLLLLLLLLLLLLLLLLLLLX.', //  8
  '.XGGGGGGGGGGGGGGGGGGGGGGGGGGGGX.', //  9
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 10
  '..V.XXXXXXV...XGNX...V..XXX..V..', // 11 coins and weight
  '..VXGLGGGNX...XGNX...V.XSVSX.V..', // 12
  '..VXXXXXXXX...XGNX...VXXXXXXXV..', // 13
  '..VXGLGGGNX...XGNX...VXSVSSBXV..', // 14
  '..VXXXXXXXX...XGNX...VXSSSSBXV..', // 15
  '..VXGLGGGNX...XGNX...VXSSSBBXV..', // 16
  '..VXXXXXXXX...XGNX...VXXXXXXXV..', // 17
  'XLLLLLLLLLLLX.XGNX.XLLLLLLLLLLLX', // 18 pans
  '.XGGGGGGGGGX..XGNX..XGGGGGGGGGX.', // 19
  '..XGGGGGGGX...XGNX...XGGGGGGGX..', // 20
  '...XGGGGGX....XGNX....XGGGGGX...', // 21
  '....XGGGX.....XGNX.....XGGGX....', // 22
  '..............XGNX..............', // 23
  '..............XGNX..............', // 24
  '........XXXXXXXXXXXXXXXX........', // 25 foot
  '........XLLLLLLLLLLLLLLX........', // 26
  '........XGGGGGGGGGGGGGGX........', // 27
  '........XGGGGGGGGGGGGGGX........', // 28
  '........XNNNNNNNNNNNNNNX........', // 29
  '........XXXXXXXXXXXXXXXX........', // 30
  '................................', // 31
];

window.REF159_GRID = REF159_GRID;
