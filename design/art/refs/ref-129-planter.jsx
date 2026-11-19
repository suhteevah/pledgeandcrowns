// SPDX-License-Identifier: MIT
// REF-129 - Planter, Act 3 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A clipped round bush in a pale stone planter, three dusty-rose
// blooms. Green for the paved districts; meant for any plaza.
// Bottom-centre anchor; solid over the planter.
//
// Legend: X coalblack  J mossbed  F pine  E forest  H spring meadow
//         R dusty rose  Z bronze  V aged paper  C parchment cream
//         S stone grey

const REF129_GRID = [
  '................................', //  0
  '................................', //  1
  '..............JJJJ..............', //  2 bush
  '...........JJJHHHHJJJ...........', //  3
  '..........JJHHHHHEEEJJ..........', //  4
  '.........JHHHHHHEEEEEEJ.........', //  5
  '........JJHHHHHEEEREEEJJ........', //  6
  '........JHHHHHEEEEEEEEEJ........', //  7
  '.......JHHHHHEEEEEEEEEEEJ.......', //  8
  '.......JHHHHREEEEHEEEEEFJ.......', //  9
  '.......JHHHEEEEEEEEEEEFFJ.......', // 10
  '.......JHHHEEEEEEEEEEFFFJ.......', // 11
  '.......JHEEEEEEEEEEEFFFFJ.......', // 12
  '.......JEEEEEHEEEEEFFFFFJ.......', // 13
  '.......JEEEEEEEEEEFFFFFFJ.......', // 14
  '........JEEEEEEFEFFFFFFJ........', // 15
  '........JJEEEEEEFFFFRFJJ........', // 16
  '.........JEEEEEFFFFFFFJ.........', // 17
  '..........JJEEFFFFFFJJ..........', // 18
  '...........JJJXZZXJJJ...........', // 19
  '..............XZZX..............', // 20
  '.......XXXXXXXXXXXXXXXXXX.......', // 21 planter
  '.......XCCCCCCCCCCCCCCCCX.......', // 22
  '.......XVVVVVVVVVVVVVVVVX.......', // 23
  '.......XXXXXXXXXXXXXXXXXX.......', // 24
  '.........XCVVVVVVVVVVSX.........', // 25
  '.........XCVVVVVVVVVVSX.........', // 26
  '.........XCVVVVVVVVVVSX.........', // 27
  '.........XSSSSSSSSSSSSX.........', // 28
  '.........XXXXXXXXXXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF129_GRID = REF129_GRID;
