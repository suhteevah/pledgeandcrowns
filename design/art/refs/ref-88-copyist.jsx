// SPDX-License-Identifier: MIT
// REF-88 - The Copyist on the Throne kit (REF-86), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Scribe. Lesson prop (cow_borrow): an open book with a wine ribbon marking
// the place, and a quill held ready but not yet used. Read from the page
// that exists; pick up the quill only if something has to change.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         # specular white  P pink quartz  R dusty rose  Z bronze  U bog umber
//         W wineflesh

const REF88_GRID = [
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
  '......XSSVSSBCVVVVVBSSSCBX......', // 11 quill
  '......XSSVSSBCVVVVVBSSCBBX......', // 12
  '......XSSVSSBXXXXXXBSSCBBX......', // 13 open book
  '......XSSVSSBCCXCWCBSCSBBX......', // 14 ribbon marks the place
  '......XSSVSSBCCXCWCBSXSBBX......', // 15 nib
  '......XSSVSSBXXXXWXBSSSBBX......', // 16
  '......XSSVSSBCVVVWVBSSSBBX......', // 17 ribbon tail
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

window.REF88_GRID = REF88_GRID;
