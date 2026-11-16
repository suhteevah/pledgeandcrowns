// SPDX-License-Identifier: MIT
// REF-74 - Costume kit exemplar, Act 8 Vault of Pointers (Warden), 32x32.
// APPROVED by Matt 2026-11-13 (tranche 1) as a kit exemplar. Wired as game/assets/sprites/npc/warden_idle_0.png (the Warden's sprite, confirmed by Matt 2026-11-15).
//
// Kit rule for the Vault cast: stone-grey plate with old-gold trim, a crested
// helm with a visor slit, gold pauldrons, a ring-key at the hip. Silhouette =
// armoured and square, the only cast with no visible face.
// Zone roles: Old gold, Brass leaf, Stone grey, Basalt.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         G old gold  L brass leaf  N antique brass
//         P pink quartz  R dusty rose

const REF74_GRID = [
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
  '........XXXXXXXXXXXXXXXX........', // 11
  '.......XGGGXSVSSSSSBXGGGX.......', // 12 pauldrons
  '......XGLGGXSVSSGSSBXGGGNX......', // 13
  '......XGGGNXSSSGLGSBXGGNNX......', // 14 breast sigil
  '......XXXXXXSSSSGSSBXXXXXX......', // 15
  '......XSSBXXSSSSSSBBXXSBBX......', // 16 vambraces
  '......XSSBXXGGGGGGGGXXSBBX......', // 17 gold belt
  '......XVSBXXSSSSSSBBXXSBBX......', // 18
  '......XPPRXXSSSSSSBBXXPPRX......', // 19 hands
  '......XXXXXXSSSXXSBBXXXXXX......', // 20 tassets
  '...........XSSSX.XSBBX.XGX......', // 21 ring-key
  '...........XSVSX.XSBBX.XGX......', // 22
  '...........XSSSX.XSBBX.XGXX.....', // 23
  '...........XGGGX.XGGGX.XGX......', // 24 knee bands
  '...........XSSBX.XSBBX.XGXX.....', // 25
  '...........XSSBX.XSBBX..XX......', // 26
  '..........XSSSBX.XSSBBX.........', // 27 sabatons
  '..........XBBBBX.XBBBBX.........', // 28
  '..........XXXXXX.XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF74_GRID = REF74_GRID;
