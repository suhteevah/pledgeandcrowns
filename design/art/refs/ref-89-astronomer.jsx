// SPDX-License-Identifier: MIT
// REF-89 - The Astronomer on the Throne kit (REF-86), 32x32, idle.
// APPROVED by Matt 2026-11-15, with the designer's note applied (narrowed
// kit). Not wired yet.
//
// Chart-maker. The collar stands two pixels taller than anyone else at the
// Throne. Lesson prop (const_eval): a brass astrolabe on the tabard - the
// sky worked out ahead of time, so nobody has to work it out at night.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         # specular white  P pink quartz  R dusty rose  Z bronze  U bog umber

const REF89_GRID = [
  '................................', //  0
  '................................', //  1
  '............XXXXXXXX............', //  2
  '...........XZZZZZZZZX...........', //  3 hair
  '.........X.XLL#LLLLGX.X.........', //  4 circlet, collar tips
  '........XSXXPPPPPPPRXXBX........', //  5 tall collar
  '........XSVXPXPPPXPRXSBX........', //  6 eyes
  '........XSVXPPPRRPPRXSBX........', //  7
  '........XSVSXPPPPPPXSSBX........', //  8
  '........XSVSSXXXXXXSSBBX........', //  9
  '.......XSVSSSVVVVVVSSSBBX.......', // 10
  '.......XSVSSBCVVVVVBSSBBX.......', // 11
  '.......XSVSSBVGGGGVBSSBBX.......', // 12 astrolabe
  '.......XSVSSBGVVLVGBSSBBX.......', // 13
  '.......XSVSSBGLL#LGBSSBBX.......', // 14
  '.......XSVSSBGVVLVGBSSBBX.......', // 15
  '.......XSVSSBVGGGGVBSSBBX.......', // 16
  '.......XSVSSBCVVVVVBSSBBX.......', // 17
  '.......X<<SSBCVVVVVBSS<<X.......', // 18 cobalt cuffs
  '.......XPPSSBCVVVVVBSSPPX.......', // 19 hands
  '.......XSVSSBCVVVVVBSSBBX.......', // 20
  '.......XSVSSBCVVVVVBSSBBX.......', // 21
  '.......XBBBBBCVVVVVBBBBBX.......', // 22
  '.......X<><<<CVVVVV<<<><X.......', // 23
  '.......X<><<<CVVVVV<<<><X.......', // 24
  '.......X<><<<<<<<<<<<<><X.......', // 25
  '.......X<<<<<<<<<<<<<<<<X.......', // 26
  '.......XXXXXXXXXXXXXXXXXX.......', // 27
  '.........XUUX......XUUX.........', // 28
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF89_GRID = REF89_GRID;
