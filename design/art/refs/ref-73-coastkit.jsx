// SPDX-License-Identifier: MIT
// REF-73 - Costume kit exemplar, Act 7 Concurrent Coast (Dockmaster), 32x32.
// APPROVED by Matt 2026-11-13 (tranche 1) as a kit exemplar. Wired as game/assets/sprites/npc/dockmaster_idle_0.png (the Dockmaster's sprite, confirmed by Matt 2026-11-15).
//
// Kit rule for the Coast cast: teal knit cap, hayfield scarf, hip-length cobalt
// peacoat with gold buttons, basalt trousers. Silhouette = cap + boxy coat,
// legs visible. Zone roles: Cobalt deep, Cobalt, teal, Hayfield.
//
// Legend: X coalblack  T main teal  I bright teal  D deep teal
//         > cobalt  < cobalt deep  Q hayfield  H spring meadow
//         P pink quartz  R dusty rose  W wineflesh  S stone grey
//         G old gold  B basalt  U bog umber

const REF73_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '............XXXXXXXX............', //  3 knit cap
  '...........XTTIITTTTX...........', //  4
  '..........XTTIITTTTTDX..........', //  5
  '..........XDDDDDDDDDDX..........', //  6 cuff
  '..........XSPPPPPPPRSX..........', //  7
  '..........XPPXPPPPXPRX..........', //  8 eyes
  '..........XPPPPRRPPPWX..........', //  9
  '...........XPRRRRRRWX...........', // 10
  '.........XXQQQQQQQQQQXX.........', // 11 scarf
  '.......XX>>XQQHQQQQHX>><XX......', // 12
  '......X>>>>>XQQQXX>>>>><<X......', // 13
  '......X>I>>>>XQQX>>>>>><<X......', // 14 scarf tail
  '......X>I>>>>XQHX>>G>>><<X......', // 15 button
  '......X>>>>>>>XX>>>>>>><<X......', // 16
  '......XRX>>>>>>>>G>>><<XRX......', // 17 hands
  '......XXX>>>>>>>>>>>><<XXX......', // 18
  '.........X>>>>>>>G>>><<X........', // 19
  '.........X>>>>>>>>>>><<X........', // 20
  '.........X>>>>>>>>>>><<X........', // 21
  '.........X<<<<<<<<<<<<<X........', // 22 coat hem
  '.........XXXXXXXXXXXXXXX........', // 23
  '..........XBBBBXXBBBBX..........', // 24 trousers
  '..........XBBBX..XBBBX..........', // 25
  '..........XBSBX..XBSBX..........', // 26
  '..........XUUUX..XUUUX..........', // 27 boots
  '.........XUUUUX..XUUUUX.........', // 28
  '.........XXXXXX..XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF73_GRID = REF73_GRID;
