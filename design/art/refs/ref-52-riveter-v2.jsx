// SPDX-License-Identifier: MIT
// REF-52 v2 - The Riveter on the Forge kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/riveter_idle_0.png.
//
// Kit: brass goggles, basalt apron, bare arms, glove (REF-72), bare-headed
// with dark hair and a full basalt beard. Lesson prop (iter_zip): a rivet
// hammer raised in the frame-right hand, and along the bottom edge two strips
// - one stone grey, one basalt - joined pairwise by a row of gold rivets.
//
// Legend: X coalblack  B basalt  S stone grey  P pink quartz  R dusty rose
//         W wineflesh  N antique brass  Z bronze  U bog umber
//         G old gold  L brass leaf

const REF52_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX....XXXXXX.', //  2 hair, hammer head
  '..........XBBBBBBBBBBX...XSSSBX.', //  3
  '..........XBBBBBBBBBBX...XXXZXX.', //  4
  '.........XXNNNNNNNNNNXX....XZX..', //  5 goggle strap, haft
  '.........XNXLGXNNXLGXNX....XZX..', //  6 lenses
  '.........XXNXXXNNXXXNXX...XPPRX.', //  7 raised hand
  '..........XPPPPRPPPRWX....XPRRX.', //  8
  '..........XBBBBBBBBBBX...XPRRX..', //  9 beard
  '..........XBBBBBBBBBBX...XPRRX..', // 10
  '.......XXXXXBBBBBBBBXXXXXPRRX...', // 11
  '.....XXPPPRXBBBBBBBBBXPPRRRRX...', // 12 bare shoulders
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
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 26 two strips
  '.XSSSGSSSSGSSSSGSSSSGSSSSGSSSX..', // 27 rivets pair them
  '.XBBBGBBBBGBBBBGBBBBGBBBBGBBBX..', // 28
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 29
  '................................', // 30
  '................................', // 31
];

window.REF52_V2_GRID = REF52_V2_GRID;
