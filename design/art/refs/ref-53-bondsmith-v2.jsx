// SPDX-License-Identifier: MIT
// REF-53 v2 - The Bondsmith on the Forge kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: brass goggles, basalt apron, bare arms (REF-72), under a basalt cowl
// instead of the cap - the only hooded head in the forge. Lesson prop
// (closure_move): a small crucible of molten gold held tight against the
// chest, both forearms wrapped around it. What the closure captures, it keeps.
//
// Legend: X coalblack  B basalt  S stone grey  P pink quartz  R dusty rose
//         W wineflesh  N antique brass  Z bronze  U bog umber
//         G old gold  L brass leaf

const REF53_V2_GRID = [
  '................................', //  0
  '...........XXXXXXXXXX...........', //  1 cowl
  '..........XBBBBBBBBBBX..........', //  2
  '.........XBBSSBBBBBBBBX.........', //  3
  '.........XBSBBBBBBBBBBX.........', //  4
  '.........XBNNNNNNNNNNBX.........', //  5 goggle strap
  '.........XBXLGXNNXLGXBX.........', //  6 lenses
  '.........XBNXXXNNXXXNBX.........', //  7
  '.........XBPPPPRPPPRWBX.........', //  8
  '.........XBBPPRRRRPWBBX.........', //  9
  '..........XBBXPRRWXBBX..........', // 10
  '.......XXXXBBBXXXXBBBXXXX.......', // 11 cowl mantle
  '.....XXPPPRXBBSBBBBBBXPPRRXX....', // 12 bare shoulders
  '....XPPPRRRXBSXXXXXBBXPRRRRWX...', // 13 crucible rim
  '....XPPRRRWXBXLGGGXBBXPRRRWWX...', // 14 molten gold
  '....XPRRWXXPPXGGGGXPPXXXRRWWX...', // 15 cupped hands
  '....XPRRRRRRPXNNNNXPRRRRRRWX....', // 16 forearms wrapped round
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 17
  '...........XBSBBBBBBBX..........', // 18
  '...........XNNNNGNNNNX..........', // 19 belt
  '...........XBBBBBBBBBX..........', // 20
  '...........XBBBBBBBBBX..........', // 21
  '...........XBBBBBBBBBX..........', // 22
  '...........XXXXXXXXXXX..........', // 23 apron hem
  '...........XZZZZXZZZZX..........', // 24
  '...........XZZZX.XZZZX..........', // 25
  '...........XNZZX.XNZZX..........', // 26
  '...........XUUUX.XUUUX..........', // 27
  '..........XUUUUX.XUUUUX.........', // 28
  '..........XXXXXX.XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF53_V2_GRID = REF53_V2_GRID;
