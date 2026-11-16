// SPDX-License-Identifier: MIT
// REF-90 - The Choirmaster on the Throne kit (REF-86), 32x32, idle.
// APPROVED by Matt 2026-11-15, with the designer's notes applied (narrowed
// kit; the four-tined fork, which read as a pitchfork, replaced by a set of
// four brass pipes).
// Wired as game/assets/sprites/npc/choirmaster_idle_0.png.
//
// Conductor. One arm raised. Lesson prop (chunks_exact): a set of four
// brass pipes held up beside the head, each a step shorter than the last
// and bound by one band - four voices in step, four lanes taken at a time.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         N antique brass  # specular white  P pink quartz  R dusty rose
//         Z bronze  U bog umber

const REF90_GRID = [
  '................................', //  0
  '................................', //  1
  '............XXXXXXXX............', //  2
  '...........XZZZZZZZZX....L......', //  3 hair, tallest pipe
  '...........XLL#LLLLGX....LNL....', //  4 circlet
  '...........XPPPPPPPRX....LNLNL..', //  5 face
  '.........XXXPXPPPXPRXXX..LNLNLNL', //  6 eyes, four pipes
  '........XSVXPPPRRPPRXSBX.LNLNLNL', //  7
  '........XSVSXPPPPPPXSSBX.ZZZZZZZ', //  8 the band
  '........XSVSSXXXXXXSSBBX.LNLNLNL', //  9
  '.......XSVSSSVVVVVVSSSBBX.PP....', // 10 raised hand
  '.......XSVSSBCVVVVVBSSBBX<<X....', // 11 raised cuff
  '.......XSVSSBCVVVVVBSSBBXXX.....', // 12
  '.......XSVSSBCVVLVVBSSBBX.......', // 13 brass device
  '.......XSVSSBCVL#LVBSSBBX.......', // 14
  '.......XSVSSBCVVLVVBSSBBX.......', // 15
  '.......XSVSSBCVVVVVBSSBBX.......', // 16
  '.......XSVSSBCVVVVVBSSBBX.......', // 17
  '.......X<<SSBCVVVVVBSSBBX.......', // 18 one cuff at the side
  '.......XPPSSBCVVVVVBSSBBX.......', // 19 one hand at the side
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

window.REF90_GRID = REF90_GRID;
