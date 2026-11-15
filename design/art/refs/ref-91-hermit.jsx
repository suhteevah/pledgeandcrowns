// SPDX-License-Identifier: MIT
// REF-91 - The Hermit on the Throne kit (REF-86), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Outpost keeper. The mantle is worn short and ragged at the edge, so far
// more of the cobalt underrobe shows; white hair and beard, a dulled
// circlet, a plain staff. Lesson prop (no_std): one small brass lantern at
// the hip that carries its own light - nothing else out here to lean on.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         N antique brass  # specular white  P pink quartz  R dusty rose
//         Z bronze  U bog umber

const REF91_GRID = [
  '................................', //  0
  '....ZZ..........................', //  1 staff
  '....Z.......XXXXXXXX............', //  2
  '....Z......XVVVVVVVVX...........', //  3 white hair
  '....Z......XNNNNNNNGX...........', //  4 dulled circlet
  '....Z......XPPPPPPPRX...........', //  5 face
  '....Z....XXXPXPPPXPRXXX.........', //  6 eyes
  '....Z...XSVXPPPRRPPRXSBX........', //  7 raised collar
  '....Z...XSVSXVVVVVVXSSBX........', //  8 beard
  '....Z..XSSVSSXVVVVXSSSBBX.......', //  9
  '....Z.XSSVSSSVVVVVVSSSSBBX......', // 10
  '....Z.XSSVSSBCVVVVVBSSSBBX......', // 11
  '....Z.XSSVSSBCVVVVVBSSSBBX......', // 12
  '....Z.XSSVSSBCVVLVVBSSSBBX......', // 13 brass device
  '....Z.XSSVSSBCVL#LVBSSSBBX......', // 14
  '....Z.XSSVSSBCVVLVVBSSSBBX......', // 15
  '....Z.XSSVSSBCVVVVVBSSSBBX......', // 16
  '....Z.XSSVSSBCVVVVVBSSSBBX......', // 17
  '....Z.XBSBBSBCVVVVVBSBBSBX......', // 18 ragged mantle edge
  '....ZPX<<><<<CVVVVV<<N<<<X......', // 19 hand on the staff
  '....Z.X<<><<<CVVVVV<NLN<<X......', // 20 lantern
  '....Z.X<<><<<CVVVVV<N#N<<X......', // 21
  '....Z.X<<><<<CVVVVV<NNN<<X......', // 22
  '....Z.X<<><<<CVVVVV<<<><<X......', // 23
  '....Z.X<<><<<CVVVVV<<<><<X......', // 24
  '....Z.X<<><<<<<<<<<<<<><<X......', // 25
  '....Z.X<<<<<<<<<<<<<<<<<<X......', // 26
  '....Z.XXXXXXXXXXXXXXXXXXXX......', // 27
  '....Z....XUUX......XUUX.........', // 28
  '....Z....XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF91_GRID = REF91_GRID;
