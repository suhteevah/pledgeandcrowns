// SPDX-License-Identifier: MIT
// REF-70 - Costume kit exemplar, Act 4 Trait Mage Tower (Conjurer), 32x32.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit rule for the Tower cast: tall pointed hat, floor-length cobalt-deep robe
// with a gold hem, aged-paper beard, one small violet accent (hat band + orb).
// Silhouette = a tall triangle. Violet stays at 10 px, under the 5% cap (51).
//
// Legend: X coalblack  < cobalt deep  > cobalt  % mage glow  M mist teal
//         P pink quartz  R dusty rose  V aged paper
//         G old gold  L brass leaf  N antique brass  U bog umber

const REF70_GRID = [
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
  '...........XPPPPPPPRX...........', // 10 face
  '...........XPXPPPXPRX...........', // 11 eyes
  '...........XVPPRRPPVX...........', // 12 beard starts
  '...........XVVVVVVVVX...........', // 13
  '....%.....X<XVVVVVVX<X..........', // 14 orb top
  '...%M%...X<<>XVVVVX<<<X.........', // 15 orb
  '....%...X<<>>>XVVX><<<<X........', // 16
  '........X<>>>>>XX>><<<<X........', // 17
  '.......X<<>>>>>>>>><<<<<X.......', // 18
  '.......XP<>>>>>>>>><<<<PX.......', // 19 hands in sleeves
  '.......X<<>>>>>>>>><<<<<X.......', // 20
  '.......X<<>>>>>>>>><<<<<X.......', // 21
  '......X<<<>>>>>>>>><<<<<<X......', // 22
  '......X<<<>>>>>>>>><<<<<<X......', // 23
  '......X<<>>>>>>>>>>><<<<<X......', // 24
  '.....X<<<>>>>>>>>>>><<<<<<X.....', // 25
  '.....XGGLGGGGGGGGGGGGGGGGNX.....', // 26 gold hem
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 27
  '.........XUUX......XUUX.........', // 28 shoes
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF70_GRID = REF70_GRID;
