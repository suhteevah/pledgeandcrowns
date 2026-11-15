// SPDX-License-Identifier: MIT
// REF-63 v2 - The Swapwarden on the Vault kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/swapwarden_idle_0.png.
//
// Kit: stone-grey plate with old-gold trim and gold pauldrons (REF-74), under
// a mail coif instead of the helm, face showing. The guard stands at
// frame-left. Lesson prop (cell): a stone pedestal at frame-right with a
// single slot in its top, and one gold coin pinched in the fingers just above
// it - the old value out, the new one in, nothing else touched.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         G old gold  L brass leaf  N antique brass
//         P pink quartz  R dusty rose  W wineflesh

const REF63_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '.......XXXXXXXXXX...............', //  3 mail coif
  '......XSVVSSSSSSBX..............', //  4
  '......XSVPPPPPPSBX..............', //  5 face
  '......XSPXPPPPXRBX..............', //  6 eyes
  '......XSPPPRRPPWBX..............', //  7
  '......XSSPRRRRWBBX..............', //  8
  '......XSSSSSSSSBBX..............', //  9
  '.......XSSSSSSBBX...............', // 10
  '....XXXXXXXXXXXXXXXX............', // 11
  '...XGGGXSVSSSSSBXGGGX.....XXX...', // 12 pauldrons, coin
  '..XGLGGXSVSSGSSBXGGGNXXX.XGLGX..', // 13
  '..XGGGNXSSSGLGSBXGGNNSSPPXGGGX..', // 14 fingers pinch the coin
  '..XXXXXXSSSSGSSBXXXXXXXXX.XXX...', // 15
  '..XSSBXXSSSSSSBBXXSBBX..........', // 16
  '..XSSBXXGGGGGGGGXXSBBX..XXXXXXX.', // 17 gold belt, pedestal top
  '..XVSBXXSSSSSSBBXXSBBX..XVSXSBX.', // 18 the slot
  '..XPPRXXSSSSSSBBXXPPRX..XXXXXXX.', // 19
  '..XXXXXXSSSXXSBBXXXXXX...XSSBX..', // 20 pedestal column
  '.......XSSSX.XSBBX.......XSSBX..', // 21
  '.......XSVSX.XSBBX.......XSVBX..', // 22
  '.......XSSSX.XSBBX.......XSSBX..', // 23
  '.......XGGGX.XGGGX.......XSSBX..', // 24 knee bands
  '.......XSSBX.XSBBX.......XSSBX..', // 25
  '.......XSSBX.XSBBX.......XSSBX..', // 26
  '......XSSSBX.XSSBBX.....XSSSSBX.', // 27 sabatons, pedestal foot
  '......XBBBBX.XBBBBX.....XBBBBBX.', // 28
  '......XXXXXX.XXXXXX.....XXXXXXX.', // 29
  '................................', // 30
  '................................', // 31
];

window.REF63_V2_GRID = REF63_V2_GRID;
