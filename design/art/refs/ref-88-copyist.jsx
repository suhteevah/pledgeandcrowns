// SPDX-License-Identifier: MIT
// REF-88 - The Copyist on the Throne kit (REF-86), 32x32, idle.
// APPROVED by Matt 2026-11-15, with the designer's note applied (narrowed
// kit). Not wired yet.
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
  '........XSVSSXXXXXXSSBBX........', //  9
  '.......XSVSSSVVVVVVSSSBBX.......', // 10
  '.......XSVSSBCVVVVVBSSCBX.......', // 11 quill
  '.......XSVSSBCVVVVVBSCBBX.......', // 12
  '.......XSVSSBXXXXXXBSCBBX.......', // 13 open book
  '.......XSVSSBCCXCWCBCSBBX.......', // 14 ribbon marks the place
  '.......XSVSSBCCXCWCBXSBBX.......', // 15 nib
  '.......XSVSSBXXXXWXBSSBBX.......', // 16
  '.......XSVSSBCVVVWVBSSBBX.......', // 17 ribbon tail
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

window.REF88_GRID = REF88_GRID;
