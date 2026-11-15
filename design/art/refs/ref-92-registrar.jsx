// SPDX-License-Identifier: MIT
// REF-92 - The Registrar on the Throne kit (REF-86), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Archivist. Lesson prop (semver_caret): a tall oxblood register held in
// both hands in front of the tabard, a brass seal set in its cover. Every
// version is entered, and the seal says which ones may stand in for which.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         # specular white  P pink quartz  R dusty rose  Z bronze  U bog umber
//         O oxblood  W wineflesh

const REF92_GRID = [
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
  '......XSSVSSBXXXXXXBSSSBBX......', // 12 register
  '......XSSVSSBXWOOOXBSSSBBX......', // 13
  '......XSSVSSBXWOOOXBSSSBBX......', // 14
  '......XSSVSSBXWLGOXBSSSBBX......', // 15 brass seal
  '......XSS<<PPXWGGOXPP<<BBX......', // 16 both hands on it
  '......XSSVSSBXWOOOXBSSSBBX......', // 17
  '......XSSVSSBXWOOOXBSSSBBX......', // 18
  '......XSSVSSBXWOOOXBSSSBBX......', // 19
  '......XSSVSSBXXXXXXBSSSBBX......', // 20
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

window.REF92_GRID = REF92_GRID;
