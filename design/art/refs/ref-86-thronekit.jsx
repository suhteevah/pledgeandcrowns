// SPDX-License-Identifier: MIT
// REF-86 - Costume kit exemplar, Act 10 The Throne, 32x32.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit rule for the Throne cast: a stiff stone-grey mantle squared at the
// shoulders like carved stone, a raised collar framing the head, an
// aged-paper tabard down the front with one brass device, a cobalt-deep
// underrobe showing at the cuffs and below the mantle, a thin brass-leaf
// circlet, and a visible face.
// Silhouette = statuesque: a near-rectangle from shoulder to hem, the widest
// villagers in the game. Specular white is 2 px (cap 10), on brass only.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         # specular white  P pink quartz  R dusty rose  Z bronze  U bog umber

const REF86_GRID = [
  '................................', //  0
  '................................', //  1
  '............XXXXXXXX............', //  2
  '...........XZZZZZZZZX...........', //  3 hair
  '...........XLL#LLLLGX...........', //  4 circlet
  '...........XPPPPPPPRX...........', //  5 face
  '.........XXXPXPPPXPRXXX.........', //  6 eyes, collar tips
  '........XSVXPPPRRPPRXSBX........', //  7 raised collar
  '........XSVSXPPPPPPXSSBX........', //  8
  '.......XSSVSSXXXXXXSSSBBX.......', //  9
  '......XSSVSSSVVVVVVSSSSBBX......', // 10 squared shoulders
  '......XSSVSSBCVVVVVBSSSBBX......', // 11 tabard
  '......XSSVSSBCVVVVVBSSSBBX......', // 12
  '......XSSVSSBCVVLVVBSSSBBX......', // 13 brass device
  '......XSSVSSBCVL#LVBSSSBBX......', // 14
  '......XSSVSSBCVVLVVBSSSBBX......', // 15
  '......XSSVSSBCVVVVVBSSSBBX......', // 16
  '......XSSVSSBCVVVVVBSSSBBX......', // 17
  '......X<<VSSBCVVVVVBSSS<<X......', // 18 cobalt cuffs
  '......XPPSSSBCVVVVVBSSSPPX......', // 19 hands
  '......XSSVSSBCVVVVVBSSSBBX......', // 20
  '......XSSVSSBCVVVVVBSSSBBX......', // 21
  '......XBBBBBBCVVVVVBBBBBBX......', // 22 mantle ends
  '......X<<><<<CVVVVV<<<><<X......', // 23 underrobe
  '......X<<><<<CVVVVV<<<><<X......', // 24
  '......X<<><<<<<<<<<<<<><<X......', // 25
  '......X<<<<<<<<<<<<<<<<<<X......', // 26
  '......XXXXXXXXXXXXXXXXXXXX......', // 27
  '.........XUUX......XUUX.........', // 28 shoes
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF86_GRID = REF86_GRID;
