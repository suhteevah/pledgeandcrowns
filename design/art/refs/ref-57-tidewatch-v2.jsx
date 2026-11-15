// SPDX-License-Identifier: MIT
// REF-57 v2 - The Tidewatch on the Coast kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/tidewatch_idle_0.png.
//
// Kit: hayfield scarf, cobalt peacoat with gold buttons (REF-73), under a
// wide hayfield sou'wester instead of the knit cap - the only brimmed hat on
// the coast. Lesson prop (atomic): a tide gauge post at frame-right with gold
// marks, and one hand set on the brass-leaf marker at the current level. One
// counter, moved one notch at a time.
//
// Legend: X coalblack  > cobalt  < cobalt deep  I bright teal
//         Q hayfield  H spring meadow  P pink quartz  R dusty rose
//         W wineflesh  S stone grey  B basalt  G old gold  L brass leaf
//         Z bronze  U bog umber

const REF57_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '........................XXX.....', //  2 gauge post
  '.......XXXXXXXX.........XZX.....', //  3 rain hat
  '......XQQHHQQQQX........XGX.....', //  4 mark
  '....XXQQHHQQQQQQXX......XZX.....', //  5 brim
  '....XQQQQQQQQQQQQX......XZX.....', //  6
  '.....XSPPPPPPPRSX.......XGX.....', //  7 mark
  '.....XPPXPPPPXPRX.......XZX.....', //  8 eyes
  '.....XPPPPRRPPPWX.......XZX.....', //  9
  '......XPRRRRRRWX........XGX.....', // 10 mark
  '....XXQQQQQQQQQQXX......XZX.....', // 11 scarf
  '..XX>>XQQHQQQQHX>><XX...XZX.....', // 12
  '.X>>>>>XQQQXX>>>>><<XXXX.XLX....', // 13 level marker
  '.X>I>>>>XQQX>>>>>>>>>>RPXZX.....', // 14 hand on the gauge
  '.X>I>>>>XQHX>>G>>><<XXXXXGX.....', // 15
  '.X>>>>>>>XX>>>>>>><<X...XZX.....', // 16
  '.XRX>>>>>>>>G>>><<XX....XZX.....', // 17
  '.XXX>>>>>>>>>>>><<X.....XGX.....', // 18 mark
  '....X>>>>>>>G>>><<X.....XZX.....', // 19
  '....X>>>>>>>>>>><<X.....XZX.....', // 20
  '....X>>>>>>>>>>><<X.....XGX.....', // 21 mark
  '....X<<<<<<<<<<<<<X.....XZX.....', // 22 coat hem
  '....XXXXXXXXXXXXXXX.....XZX.....', // 23
  '.....XBBBBXXBBBBX.......XGX.....', // 24 mark
  '.....XBBBX..XBBBX.......XZX.....', // 25
  '.....XBSBX..XBSBX.......XZX.....', // 26
  '.....XUUUX..XUUUX......XSSSX....', // 27 footing
  '....XUUUUX..XUUUUX.....XSSBX....', // 28
  '....XXXXXX..XXXXXX.....XXXXX....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF57_V2_GRID = REF57_V2_GRID;
