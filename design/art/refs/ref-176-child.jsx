// SPDX-License-Identifier: MIT
// REF-176 - Townsfolk: boy, 32x32. Ambient villager, gives no mission.
// APPROVED by Matt 2026-11-19. Not wired yet.
//
// Four pixels shorter than an adult with the same size head, a mop
// of bronze hair with a brass-lit lock, a dusty-rose tunic, cobalt
// trousers. Third drawing: the head is kept, the body is new. One
// arm is up in a wave, the other at his side. For anywhere a
// parent stands.
//
// Legend: X coalblack  Z bronze  N antique brass  U bog umber
//         P pink quartz  R dusty rose  W wineflesh  L brass leaf
//         < cobalt deep

const REF176_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '..........XXXXXXXXXXXX..........', //  7 head
  '..........XZZZZNZZZZUX..........', //  8
  '..........XZZPPPPPZZUX..........', //  9
  '..........XPXXPPPPXXRX.XXXX.....', // 10
  '..........XPPXPPPPXPRX.XPRX.....', // 11 waving hand
  '..........XPPPPRRPPPWX.XPRX.....', // 12
  '..........XPPPRRRRPPWX.XXXX.....', // 13
  '..........XPPPRWWRPPRX.XRWX.....', // 14
  '...........XXPRRRRWXX..XRWX.....', // 15
  '........XXXRRRRRRRRRWXRRRWX.....', // 16 tunic
  '.......XRRRRRRRRRRRRRRRRWXX.....', // 17
  '.......XRRRXRRRRRRRRXXXXX.......', // 18
  '.......XRRRXZZZLLZZZX...........', // 19
  '.......XPPRXRRRRRRRWX...........', // 20
  '........XXXXRRRRRRRWX...........', // 21
  '...........XXXXXXXXXX...........', // 22
  '...........X<<<<<<<<X...........', // 23 trousers
  '...........X<<<XX<<<X...........', // 24
  '...........X<<<XX<<<X...........', // 25
  '...........X<<<XX<<<X...........', // 26
  '...........XUUUXXUUUX...........', // 27
  '..........XUUUUXXUUUUX..........', // 28
  '..........XXXXXXXXXXXX..........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF176_GRID = REF176_GRID;
