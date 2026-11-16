// SPDX-License-Identifier: MIT
// REF-72 - Costume kit exemplar, Act 6 Iterator Forge (Smelter), 32x32.
// APPROVED by Matt 2026-11-13 (tranche 1) as a kit exemplar. Wired as game/assets/sprites/npc/smelter_idle_0.png (the Smelter's sprite, confirmed by Matt 2026-11-15).
//
// Kit rule for the Forge cast: basalt leather cap and apron, brass goggles,
// bare arms, heavy gloves. Silhouette = the broadest shoulders in the game.
// Zone roles: Basalt, Coalblack, Bronze, Stone grey. One Alarm-scarlet ember
// pixel (forge fire only), far under the 1% cap.
//
// Legend: X coalblack  B basalt  S stone grey
//         P pink quartz  R dusty rose  W wineflesh
//         N antique brass  Z bronze  U bog umber  G old gold  L brass leaf
//         ! alarm scarlet

const REF72_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX...........', //  2 leather cap
  '..........XBSSBBBBBBBX..........', //  3
  '..........XBSBBBBBBBBX..........', //  4
  '.........XXNNNNNNNNNNXX.........', //  5 goggle strap
  '.........XNXLGXNNXLGXNX.........', //  6 lenses
  '.........XXNXXXNNXXXNXX.........', //  7
  '..........XPPPPRPPPRWX..........', //  8
  '..........XPPBBBBBBRWX..........', //  9 soot beard
  '...........XBBBBBBBBX...........', // 10
  '.......XXXXXXBBBBBBXXXXXX.......', // 11
  '.....XXPPPRXBBSBBBBBBXPPRRXX....', // 12 bare shoulders
  '....XPPPRRRXBSBBBBBBBXPRRRRWX...', // 13
  '....XPPRRRWXBBBBBBBBBXPRRRWWX...', // 14
  '....XPRRWXXXBBBBBBBBBXXXRRWWX...', // 15
  '....XPRRX..XNNNNGNNNNX..XRRWX...', // 16 belt
  '....XBBBX..XBBBBBBBBBX..XBBBX...', // 17 gloves
  '....XBSBX..XBSBBBBBBBX..XBBBX...', // 18
  '.....XXX...XBSBBBBBBBX...XXX....', // 19
  '...........XBBBBBBBBBX......!...', // 20 ember
  '...........XBBBBBBBBBX..........', // 21
  '...........XBBBBBBBBBX..........', // 22
  '...........XXXXXXXXXXX..........', // 23 apron hem
  '...........XZZZZXZZZZX..........', // 24 trousers
  '...........XZZZX.XZZZX..........', // 25
  '...........XNZZX.XNZZX..........', // 26
  '...........XUUUX.XUUUX..........', // 27 boots
  '..........XUUUUX.XUUUUX.........', // 28
  '..........XXXXXX.XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF72_GRID = REF72_GRID;
