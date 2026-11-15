// SPDX-License-Identifier: MIT
// REF-35 v2 - The Armorer on the Guildhall kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: Guildhall oxblood (REF-69), cut for the workshop: bare head, a full
// basalt beard, a hip-length tunic with a bronze belt and pine hem, bronze
// trousers and dark boots. Lesson prop (enum_data_match): a blade raised in the
// frame-left hand and a teal potion vial held out in the frame-right hand -
// the two variants of one enum, each carrying different data. Teal appears on
// the vial only (6 px).
//
// Legend: X coalblack  Y crypt  O oxblood  W wineflesh
//         P pink quartz  R dusty rose  B basalt  S stone grey  V aged paper
//         G old gold  N antique brass  Z bronze  U bog umber  F pine
//         T main teal  I bright teal

const REF35_V2_GRID = [
  '....X...........................', //  0 blade tip
  '...XVX..........................', //  1
  '...XSX....XXXXXXXXXX............', //  2 bare head
  '...XSX...XPPPPPPPPRRX...........', //  3
  '...XSX...XPPPPPPPPPRX...........', //  4
  '...XSX...XPXXPPPPXXRX...........', //  5 brows
  '...XSX...XPPXPPPPXPRX...........', //  6 eyes
  '...XSX...XPPPPRRPPPWX...........', //  7
  '..XNNNX..XBBBBBBBBBBX...........', //  8 hilt guard, beard
  '...XZX...XBBBBBBBBBBX...........', //  9 grip
  '...XPX....XBBBBBBBBX............', // 10 hand
  '...XRXXXXXXXOOOOOOXXXXXX........', // 11 raised arm
  '...XGXOOOOOOOWOOOOOOOOYX.XTX....', // 12 pommel, vial
  '....X.XOOOOWWOOOOOOOOYX.XITX....', // 13
  '......XOOOWWOOOOOOOOOYXPXTTX....', // 14 hand on the vial
  '.......XOOWOOOOOOOOOYXRPXXXX....', // 15
  '.......XZZZZZZNNZZZZZX..........', // 16 bronze belt
  '.......XOOWOOOOOOOOOYX..........', // 17 short tunic
  '.......XOOOOOOOOOOOOYX..........', // 18
  '.......XFFFFFFFFFFFFFX..........', // 19 pine hem
  '.......XXXXXXXXXXXXXXX..........', // 20
  '........XZZZZX.XZZZZX...........', // 21 trousers
  '........XZZZNX.XZZZNX...........', // 22
  '........XZZZNX.XZZZNX...........', // 23
  '........XZZZZX.XZZZZX...........', // 24
  '........XUUUUX.XUUUUX...........', // 25 boots
  '........XUUUUX.XUUUUX...........', // 26
  '.......XUUUUUX.XUUUUUX..........', // 27
  '.......XXXXXXX.XXXXXXX..........', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF35_V2_GRID = REF35_V2_GRID;
