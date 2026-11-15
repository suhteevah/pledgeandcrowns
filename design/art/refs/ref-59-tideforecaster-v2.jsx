// SPDX-License-Identifier: MIT
// REF-59 v2 - The Tideforecaster on the Coast kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: teal knit cap, hayfield scarf, cobalt peacoat with gold buttons
// (REF-73). Lesson prop (async_fn): a brass spyglass raised to one eye and
// pointing out past the frame-right edge, with a rolled chart in the other
// hand. The tide he is watching has not arrived yet; he waits for it.
//
// Legend: X coalblack  T main teal  I bright teal  D deep teal
//         > cobalt  < cobalt deep  Q hayfield  H spring meadow
//         P pink quartz  R dusty rose  W wineflesh  S stone grey  B basalt
//         C parchment cream  V aged paper  G old gold  L brass leaf
//         N antique brass  U bog umber

const REF59_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '............XXXXXXXX............', //  3 knit cap
  '...........XTTIITTTTX...........', //  4
  '..........XTTIITTTTTDX..........', //  5
  '..........XDDDDDDDDDDX..........', //  6
  '..........XSPPPPPPPRSXXXXXXXXX..', //  7 spyglass
  '..........XPPXPPPPXNNNNNNNNGLX..', //  8 raised to the eye
  '..........XPPPPRRPPPWXXXXPPXXX..', //  9 hand under the tube
  '...........XPRRRRRRWX....XRRX...', // 10
  '.........XXQQQQQQQQQQXX.X>>X....', // 11 scarf, raised sleeve
  '.......XX>>XQQHQQQQHX>><X>>X....', // 12
  '......X>>>>>XQQQXX>>>>><>>X.....', // 13
  '......X>I>>>>XQQX>>>>>><<X......', // 14
  '......X>I>>>>XQHX>>G>>><<X......', // 15 button
  '......X>>>>>>>XX>>>>>>><<X......', // 16
  '......XRX>>>>>>>>G>>><<X........', // 17
  '.....XCCX>>>>>>>>>>>><<X........', // 18 rolled chart
  '.....XCVXX>>>>>>>G>>><<X........', // 19
  '.....XCCXX>>>>>>>>>>><<X........', // 20
  '.....XXXXX>>>>>>>>>>><<X........', // 21
  '.........X<<<<<<<<<<<<<X........', // 22 coat hem
  '.........XXXXXXXXXXXXXXX........', // 23
  '..........XBBBBXXBBBBX..........', // 24
  '..........XBBBX..XBBBX..........', // 25
  '..........XBSBX..XBSBX..........', // 26
  '..........XUUUX..XUUUX..........', // 27
  '.........XUUUUX..XUUUUX.........', // 28
  '.........XXXXXX..XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF59_V2_GRID = REF59_V2_GRID;
