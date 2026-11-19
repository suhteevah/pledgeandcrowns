// SPDX-License-Identifier: MIT
// REF-172 - The Crown on its plinth, Act 10 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// The Crown the game is named for: old gold with three points, a wine
// stone and a cobalt stone, on a wine cushion on a pale stone plinth.
// One specular-white pixel. One of these, placed where the player
// passes it on the way to the dais. Bottom-centre anchor; solid over
// the plinth.
//
// Legend: X coalblack  G old gold  L brass leaf  N antique brass
//         W wineflesh  R dusty rose  > cobalt  V aged paper
//         C parchment cream  S stone grey  # specular white

const REF172_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '...........L...L....L...........', //  4 the Crown
  '...........G...GG...G...........', //  5
  '...........G...GG...G...........', //  6
  '..........XGX.XGGX.XGX..........', //  7
  '..........XGGXGGGGXGGX..........', //  8
  '..........XG#GGGGGGGGX..........', //  9
  '..........XGLGWGG>GGNX..........', // 10
  '..........XGGGGGGGGGNX..........', // 11
  '..........XNNNNNNNNNNX..........', // 12
  '..........XXXXXXXXXXXX..........', // 13
  '................................', // 14
  '................................', // 15
  '..........XXXXXXXXXXXX..........', // 16 cushion
  '..........XRRRRRRRRRRX..........', // 17
  '..........XWWWWWWWWWWX..........', // 18
  '........XXXXXXXXXXXXXXXX........', // 19 plinth
  '........XCCCCCCCCCCCCCCX........', // 20
  '........XVVVVVVVVVVVVVVX........', // 21
  '........XXXXXXXXXXXXXXXX........', // 22
  '..........XCVVVVVVVVSX..........', // 23
  '..........XCVVVVVVVVSX..........', // 24
  '..........XCVVVVVVVVSX..........', // 25
  '..........XCVVVVVVVVSX..........', // 26
  '.......XXXXXXXXXXXXXXXXXX.......', // 27
  '.......XCCCCCCCCCCCCCCCCX.......', // 28
  '.......XSSSSSSSSSSSSSSSSX.......', // 29
  '.......XXXXXXXXXXXXXXXXXX.......', // 30
  '................................', // 31
];

window.REF172_GRID = REF172_GRID;
