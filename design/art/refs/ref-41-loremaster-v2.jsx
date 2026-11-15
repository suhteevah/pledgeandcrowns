// SPDX-License-Identifier: MIT
// REF-41 v2 - The Loremaster on the Tower kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: pointed hat, cobalt-deep robe, gold hem, aged-paper beard (REF-70).
// Lesson prop (assoc_type): an open gold-trimmed tome held forward at the
// waist in both hands, wider than the scholar. Each cream page carries one
// short inscription - the input on one page, its bound output on the other -
// joined by the bronze spine. Violet = 6 px (hat band).
//
// Legend: X coalblack  < cobalt deep  > cobalt  P pink quartz  R dusty rose
//         V aged paper  C parchment cream  G old gold  L brass leaf
//         N antique brass  Z bronze  U bog umber  % mage glow

const REF41_V2_GRID = [
  '...............XX...............', //  0 hat tip
  '..............X<<X..............', //  1
  '..............X<>X..............', //  2
  '.............X<<><X.............', //  3
  '.............X<>><X.............', //  4
  '............X<<>><<X............', //  5
  '............X%%%%%%X............', //  6 violet band
  '.........XXXX<<>>><<XXXX........', //  7 brim
  '........X<<<<<<<<<<<<<<<X.......', //  8
  '.........XXXXXXXXXXXXXXX........', //  9
  '...........XPPPPPPPRX...........', // 10
  '...........XPXPPPXPRX...........', // 11 eyes
  '...........XVPPRRPPVX...........', // 12
  '...........XVVVVVVVVX...........', // 13 beard
  '..........X<XVVVVVVX<X..........', // 14
  '.........X<<>XVVVVX<<<X.........', // 15
  '........X<<>>>XVVX><<<<X........', // 16
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 17 tome
  '....XGCCCCCCCCCZZCCCCCCCCCGX....', // 18
  '...XPXCCXXXCCCCZZCCCXXXXCCXPX...', // 19 input | output
  '...XRXCCCXCCCCCZZCCCXCCXCCXRX...', // 20
  '....XGCCCCCCCCCZZCCCCCCCCCGX....', // 21
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 22
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

window.REF41_V2_GRID = REF41_V2_GRID;
