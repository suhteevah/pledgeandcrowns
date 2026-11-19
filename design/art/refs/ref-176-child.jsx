// SPDX-License-Identifier: MIT
// REF-176 - Townsfolk: boy, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Four pixels shorter than an adult with the same size head, a mop
// of bronze hair with a brass-lit lock, a dusty-rose tunic, cobalt
// trousers. For anywhere a parent stands.
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
  '..........XPXXPPPPXXRX..........', // 10
  '..........XPPXPPPPXPRX..........', // 11
  '..........XPPPPRRPPPWX..........', // 12
  '..........XPPPRRRRPPWX..........', // 13
  '..........XPPPRWWRPPRX..........', // 14
  '...........XXPRRRRWXX...........', // 15
  '........XXXRRRWWWWRRRWXXX.......', // 16 tunic
  '......XXRRRRRRRRRRRRRRRWWXX.....', // 17
  '.....XRRRRWXRRRRRRRRXRRRWWX.....', // 18
  '.....XRRRWWXRRRRRRRRXRRWWWX.....', // 19
  '.....XPPRXXXRRRRRRRRXXXPPRX.....', // 20
  '.....XPRRX.XRRRRRRRWX.XPRRX.....', // 21
  '.........X<XRRRRRRRWX<X.........', // 22
  '.........X<XXXXXXXXXX<X.........', // 23
  '.........X<<<<<XX<<<<<X.........', // 24 trousers
  '.........X<<<<X..X<<<<X.........', // 25
  '.........X<<<<X..X<<<<X.........', // 26
  '.........XUUUUX..XUUUUX.........', // 27
  '........XUUUUUX..XUUUUUX........', // 28
  '........XXXXXXX..XXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF176_GRID = REF176_GRID;
