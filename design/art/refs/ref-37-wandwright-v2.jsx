// SPDX-License-Identifier: MIT
// REF-37 v2 - The Wandwright on the Tower kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: pointed hat, cobalt-deep robe, gold hem (REF-70). The Tower's
// craftsman: a short workshop cap instead of the tall hat, clean-shaven, and a
// bronze apron over the robe. Lesson prop (generic_fn): a plain stone-grey
// wand-blank raised in the frame-left hand, deliberately unfinished - no
// element bound yet, it will take whatever type it is given. Violet = 6 px.
//
// Legend: X coalblack  < cobalt deep  > cobalt  P pink quartz  R dusty rose
//         W wineflesh  S stone grey  V aged paper  G old gold  L brass leaf
//         N antique brass  Z bronze  U bog umber  % mage glow

const REF37_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...............XX...............', //  3 short cap
  '..............X<>X..............', //  4
  '.............X<<><X.............', //  5
  '............X%%%%%%X............', //  6 violet band
  '...........XXXXXXXXXX...........', //  7 small brim
  '...........XSPPPPPRSX...........', //  8
  '...........XPXPPPXPRX...........', //  9 eyes
  '....X......XPPPRRPPWX...........', // 10 wand tip
  '...XVX......XPRRRRWX............', // 11
  '...XSX....XX<<<<<<<<XX..........', // 12 shoulders
  '...XSX...X<<ZZZZZZZZ<<<X........', // 13 apron bib
  '...XSX..X<>>XNNZZZZZZX<<<X......', // 14
  '...XNX..X<>>XNZZZZZZZX<<<X......', // 15 wand collar
  '...XPXXXX>>>XNZZZZZZZX<<PX......', // 16 hands
  '...XRX<<<>>>XNZZGGZZZX<<<X......', // 17 apron pocket
  '....XXXXXX>>XNZZZZZZZX<<<X......', // 18
  '........X<>>XNZZZZZZZX<<<X......', // 19
  '........X<>>XNZZZZZZZX<<<X......', // 20
  '.......X<<>>XXXXXXXXX<<<<X......', // 21 apron hem
  '.......X<<>>>>>>>>>><<<<<X......', // 22
  '......X<<<>>>>>>>>>><<<<<<X.....', // 23
  '......X<<<>>>>>>>>>><<<<<<X.....', // 24
  '.....X<<<>>>>>>>>>>><<<<<<X.....', // 25
  '.....XGGLGGGGGGGGGGGGGGGGNX.....', // 26 gold hem
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 27
  '.........XUUX......XUUX.........', // 28
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF37_V2_GRID = REF37_V2_GRID;
