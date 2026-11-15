// SPDX-License-Identifier: MIT
// REF-51 v2 - The Tallywright on the Forge kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: leather cap, brass goggles, basalt apron, bare arms, glove (REF-72),
// with a stone-grey moustache. Lesson prop (iter_enumerate): a counter-stamp
// raised mid-stroke in the frame-right hand, and along the bottom edge a rail
// of four ingots already stamped with one, two, three and four gold ticks -
// each item paired with its index.
//
// Legend: X coalblack  B basalt  S stone grey  P pink quartz  R dusty rose
//         W wineflesh  N antique brass  Z bronze  U bog umber
//         G old gold  L brass leaf

const REF51_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX....XXXXX..', //  2 cap, stamp head
  '..........XBSSBBBBBBBX...XNNNX..', //  3
  '..........XBSBBBBBBBBX...XXGXX..', //  4 stamp face
  '.........XXNNNNNNNNNNXX...XZX...', //  5 goggle strap, stamp handle
  '.........XNXLGXNNXLGXNX...XZX...', //  6 lenses
  '.........XXNXXXNNXXXNXX..XPPRX..', //  7 raised hand
  '..........XPPPPRPPPRWX...XPRRX..', //  8
  '..........XPSSSSSSSRWX...XPRRX..', //  9 moustache
  '...........XPRRRRRRWX....XPRRX..', // 10
  '.......XXXXXXBBBBBBXXXXXXPRRX...', // 11
  '.....XXPPPRXBBSBBBBBBXPPRRRRX...', // 12 bare shoulders
  '....XPPPRRRXBSBBBBBBBXPRRRWX....', // 13
  '....XPPRRRWXBBBBBBBBBXXXXXX.....', // 14
  '....XPRRWXXXBBBBBBBBBX..........', // 15
  '....XPRRX..XNNNNGNNNNX..........', // 16 belt
  '....XBBBX..XBBBBBBBBBX..........', // 17 glove
  '....XBSBX..XBSBBBBBBBX..........', // 18
  '.....XXX...XBSBBBBBBBX..........', // 19
  '...........XBBBBBBBBBX..........', // 20
  '...........XBBBBBBBBBX..........', // 21
  '...........XXXXXXXXXXX..........', // 22 apron hem
  '...........XZZZZXZZZZX..........', // 23
  '...........XZZZX.XZZZX..........', // 24
  '...........XUUUX.XUUUX..........', // 25
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 26 rail of ingots
  '.XSSSSSSXSSSSSSXSSSSSSXSSSSSSX..', // 27
  '.XSSGSSSXSGSGSSXGSGSGSXGGSGGSX..', // 28 ticks: 1, 2, 3, 4
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 29
  '................................', // 30
  '................................', // 31
];

window.REF51_V2_GRID = REF51_V2_GRID;
