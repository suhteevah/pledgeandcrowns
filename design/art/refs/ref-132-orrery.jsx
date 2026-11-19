// SPDX-License-Identifier: MIT
// REF-132 - Orrery, Act 4 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A brass orrery on a stand: a brass ring and a tilted bronze ring
// round a gold sun, with three small worlds on them (cobalt, teal,
// wine). Many bodies obeying one rule: Act 4 is traits. Bottom-centre
// anchor; solid at the stand.
//
// Legend: X coalblack  G old gold  L brass leaf  N antique brass
//         Z bronze  > cobalt  I bright teal  M mist teal  W wineflesh

const REF132_GRID = [
  '................................', //  0
  '..............NNNN..............', //  1 rings
  '...........NNNNNNNNNN...........', //  2
  '.........NNN........NNN.........', //  3
  '........NNN..........NNXXXX.....', //  4
  '.......NN..............XMIX.....', //  5
  '.......NN..............XIIX.....', //  6
  '......NN....ZZZZZZZZ...XXXX.....', //  7
  '......N.ZZZ..........ZZZ.N......', //  8
  '......NZ......XXXX......ZN......', //  9 sun
  '....XXXX.....XLLGGX......NN.....', // 10
  '....XM>X.....XLGGGX......NNZ....', // 11
  '....X>>X.....XGGGGX......NNZ....', // 12
  '....XXXX.....XGGGGX......NN.....', // 13
  '......NZ......XXXX......ZN......', // 14
  '......N.ZZZ..........ZZZ.N......', // 15
  '......NN....ZZZZZZZZ....NN......', // 16
  '.......NN.............XXX.......', // 17
  '.......NN.............XWX.......', // 18
  '........NNN..........NXXX.......', // 19
  '.........NNN..XXXX..NNN.........', // 20 stand
  '...........NNNNNNNNNN...........', // 21
  '..............NNNN..............', // 22
  '..............XNNX..............', // 23
  '..............XNNX..............', // 24
  '..............XNNX..............', // 25
  '..............XNNX..............', // 26
  '.........XXXXXXXXXXXXXX.........', // 27
  '.........XGGGGGGGGGGGGX.........', // 28
  '.........XNNNNNNNNNNNNX.........', // 29
  '.........XXXXXXXXXXXXXX.........', // 30
  '................................', // 31
];

window.REF132_GRID = REF132_GRID;
