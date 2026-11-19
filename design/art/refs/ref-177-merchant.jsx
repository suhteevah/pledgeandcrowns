// SPDX-License-Identifier: MIT
// REF-177 - Townsfolk: merchant, 32x32. Ambient villager, gives no mission.
// APPROVED by Matt 2026-11-19. Not wired yet.
//
// A wide forest cap with a brass-leaf feather, a waxed basalt
// moustache, a cobalt doublet with gold buttons. Third drawing: the
// head is kept, the body is new. He is broad, with his arms folded
// across his chest. For the market stall and the Guildhall.
//
// Legend: X coalblack  E forest  F pine  H spring meadow
//         L brass leaf  G old gold  P pink quartz  R dusty rose
//         W wineflesh  B basalt  Z bronze  > cobalt
//         < cobalt deep  U bog umber

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
  '........XXX>>>>>>>>><XXX........', // 12
  '.......X>>>>>>>G>>>>>><<X.......', // 13
  '......X>>>>>>>>>>>>>>>><<X......', // 14
  '......X>>XXXXXXXXXXXXXX><X......', // 15 folded arms
  '......X>>XPP>>>>>>>>><X><X......', // 16
  '......X>>X>>>>>>>>><PPX><X......', // 17
  '......XXXXXXXXXXXXXXXXXXXX......', // 18
  '.........X>>>>>G>>>>><X.........', // 19
  '.........XZZZZZGGZZZZZX.........', // 20 belt
  '.........X>>>>>G>>>>><X.........', // 21
  '.........XXXXXXXXXXXXXX.........', // 22
  '.........XBBBBBXXBBBBBX.........', // 23
  '.........XBBBBX..XBBBBX.........', // 24
  '.........XBBBBX..XBBBBX.........', // 25
  '.........XBBBBX..XBBBBX.........', // 26
  '.........XUUUUX..XUUUUX.........', // 27
  '........XUUUUUX..XUUUUUX........', // 28
  '........XXXXXXX..XXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF177_GRID = REF177_GRID;
