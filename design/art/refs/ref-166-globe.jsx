// SPDX-License-Identifier: MIT
// REF-166 - Globe, Act 9 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A cobalt globe with forest continents in a gold meridian ring on a
// brass stand, one specular-white pixel. For the Library and the
// Mage's Tower. Bottom-centre anchor; solid at the stand.
//
// Legend: X coalblack  > cobalt  < cobalt deep  M mist teal
//         E forest  H spring meadow  G old gold  N antique brass
//         # specular white

const REF166_GRID = [
  '...............XX...............', //  0
  '.............GGGNGGGG...........', //  1 globe
  '...............XX...GGG.........', //  2
  '............XXXXXXXX..GG........', //  3
  '..........XXXMMMMM>XXX.GG.......', //  4
  '.........XXMMMMMM>>>>XX.GG......', //  5
  '........X#MMMMMM>>>>>>XX.G......', //  6
  '........XMEEMMM>>>>>>>>X.GG.....', //  7
  '.......XXMMHEM>>>>>>E>>XX.G.....', //  8
  '.......XMMMEEE>>>>>>>E>>X.G.....', //  9
  '.......XMMEE>>>>>>>>>>><X.GG....', // 10
  '......XXMMM>>>>>>>>>>><<XXGG....', // 11
  '......XXMM>>>>>>>EE>><<<XXGG....', // 12
  '.......XM>>>>>>>>>HE<<<<X.GG....', // 13
  '.......X>>>>>>>>>>EE<<<<X.G.....', // 14
  '.......XX>>>>>E>>EE<<<<XX.G.....', // 15
  '........X>>>>>EE><<<<<<X.GG.....', // 16
  '........XX>>>>>><<<<<<XX.G......', // 17
  '.........XX>>>><<<<<<XX.GG......', // 18
  '..........XXX><<<<<XXX.GG.......', // 19
  '............XXXXXXXX..GG........', // 20
  '..............XXXX..GGG.........', // 21 stand
  '.............GGGGGGGG...........', // 22
  '..............XNNX..............', // 23
  '..............XNNX..............', // 24
  '..............XNNX..............', // 25
  '.........XXXXXXXXXXXXXX.........', // 26
  '.........XGGGGGGGGGGGGX.........', // 27
  '.........XNNNNNNNNNNNNX.........', // 28
  '.........XNNNNNNNNNNNNX.........', // 29
  '.........XXXXXXXXXXXXXX.........', // 30
  '................................', // 31
];

window.REF166_GRID = REF166_GRID;
