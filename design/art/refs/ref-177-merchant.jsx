// SPDX-License-Identifier: MIT
// REF-177 - Townsfolk: merchant, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A wide forest cap with a brass-leaf feather, a waxed basalt
// moustache, a cobalt doublet with gold buttons down the front.
// For the market stall and the Guildhall.
//
// Legend: X coalblack  E forest  F pine  H spring meadow
//         L brass leaf  G old gold  P pink quartz  R dusty rose
//         W wineflesh  B basalt  > cobalt  < cobalt deep
//         U bog umber

const REF177_GRID = [
  '................................', //  0
  '.................XLX............', //  1 feather
  '.........XXXXXXXXXLXXX..........', //  2
  '........XEEEEEEEEEELEFX.........', //  3 cap
  '........XHEEEEEEEEEEFFX.........', //  4
  '.........XXXXXXXXXXXXX..........', //  5
  '..........XPXXPPPPXXRX..........', //  6
  '..........XPPXPPPPXPRX..........', //  7
  '..........XPPPPRRPPPWX..........', //  8
  '..........XPBBRRRRBBWX..........', //  9 moustache
  '..........XPPBBWWBBPRX..........', // 10
  '...........XXPRRRRWXX...........', // 11
  '........XXX>>><<<<>>><XXX.......', // 12 doublet
  '......XX>>>>>>>G>>>>>>><<XX.....', // 13
  '.....X>>>><XLL>>>>>>X>>><<X.....', // 14
  '.....X>>><<XL>>>>>>>X>><<<X.....', // 15
  '.....XPPRXXXL>>G>>>>XXXPPRX.....', // 16
  '.....XPRRX.XL>>>>>><X.XPRRX.....', // 17
  '......XXX..XL>>>>>><X..XXX......', // 18
  '..........XXL>>G>>><XX..........', // 19
  '.........XBXL>>GG>><XBX.........', // 20
  '.........XBXL>>>>>><XBX.........', // 21
  '.........XBXL>>G>>><XBX.........', // 22
  '.........XBXXXXXXXXXXBX.........', // 23
  '.........XBBBBBXXBBBBBX.........', // 24
  '.........XBBBBX..XBBBBX.........', // 25
  '.........XBBBBX..XBBBBX.........', // 26
  '.........XUUUUX..XUUUUX.........', // 27
  '........XUUUUUX..XUUUUUX........', // 28
  '........XXXXXXX..XXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF177_GRID = REF177_GRID;
