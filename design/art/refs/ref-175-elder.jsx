// SPDX-License-Identifier: MIT
// REF-175 - Townsfolk: elder, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-19. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Bald on top with white hair at the sides and a long aged-paper
// beard that falls to a point on his chest. Third drawing: the head
// is kept, the body is new. A long bronze coat to the shin, one arm
// down, the other out to a brass walking stick that he leans on.
// For benches and the well.
//
// Legend: X coalblack  V aged paper  S stone grey  P pink quartz
//         R dusty rose  W wineflesh  Z bronze  N antique brass
//         U bog umber

const REF175_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...........XXXXXXXXXX...........', //  3
  '..........XPPPPPPPPPRX..........', //  4
  '..........XVPPPPPPPPVX..........', //  5 white side hair
  '..........XVXXPPPPXXVX..........', //  6
  '..........XVPXPPPPXPVX..........', //  7
  '..........XVVPPRRPPVSX..........', //  8 beard
  '..........XVVVRRRRVVSX..........', //  9
  '..........XVVVVWWVVVSX..........', // 10
  '..........XVVVVVVVVVSX..........', // 11
  '........XXXXVVVVVVVVSXXX........', // 12
  '.......XNZZZXVVVVVVSXZUUX.......', // 13
  '.......XNZZXZXVVVVSXXZUUX.......', // 14
  '.......XNZZXZZXVVSXZXZUUXX......', // 15
  '.......XNZZXZZZZZZZZXZZUUUX.....', // 16
  '.......XNZZXZZZZZZZZXXXZPRX.....', // 17 hand on the stick
  '.......XZZUXZZZZZZZZX..XNXX.....', // 18
  '.......XPPRXZZZZZZZZX..XNX......', // 19
  '.......XPPRXZZZZZZZUX..XNX......', // 20
  '........XXXXZZZZZZZUX..XNX......', // 21
  '...........XZZZZZZZUX..XNX......', // 22
  '...........XZZZZZZZUX..XNX......', // 23
  '..........XZZZZZZZZUUX.XNX......', // 24 coat hem
  '..........XUUUUUUUUUUX.XNX......', // 25
  '..........XXXXXXXXXXXX.XNX......', // 26
  '...........XUUUXXUUUX..XNX......', // 27
  '..........XUUUUXXUUUUX.XNX......', // 28
  '..........XXXXXXXXXXXX.XXX......', // 29
  '................................', // 30
  '................................', // 31
];

window.REF175_GRID = REF175_GRID;
