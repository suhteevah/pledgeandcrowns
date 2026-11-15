// SPDX-License-Identifier: MIT
// REF-48 v2 - The Keymaster on the Forge kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/keymaster_idle_0.png.
//
// Kit: leather cap, brass goggles, basalt apron, bare arms, heavy gloves
// (REF-72). Goggles pushed up onto the cap, clean face. Lesson prop
// (hashmap_basic): a brass key ring raised in the frame-right fist with a key
// hanging either side of the hand - a value fetched by its key.
//
// Legend: X coalblack  B basalt  S stone grey  P pink quartz  R dusty rose
//         W wineflesh  N antique brass  Z bronze  U bog umber
//         G old gold  L brass leaf

const REF48_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX.....XXX...', //  2 cap, key ring
  '..........XBSSBBBBBBBX...XNNNX..', //  3
  '.........XNXLGXNNXLGXNX.XNXXXNX.', //  4 goggles up
  '.........XXNXXXNNXXXNXX.XNX.XNX.', //  5
  '..........XPPPPPPPPRRX..XGPPRGX.', //  6 fist through the ring
  '..........XPXXPPPPXXRX..XGPRRGX.', //  7 keys either side
  '..........XPPXPPPPXPRX..XXPRRXX.', //  8
  '..........XPPPPRRPPPWX...XPRRX..', //  9 raised arm
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

window.REF48_V2_GRID = REF48_V2_GRID;
