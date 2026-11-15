// SPDX-License-Identifier: MIT
// REF-36 v2 - Vexis the Archmage on the Tower kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/vexis_idle_0.png.
//
// Kit: pointed hat, floor-length cobalt-deep robe, gold hem, aged-paper beard
// (REF-70). The Archmage outranks the rest: the widest brim in the cast, an
// old-gold hat band and a gold yoke at the shoulders. Lesson prop (trait_def):
// a staff as tall as the frame, topped by the only violet on the figure - the
// capability being granted, at the top of the frame. Violet = 10 px (cap 51).
//
// Legend: X coalblack  < cobalt deep  > cobalt  P pink quartz  R dusty rose
//         V aged paper  G old gold  L brass leaf  N antique brass  Z bronze
//         U bog umber  * royal arcane  % mage glow  M mist teal

const REF36_V2_GRID = [
  '...............XX...........*%*.', //  0 orb
  '..............X<<X.........*%M%*', //  1
  '..............X<>X..........*%*.', //  2
  '.............X<<><X.........XGX.', //  3 finial cup
  '.............X<>><X.........XNX.', //  4
  '............X<<>><<X........XZX.', //  5
  '............XGGGGGGX........XZX.', //  6 gold hat band
  '.......XXXXXX<<>>><<XXXXXX..XZX.', //  7 wide brim
  '......X<<<<<<<<<<<<<<<<<<<X.XZX.', //  8
  '.......XXXXXXXXXXXXXXXXXXX..XZX.', //  9
  '...........XPPPPPPPRX.......XZX.', // 10
  '...........XPXPPPXPRX.......XZX.', // 11 eyes
  '...........XVPPRRPPVX.......XZX.', // 12
  '...........XVVVVVVVVX.......XZX.', // 13 beard
  '.........XGGXVVVVVVXGGX.....XZX.', // 14 gold yoke
  '........XGLG>XVVVVX<GGGX....XZX.', // 15
  '........X<<>>>XVVX><<<<X....XZX.', // 16
  '........X<>>>>>XX>><<<<<<<XPXZX.', // 17 hand on the staff
  '.......X<<>>>>>>>>><<<<<<<XRXZX.', // 18
  '.......XP<>>>>>>>>><<<<<XXX.XZX.', // 19
  '.......X<<>>>>>>>>><<<<<X...XZX.', // 20
  '.......X<<>>>>>>>>><<<<<X...XZX.', // 21
  '......X<<<>>>>>>>>><<<<<<X..XZX.', // 22
  '......X<<<>>>>>>>>><<<<<<X..XZX.', // 23
  '......X<<>>>>>>>>>>><<<<<X..XZX.', // 24
  '.....X<<<>>>>>>>>>>><<<<<<X.XZX.', // 25
  '.....XGGLGGGGGGGGGGGGGGGGNX.XZX.', // 26 gold hem
  '.....XXXXXXXXXXXXXXXXXXXXXX.XZX.', // 27
  '.........XUUX......XUUX.....XNX.', // 28
  '.........XXXX......XXXX.....XXX.', // 29
  '................................', // 30
  '................................', // 31
];

window.REF36_V2_GRID = REF36_V2_GRID;
