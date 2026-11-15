// SPDX-License-Identifier: MIT
// REF-64 v2 - The Strongbox keeper on the Vault kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: crested, visored helm and stone-grey plate with old-gold trim (REF-74).
// Lesson prop (rc_refcell): a strongbox so large it hides the guard from neck
// to knee, gauntlets resting on its lid, with TWO lock plates on its face -
// more than one keyholder, and any of them can open it to change what is
// inside. The widest, squarest silhouette in the Vault.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         G old gold  L brass leaf  N antique brass  Z bronze  U bog umber

const REF64_V2_GRID = [
  '................................', //  0
  '..............XLLX..............', //  1 crest
  '..............XGGX..............', //  2
  '...........XXXXGGXXXX...........', //  3
  '..........XSVVSGGSSSBX..........', //  4 helm
  '..........XSVSSGGSSSBX..........', //  5
  '..........XGGGGGGGGGGX..........', //  6 brow band
  '..........XSXXXXXXXXBX..........', //  7 visor slit
  '..........XSSSSGGSSSBX..........', //  8
  '..........XSSSSGGSSBBX..........', //  9
  '...........XSSSGGSBBX...........', // 10
  '....XSSXXXXXXXXXXXXXXXXXXSSX....', // 11 gauntlets on the lid
  '....XGGGGGGGGGGLLGGGGGGGGGGX....', // 12 lid
  '....XGNNNNNNNNNNNNNNNNNNNNNX....', // 13
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 14
  '....XZZZZZZZZZZZZZZZZZZZZZUX....', // 15 box face
  '....XZZXXXXZZZZZZZZZXXXXZZUX....', // 16 two lock plates
  '....XZZXGGXZZZZZZZZZXGGXZZUX....', // 17
  '....XZZXGXXZZZZZZZZZXGXXZZUX....', // 18 keyholes
  '....XZZXXXXZZZZZZZZZXXXXZZUX....', // 19
  '....XZZZZZZZZZZZZZZZZZZZZZUX....', // 20
  '....XNNNNNNNNNNNNNNNNNNNNNNX....', // 21 base band
  '....XUUUUUUUUUUUUUUUUUUUUUUX....', // 22
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 23
  '...........XGGGX.XGGGX..........', // 24 knee bands
  '...........XSSBX.XSBBX..........', // 25
  '...........XSSBX.XSBBX..........', // 26
  '..........XSSSBX.XSSBBX.........', // 27 sabatons
  '..........XBBBBX.XBBBBX.........', // 28
  '..........XXXXXX.XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF64_V2_GRID = REF64_V2_GRID;
