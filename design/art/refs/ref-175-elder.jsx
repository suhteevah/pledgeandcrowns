// SPDX-License-Identifier: MIT
// REF-175 - Townsfolk: elder, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// White hair at the temples and a full aged-paper beard down to the
// chest, a bronze coat with a brass front, a brass walking stick in
// his left hand. Built on the approved REF-71 villager body. For
// benches and the well.
//
// Legend: X coalblack  V aged paper  S stone grey  P pink quartz
//         R dusty rose  W wineflesh  Z bronze  N antique brass
//         G old gold  U bog umber

const REF175_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...........XXXXXXXXXX...........', //  3
  '..........XVVPPPPPPVSX..........', //  4 white hair
  '..........XVPPPPPPPPSX..........', //  5
  '..........XPXXPPPPXXRX..........', //  6
  '..........XPPXPPPPXPRX..........', //  7
  '..........XPPPPRRPPPWX..........', //  8
  '..........XVVVVVVVVVSX..........', //  9 beard
  '..........XVVVRRRRVVSX..........', // 10
  '...........XXVVVVVSXX...........', // 11
  '........XXXZZVVVVVVZZVXXX.......', // 12
  '......XXZZZZZZZZZZZZZZZUUXX.....', // 13 coat
  '.....XZZZZUXNNZZZZZZXZZZUUXXXX..', // 14 walking stick
  '.....XZZZUUXNZZZZZZZXZZUUUNNNX..', // 15
  '.....XPPRXXXNZZZZZZZXXXPPRX.NX..', // 16
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
