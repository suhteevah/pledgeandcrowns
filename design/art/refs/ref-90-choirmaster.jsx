// SPDX-License-Identifier: MIT
// REF-90 - The Choirmaster on the Throne kit (REF-86), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Conductor. One arm raised. Lesson prop (chunks_exact): a brass tuning fork
// with four tines held up beside the head - four voices in step, four lanes
// taken at a time.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         N antique brass  # specular white  P pink quartz  R dusty rose
//         Z bronze  U bog umber

const REF90_GRID = [
  '................................', //  0
  '................................', //  1
  '............XXXXXXXX....N.N.N.N.', //  2 four tines
  '...........XZZZZZZZZX...N.N.N.N.', //  3 hair
  '...........XLL#LLLLGX...N.N.N.N.', //  4 circlet
  '...........XPPPPPPPRX...N.N.N.N.', //  5 face
  '.........XXXPXPPPXPRXXX.NNNNNNN.', //  6 eyes, fork bar
  '........XSVXPPPRRPPRXSBX...N....', //  7 handle
  '........XSVSXPPPPPPXSSBX...N....', //  8
  '.......XSSVSSXXXXXXSSSBBX.PP....', //  9 raised hand
  '......XSSVSSSVVVVVVSSSSBBX<<X...', // 10 raised cuff
  '......XSSVSSBCVVVVVBSSSBBXXX....', // 11
  '......XSSVSSBCVVVVVBSSSBBX......', // 12
  '......XSSVSSBCVVLVVBSSSBBX......', // 13 brass device
  '......XSSVSSBCVL#LVBSSSBBX......', // 14
  '......XSSVSSBCVVLVVBSSSBBX......', // 15
  '......XSSVSSBCVVVVVBSSSBBX......', // 16
  '......XSSVSSBCVVVVVBSSSBBX......', // 17
  '......X<<VSSBCVVVVVBSSSBBX......', // 18 one cuff at the side
  '......XPPSSSBCVVVVVBSSSBBX......', // 19 one hand at the side
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

window.REF90_GRID = REF90_GRID;
