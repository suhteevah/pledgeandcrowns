// SPDX-License-Identifier: MIT
// REF-60 v2 - The Vaultwright on the Vault kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: crested helm, stone-grey plate with old-gold trim, gold pauldrons
// (REF-74). An open-faced helm, so this is the one Vault guard with a visible
// face. Lesson prop (box_basic): a single small gold strongbox held against
// the breastplate with both forearms folded over it - one value, put away in
// one box, with one owner.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         G old gold  L brass leaf  N antique brass
//         P pink quartz  R dusty rose  W wineflesh

const REF60_V2_GRID = [
  '................................', //  0
  '..............XLLX..............', //  1 crest
  '..............XGGX..............', //  2
  '...........XXXXGGXXXX...........', //  3
  '..........XSVVSGGSSSBX..........', //  4 helm
  '..........XGGGGGGGGGGX..........', //  5 brow band
  '..........XSPPPPPPPRBX..........', //  6 open face
  '..........XSPXPPPPXRBX..........', //  7 eyes
  '..........XSPPPRRPPWBX..........', //  8
  '..........XSSPRRRRWBBX..........', //  9
  '...........XSSSGGSBBX...........', // 10 chin guard
  '........XXXXXXXXXXXXXXXX........', // 11
  '.......XGGGXSVSSSSSBXGGGX.......', // 12 pauldrons
  '......XGLGGXXXXXXXXXXGGGNX......', // 13 box lid
  '......XGGGNXGGLGGGGNXGGNNX......', // 14
  '......XXXXXXGNXXXXGNXXXXXX......', // 15 lock band
  '......XSSBXXGGGXXGGNXXSBBX......', // 16
  '......XSSSSSSSSSBBBBBBBBBX......', // 17 forearms folded over
  '......XVSSSSSPPRPPRBBBBBBX......', // 18 hands
  '......XXXXXXXXXXXXXXXXXXXX......', // 19
  '...........XSSSXXSBBX...........', // 20 tassets
  '...........XSSSX.XSBBX..........', // 21
  '...........XSVSX.XSBBX..........', // 22
  '...........XSSSX.XSBBX..........', // 23
  '...........XGGGX.XGGGX..........', // 24 knee bands
  '...........XSSBX.XSBBX..........', // 25
  '...........XSSBX.XSBBX..........', // 26
  '..........XSSSBX.XSSBBX.........', // 27 sabatons
  '..........XBBBBX.XBBBBX.........', // 28
  '..........XXXXXX.XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF60_V2_GRID = REF60_V2_GRID;
