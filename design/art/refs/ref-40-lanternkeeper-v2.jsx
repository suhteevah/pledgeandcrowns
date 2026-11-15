// SPDX-License-Identifier: MIT
// REF-40 v2 - The Lanternkeeper on the Tower kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/lanternkeeper_idle_0.png.
//
// Kit: pointed hat, cobalt-deep robe, gold hem, aged-paper beard (REF-70),
// with a gold hat band. Lesson prop (lifetimes): a crook staff planted at
// frame-left, as tall as the frame, with a brass lantern hanging from its
// hook. The violet flame burns only inside the glass and only while the
// lantern hangs there - a borrow that lives exactly as long as its holder.
// Violet = 2 px.
//
// Legend: X coalblack  < cobalt deep  > cobalt  P pink quartz  R dusty rose
//         V aged paper  G old gold  L brass leaf  N antique brass  Z bronze
//         U bog umber  * royal arcane  % mage glow

const REF40_V2_GRID = [
  '...............XX...............', //  0 hat tip
  '.XXXXXXXX.....X<<X..............', //  1 crook
  '.XZZZZZZX.....X<>X..............', //  2
  '.XZXXXNXX....X<<><X.............', //  3 lantern ring
  '.XZX.XNNNX...X<>><X.............', //  4 lantern cap
  '.XZX.XG%GX..X<<>><<X............', //  5 flame
  '.XZX.XG*GX..XGGGGGGX............', //  6 gold hat band
  '.XZX.XNNNXXXX<<>>><<XXXX........', //  7 lantern base, brim
  '.XZX..XXXX<<<<<<<<<<<<<<X.......', //  8
  '.XZX.....XXXXXXXXXXXXXXX........', //  9
  '.XZX.......XPPPPPPPRX...........', // 10
  '.XZX.......XPXPPPXPRX...........', // 11 eyes
  '.XZX.......XVPPRRPPVX...........', // 12
  '.XZX.......XVVVVVVVVX...........', // 13 beard
  '.XZX......X<XVVVVVVX<X..........', // 14
  '.XZX.....X<<>XVVVVX<<<X.........', // 15
  '.XZXXXXXX<<>>>XVVX><<<<X........', // 16 arm to the staff
  '.XZXP<<<X<>>>>>XX>><<<<X........', // 17 grip
  '.XZXXXXX<<>>>>>>>>><<<<<X.......', // 18
  '.XZX...XP<>>>>>>>>><<<<PX.......', // 19
  '.XZX...X<<>>>>>>>>><<<<<X.......', // 20
  '.XZX...X<<>>>>>>>>><<<<<X.......', // 21
  '.XZX..X<<<>>>>>>>>><<<<<<X......', // 22
  '.XZX..X<<<>>>>>>>>><<<<<<X......', // 23
  '.XZX..X<<>>>>>>>>>>><<<<<X......', // 24
  '.XZX.X<<<>>>>>>>>>>><<<<<<X.....', // 25
  '.XZX.XGGLGGGGGGGGGGGGGGGGNX.....', // 26 gold hem
  '.XZX.XXXXXXXXXXXXXXXXXXXXXX.....', // 27
  '.XNX.....XUUX......XUUX.........', // 28
  '.XXX.....XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF40_V2_GRID = REF40_V2_GRID;
