// SPDX-License-Identifier: MIT
// REF-61 v2 - The Sharekeeper on the Vault kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: stone-grey plate with old-gold trim, visored helm (REF-74), here a
// flat-topped helm with three gold studs on the brow band. Lesson prop
// (rc_basic): one hoard-chest held out at chest height, wider than the guard,
// with three gold ring-handles on its face - one chest, three holders, and it
// stays as long as any one of them holds on.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         G old gold  L brass leaf  N antique brass  Z bronze  U bog umber
//         P pink quartz  R dusty rose

const REF61_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...........XXXXXXXXXX...........', //  3 flat-topped helm
  '..........XSVVSSSSSSBX..........', //  4
  '..........XSVSSSSSSSBX..........', //  5
  '..........XGLGGLGGLGGX..........', //  6 three studs
  '..........XSXXXXXXXXBX..........', //  7 visor slit
  '..........XSSSSSSSSSBX..........', //  8
  '..........XSSSSSSSSBBX..........', //  9
  '...........XSSSSSSBBX...........', // 10
  '........XXXXXXXXXXXXXXXX........', // 11
  '.......XGGGXSVSSSSSBXGGGX.......', // 12 pauldrons
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 13 chest lid
  '.....XGGGGGGGGGLLGGGGGGGGNX.....', // 14
  '.....XNNNNNNNNNXXNNNNNNNNNX.....', // 15 lock band
  '...XPXZZGLGZZZZGLGZZZZGLGUXPX...', // 16 three ring-handles
  '...XRXZZGXGZZZZGXGZZZZGXGUXRX...', // 17
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 18
  '...........XSSSSSSBBX...........', // 19 waist
  '...........XGGGGGGGGX...........', // 20 gold belt
  '...........XSSSXXSBBX...........', // 21 tassets
  '...........XSSSX.XSBBX..........', // 22
  '...........XSVSX.XSBBX..........', // 23
  '...........XGGGX.XGGGX..........', // 24 knee bands
  '...........XSSBX.XSBBX..........', // 25
  '...........XSSBX.XSBBX..........', // 26
  '..........XSSSBX.XSSBBX.........', // 27 sabatons
  '..........XBBBBX.XBBBBX.........', // 28
  '..........XXXXXX.XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF61_V2_GRID = REF61_V2_GRID;
