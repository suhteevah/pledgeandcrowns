// SPDX-License-Identifier: MIT
// REF-89 - The Astronomer on the Throne kit (REF-86), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
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
  '.......XSSVSSXXXXXXSSSBBX.......', //  9
  '......XSSVSSSVVVVVVSSSSBBX......', // 10
  '......XSSVSSBCVVVVVBSSSBBX......', // 11
  '......XSSVSSBVGGGGVBSSSBBX......', // 12 astrolabe
  '......XSSVSSBGVVLVGBSSSBBX......', // 13
  '......XSSVSSBGLL#LGBSSSBBX......', // 14
  '......XSSVSSBGVVLVGBSSSBBX......', // 15
  '......XSSVSSBVGGGGVBSSSBBX......', // 16
  '......XSSVSSBCVVVVVBSSSBBX......', // 17
  '......X<<VSSBCVVVVVBSSS<<X......', // 18 cobalt cuffs
  '......XPPSSSBCVVVVVBSSSPPX......', // 19 hands
  '......XSSVSSBCVVVVVBSSSBBX......', // 20
  '......XSSVSSBCVVVVVBSSSBBX......', // 21
  '......XBBBBBBCVVVVVBBBBBBX......', // 22
  '......X<<><<<CVVVVV<<<><<X......', // 23
  '......X<<><<<CVVVVV<<<><<X......', // 24
  '......X<<><<<<<<<<<<<<><<X......', // 25
  '......X<<<<<<<<<<<<<<<<<<X......', // 26
  '......XXXXXXXXXXXXXXXXXXXX......', // 27
  '.........XUUX......XUUX.........', // 28
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF89_GRID = REF89_GRID;
