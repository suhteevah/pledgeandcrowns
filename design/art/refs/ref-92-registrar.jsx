// SPDX-License-Identifier: MIT
// REF-92 - The Registrar on the Throne kit (REF-86), 32x32, idle.
// APPROVED by Matt 2026-11-15, with the designer's note applied (narrowed
// kit).
// Wired as game/assets/sprites/npc/registrar_idle_0.png.
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
  '........XSVSSXXXXXXSSBBX........', //  9
  '.......XSVSSSVVVVVVSSSBBX.......', // 10
  '.......XSVSSBCVVVVVBSSBBX.......', // 11
  '.......XSVSSBXXXXXXBSSBBX.......', // 12 register
  '.......XSVSSBXWOOOXBSSBBX.......', // 13
  '.......XSVSSBXWOOOXBSSBBX.......', // 14
  '.......XSVSSBXWLGOXBSSBBX.......', // 15 brass seal
  '.......XS<<PPXWGGOXPP<<BX.......', // 16 both hands on it
  '.......XSVSSBXWOOOXBSSBBX.......', // 17
  '.......XSVSSBXWOOOXBSSBBX.......', // 18
  '.......XSVSSBXWOOOXBSSBBX.......', // 19
  '.......XSVSSBXXXXXXBSSBBX.......', // 20
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

window.REF92_GRID = REF92_GRID;
