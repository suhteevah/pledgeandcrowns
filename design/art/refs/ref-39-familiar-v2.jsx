// SPDX-License-Identifier: MIT
// REF-39 v2 - The Familiar's keeper on the Tower kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: pointed hat, cobalt-deep robe, gold hem (REF-70). A younger keeper:
// clean-shaven, a brass belt. Lesson prop (dyn_trait): a bronze cage balanced
// on the raised frame-right palm, holding one creature made of three clashing
// parts - a teal wing, an old-gold body, a forest tail - many types behind one
// box. Violet = 6 px (hat band).
//
// Legend: X coalblack  < cobalt deep  > cobalt  P pink quartz  R dusty rose
//         W wineflesh  G old gold  L brass leaf  N antique brass  Z bronze
//         U bog umber  B basalt  T main teal  E forest  % mage glow

const REF39_V2_GRID = [
  '...............XX...............', //  0 hat tip
  '..............X<<X..............', //  1
  '..............X<>X.........XNX..', //  2 cage ring
  '.............X<<><X......XZZZZZX', //  3 cage top
  '.............X<>><X......X.TT..X', //  4 teal wing
  '............X<<>><<X.....XTGGB.X', //  5 gold body, eye
  '............X%%%%%%X.....X.GGE.X', //  6 forest tail
  '.........XXXX<<>>><<XXXX.X..EE.X', //  7
  '........X<<<<<<<<<<<<<<<XXZZZZZX', //  8 brim, cage base
  '.........XXXXXXXXXXXXXXX...XPX..', //  9 palm
  '...........XPPPPPPPRX.....XPRX..', // 10
  '...........XPXPPPXPRX.....X<<X..', // 11 eyes, raised sleeve
  '...........XPPPRRPPWX....X<<X...', // 12
  '............XPRRRRWX....X<<X....', // 13 chin
  '..........X<XXXXXXXX<XXX<<X.....', // 14 shoulders
  '.........X<<>>>>>>>><<<<<<X.....', // 15
  '........X<<>>>>>>>>><<<<<X......', // 16
  '........X<>>>>>>>>>><<<<X.......', // 17
  '.......X<<>>>>>>>>><<<<<X.......', // 18
  '.......XP<>>>NNGNN><<<<<X.......', // 19 belt, resting hand
  '.......X<<>>>>>>>>><<<<<X.......', // 20
  '.......X<<>>>>>>>>><<<<<X.......', // 21
  '......X<<<>>>>>>>>><<<<<<X......', // 22
  '......X<<<>>>>>>>>><<<<<<X......', // 23
  '......X<<>>>>>>>>>>><<<<<X......', // 24
  '.....X<<<>>>>>>>>>>><<<<<<X.....', // 25
  '.....XGGLGGGGGGGGGGGGGGGGNX.....', // 26 gold hem
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 27
  '.........XUUX......XUUX.........', // 28
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF39_V2_GRID = REF39_V2_GRID;
