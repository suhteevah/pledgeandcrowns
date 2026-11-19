// SPDX-License-Identifier: MIT
// REF-165 - Book pile, Act 9 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Five thick books stacked crooked, oxblood, cobalt, forest, crypt and
// gold, with an open one face-down on top. Floor clutter for the
// Library and the Mage's Tower. Bottom-centre anchor; solid over the
// pile.
//
// Legend: X coalblack  O oxblood  W wineflesh  Y crypt  G old gold
//         L brass leaf  E forest  H spring meadow  < cobalt deep
//         > cobalt  C parchment cream  S stone grey

const REF165_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..............XXXXXX............', //  3 open book on top
  '.............XCCCCCCX...........', //  4
  '............XCSSCCSSCX..........', //  5
  '............XCCCCCCCCX..........', //  6
  '.............XXXXXXXX...........', //  7
  '...............XXXX.............', //  8
  '......XXXXXXXXXXXXXXXXXX........', //  9 the pile
  '......XLLLLLLLLLLLLLLCCX........', // 10
  '......XGGLGGGGGGGGGGGCCX........', // 11
  '......XGGGGGGGGGGGGGGCCX........', // 12
  '......XXXXXXXXXXXXXXXXXX........', // 13
  '........XOOOOOOOOOOOOCCX........', // 14
  '........XYYLYYYYYYYYYCCX........', // 15
  '........XYYYYYYYYYYYYCCX........', // 16
  '...XXXXXXXXXXXXXXXXXXXXXX.......', // 17
  '...XHHHHHHHHHHHHHHHHHHCCX.......', // 18
  '...XEELEEEEEEEEEEEEEEECCX.......', // 19
  '...XEEEEEEEEEEEEEEEEEECCX.......', // 20
  '...XXXXXXXXXXXXXXXXXXXXXXX......', // 21
  '......X>>>>>>>>>>>>>>>>CCX......', // 22
  '......X<<L<<<<<<<<<<<<<CCX......', // 23
  '......X<<<<<<<<<<<<<<<<CCX......', // 24
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 25
  '....XWWWWWWWWWWWWWWWWWWWWCCX....', // 26
  '....XOOLOOOOOOOOOOOOOOOOOCCX....', // 27
  '....XOOOOOOOOOOOOOOOOOOOOCCX....', // 28
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF165_GRID = REF165_GRID;
