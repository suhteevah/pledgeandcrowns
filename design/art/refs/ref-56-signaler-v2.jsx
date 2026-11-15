// SPDX-License-Identifier: MIT
// REF-56 v2 - The Signaler on the Coast kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: teal knit cap, hayfield scarf, cobalt peacoat with gold buttons
// (REF-73). Lesson prop (mpsc_channel): two signal flags on poles - a gold
// one raised high at frame-left, a teal one lowered at frame-right. One end
// sends, the other receives; the arms make the widest diagonal in the cast.
//
// Legend: X coalblack  T main teal  I bright teal  D deep teal
//         > cobalt  < cobalt deep  Q hayfield  H spring meadow
//         P pink quartz  R dusty rose  W wineflesh  S stone grey  B basalt
//         G old gold  L brass leaf  Z bronze  U bog umber

const REF56_V2_GRID = [
  '................................', //  0
  '...XXXXXX.......................', //  1 raised flag
  '...XGGLGX.......................', //  2
  '...XGGGGX...XXXXXXXX............', //  3 knit cap
  '...XXXXXX..XTTIITTTTX...........', //  4
  '...XZ.....XTTIITTTTTDX..........', //  5 pole
  '...XZ.....XDDDDDDDDDDX..........', //  6
  '...XZ.....XSPPPPPPPRSX..........', //  7
  '...XZ.....XPPXPPPPXPRX..........', //  8 eyes
  '..XPPX....XPPPPRRPPPWX..........', //  9 raised hand
  '..XRRX.....XPRRRRRRWX...........', // 10
  '...X>>X..XXQQQQQQQQQQXX.........', // 11 scarf
  '....X>>XX>>XQQHQQQQHX>><XX......', // 12
  '.....XX>>>>>XQQQXX>>>>><<XX.....', // 13
  '......X>I>>>>XQQX>>>>>><<>>X....', // 14 lowered arm
  '......X>I>>>>XQHX>>G>>><<X>>X...', // 15
  '......X>>>>>>>XX>>>>>>><<X.XPX..', // 16 lowered hand
  '......X>>>>>>>>>>G>>>>><<X.XRX..', // 17
  '......XXX>>>>>>>>>>>><<XXX.XZ...', // 18 pole
  '.........X>>>>>>>G>>><<X...XZ...', // 19
  '.........X>>>>>>>>>>><<X...XZ...', // 20
  '.........X>>>>>>>>>>><<X...XZ...', // 21
  '.........X<<<<<<<<<<<<<X...XZ...', // 22 coat hem
  '.........XXXXXXXXXXXXXXX...XZ...', // 23
  '..........XBBBBXXBBBBX.XXXXXZ...', // 24 lowered flag
  '..........XBBBX..XBBBX.XTITXZ...', // 25
  '..........XBSBX..XBSBX.XTTTXZ...', // 26
  '..........XUUUX..XUUUX.XXXXXZ...', // 27
  '.........XUUUUX..XUUUUX....XZ...', // 28
  '.........XXXXXX..XXXXXX....XX...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF56_V2_GRID = REF56_V2_GRID;
