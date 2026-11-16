// SPDX-License-Identifier: MIT
// REF-86 - Costume kit exemplar, Act 10 The Throne, 32x32.
// APPROVED by Matt 2026-11-15, with the designer's note applied (narrowed
// by two pixels so the figure reads statuesque, not stout). Kit exemplar:
// not wired to any NPC: the Act 10 cast (REF-87..92) is drawn on it.
//
// Kit rule for the Throne cast: a stiff stone-grey mantle squared at the
// shoulders like carved stone, a raised collar framing the head, an
// aged-paper tabard down the front with one brass device, a cobalt-deep
// underrobe showing at the cuffs and below the mantle, a thin brass-leaf
// circlet, and a visible face.
// Silhouette = statuesque: a straight column from collar to hem, 18 px wide.
// Specular white is 2 px (cap 10), on brass only.
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
  '........XSVSSXXXXXXSSBBX........', //  9
  '.......XSVSSSVVVVVVSSSBBX.......', // 10 squared shoulders
  '.......XSVSSBCVVVVVBSSBBX.......', // 11 tabard
  '.......XSVSSBCVVVVVBSSBBX.......', // 12
  '.......XSVSSBCVVLVVBSSBBX.......', // 13 brass device
  '.......XSVSSBCVL#LVBSSBBX.......', // 14
  '.......XSVSSBCVVLVVBSSBBX.......', // 15
  '.......XSVSSBCVVVVVBSSBBX.......', // 16
  '.......XSVSSBCVVVVVBSSBBX.......', // 17
  '.......X<<SSBCVVVVVBSS<<X.......', // 18 cobalt cuffs
  '.......XPPSSBCVVVVVBSSPPX.......', // 19 hands
  '.......XSVSSBCVVVVVBSSBBX.......', // 20
  '.......XSVSSBCVVVVVBSSBBX.......', // 21
  '.......XBBBBBCVVVVVBBBBBX.......', // 22 mantle ends
  '.......X<><<<CVVVVV<<<><X.......', // 23 underrobe
  '.......X<><<<CVVVVV<<<><X.......', // 24
  '.......X<><<<<<<<<<<<<><X.......', // 25
  '.......X<<<<<<<<<<<<<<<<X.......', // 26
  '.......XXXXXXXXXXXXXXXXXX.......', // 27
  '.........XUUX......XUUX.........', // 28 shoes
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF86_GRID = REF86_GRID;
