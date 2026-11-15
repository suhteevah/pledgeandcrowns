// SPDX-License-Identifier: MIT
// REF-87 - The Assayer on the Throne kit (REF-86), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Measurer. Lesson prop (prealloc): a brass balance scale across the tabard
// with one pan hanging lower than the other - you do not guess what a thing
// costs, you weigh it.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         N antique brass  # specular white  P pink quartz  R dusty rose
//         Z bronze  U bog umber

const REF87_GRID = [
  '................................', //  0
  '................................', //  1
  '............XXXXXXXX............', //  2
  '...........XZZZZZZZZX...........', //  3 hair
  '...........XLL#LLLLGX...........', //  4 circlet
  '...........XPPPPPPPRX...........', //  5 face
  '.........XXXPXPPPXPRXXX.........', //  6 eyes
  '........XSVXPPPRRPPRXSBX........', //  7 raised collar
  '........XSVSXPPPPPPXSSBX........', //  8
  '.......XSSVSSXXXXXXSSSBBX.......', //  9
  '......XSSVSSSVVVVVVSSSSBBX......', // 10
  '......XSSVSSBCVVVVVBSSSBBX......', // 11
  '......XSSVSSBNNLLNNBSSSBBX......', // 12 scale beam
  '......XSSVSSBBVVVVBBSSSBBX......', // 13 chains
  '......XSSVSSBBVVVGGBSSSBBX......', // 14 light pan, high
  '......XSSVSSBBVVVVVBSSSBBX......', // 15
  '......XSSVSSBGGVVVVBSSSBBX......', // 16 heavy pan, low
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

window.REF87_GRID = REF87_GRID;
