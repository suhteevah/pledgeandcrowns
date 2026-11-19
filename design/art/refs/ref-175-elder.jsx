// SPDX-License-Identifier: MIT
// REF-175 - Townsfolk: elder, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Bald on top with white hair at the sides and a long aged-paper
// beard that falls to a point on his chest, a bronze coat, a brass
// walking stick in his left hand. Second drawing: the beard now
// reads. For benches and the well.
//
// Legend: X coalblack  V aged paper  S stone grey  P pink quartz
//         R dusty rose  W wineflesh  Z bronze  N antique brass
//         G old gold  U bog umber  C parchment cream

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
  '........XXXXVVVVVVVVSXXXX.......', // 12
  '......XXZZZCXVVVVVVSXCZUUXX.....', // 13
  '.....XZZZZUXNXVVVVSXXCZZUUXXXX..', // 14 walking stick
  '.....XZZZUUXNZXVVSXZXCZUUUNNNX..', // 15
  '.....XPPRXXXNZZZZZZZXXXPPRX.NX..', // 16 coat
  '.....XPRRX.XNZZZZZZUX.XPRRX.NX..', // 17
  '......XXX..XNZZZZZZUX..XXX..NX..', // 18
  '..........XXNZZZZZZUXX......NX..', // 19
  '.........XUXNZZGGZZUXUX.....NX..', // 20
  '.........XUXNZZZZZZUXUX.....NX..', // 21
  '.........XUXNZZZZZZUXUX.....NX..', // 22
  '.........XUXXXXXXXXXXUX.....NX..', // 23
  '.........XUUUUUXXUUUUUX.....NX..', // 24
  '.........XUUUUX..XUUUUX.....NX..', // 25
  '.........XUUUUX..XUUUUX.....NX..', // 26
  '.........XUUUUX..XUUUUX.....NX..', // 27
  '........XUUUUUX..XUUUUUX....NX..', // 28
  '........XXXXXXX..XXXXXXX....NX..', // 29
  '............................XX..', // 30
  '................................', // 31
];

window.REF175_GRID = REF175_GRID;
