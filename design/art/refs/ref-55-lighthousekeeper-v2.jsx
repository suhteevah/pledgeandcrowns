// SPDX-License-Identifier: MIT
// REF-55 v2 - The Lighthouse Keeper on the Coast kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: teal knit cap, hayfield scarf, cobalt peacoat with gold buttons
// (REF-73). The keeper stands at frame-left. Lesson prop (arc_mutex): a small
// striped lighthouse filling the frame-right column, one lamp at the top -
// a single light every ship shares, tended by one keeper at a time.
//
// Legend: X coalblack  T main teal  I bright teal  D deep teal
//         > cobalt  < cobalt deep  Q hayfield  H spring meadow
//         P pink quartz  R dusty rose  W wineflesh  S stone grey  B basalt
//         C parchment cream  V aged paper  G old gold  L brass leaf
//         Z bronze  U bog umber

const REF55_V2_GRID = [
  '................................', //  0
  '.........................XXX....', //  1 lamp cap
  '........................XLLLX...', //  2 lamp
  '.......XXXXXXXX.........XLGLX...', //  3 knit cap
  '......XTTIITTTTX.......XXXXXXX..', //  4 gallery
  '.....XTTIITTTTTDX.......XCCCX...', //  5 tower
  '.....XDDDDDDDDDDX.......X>>>X...', //  6 stripe
  '.....XSPPPPPPPRSX.......XCCCX...', //  7
  '.....XPPXPPPPXPRX.......XCCVX...', //  8 eyes
  '.....XPPPPRRPPPWX.......X>>>X...', //  9
  '......XPRRRRRRWX........XCCVX...', // 10
  '....XXQQQQQQQQQQXX......XCCVX...', // 11 scarf
  '..XX>>XQQHQQQQHX>><XX...X>>>X...', // 12
  '.X>>>>>XQQQXX>>>>><<X...XCCVX...', // 13
  '.X>I>>>>XQQX>>>>>><<X...XCCVX...', // 14
  '.X>I>>>>XQHX>>G>>><<X...X>>>X...', // 15 button
  '.X>>>>>>>XX>>>>>>><<X...XCCVX...', // 16
  '.XRX>>>>>>>>G>>><<XRX...XCCVX...', // 17 hands
  '.XXX>>>>>>>>>>>><<XXX...X>>>X...', // 18
  '....X>>>>>>>G>>><<X.....XCCVX...', // 19
  '....X>>>>>>>>>>><<X.....XCCVX...', // 20
  '....X>>>>>>>>>>><<X.....X>>>X...', // 21
  '....X<<<<<<<<<<<<<X.....XCCVX...', // 22 coat hem
  '....XXXXXXXXXXXXXXX.....XCCVX...', // 23
  '.....XBBBBXXBBBBX.......X>>>X...', // 24
  '.....XBBBX..XBBBX.......XCZVX...', // 25 door
  '.....XBSBX..XBSBX.......XCZVX...', // 26
  '.....XUUUX..XUUUX......XSSSSSX..', // 27 footing
  '....XUUUUX..XUUUUX.....XSSSSBX..', // 28
  '....XXXXXX..XXXXXX.....XXXXXXX..', // 29
  '................................', // 30
  '................................', // 31
];

window.REF55_V2_GRID = REF55_V2_GRID;
