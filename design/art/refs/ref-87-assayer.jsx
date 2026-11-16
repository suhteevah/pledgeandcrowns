// SPDX-License-Identifier: MIT
// REF-87 - The Assayer on the Throne kit (REF-86), 32x32, idle.
// APPROVED by Matt 2026-11-15, with the designer's notes applied (narrowed
// kit; the scale enlarged to span the whole chest).
// Wired as game/assets/sprites/npc/assayer_idle_0.png.
//
// Measurer. Lesson prop (prealloc): a brass balance scale whose beam runs
// shoulder to shoulder, with the left pan hanging three rows lower than the
// right - you do not guess what a thing costs, you weigh it.
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
  '........XSVSSXXXXXXSSBBX........', //  9
  '.......XSVSSSVVVVVVSSSBBX.......', // 10
  '.......XSVSSBCVVVVVBSSBBX.......', // 11
  '.......XSVNNNNNLLNNNNNBBX.......', // 12 scale beam, shoulder to shoulder
  '.......XSVBSBCVVVVVBSBBBX.......', // 13 chains
  '.......XSVBSBCVVVVVBGGGBX.......', // 14 light pan, high
  '.......XSVBSBCVVVVVBSSBBX.......', // 15
  '.......XSVBSBCVVVVVBSSBBX.......', // 16
  '.......XSGGGBCVVVVVBSSBBX.......', // 17 heavy pan, low
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

window.REF87_GRID = REF87_GRID;
