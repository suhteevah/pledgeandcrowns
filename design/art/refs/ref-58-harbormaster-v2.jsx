// SPDX-License-Identifier: MIT
// REF-58 v2 - The Harbormaster on the Coast kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/harbormaster_idle_0.png.
//
// Kit: Coast cobalt (REF-73), cut for command: a peaked cap with a gold badge,
// a full grey beard, gold epaulettes, and a coat that runs to the ankles with
// a gold hem - the tallest block of cobalt on the coast. Lesson prop
// (thread_scope): the teal sash of office across the chest. Nothing leaves
// the harbour until every boat he sent out is back.
//
// Legend: X coalblack  > cobalt  < cobalt deep  T main teal  I bright teal
//         P pink quartz  R dusty rose  W wineflesh  S stone grey  V aged paper
//         G old gold  L brass leaf  U bog umber

const REF58_V2_GRID = [
  '................................', //  0
  '...........XXXXXXXXXX...........', //  1 peaked cap
  '..........X<<<<<<<<<<X..........', //  2
  '..........X<<<<GL<<<<X..........', //  3 badge
  '.........XXXXXXXXXXXXXX.........', //  4 peak
  '..........XSPPPPPPPRSX..........', //  5
  '..........XPPXPPPPXPRX..........', //  6 eyes
  '..........XPPPPRRPPPWX..........', //  7
  '..........XSVSSSSSSVSX..........', //  8 beard
  '...........XSSSSSSSSX...........', //  9
  '.......XXXXXXXSSSSXXXXXXX.......', // 10
  '......XGGG>>>>XX>>>>TGGGX.......', // 11 epaulettes
  '......X>>>>>>>>>>>>TTI><<X......', // 12 sash
  '......X>I>>>>>>>>TTI>>><<X......', // 13
  '......X>I>>>>>>TTI>>G>><<X......', // 14
  '......X>I>>>>TTI>>>>>>><<X......', // 15
  '......X>>>>TTI>>>>G>>>><<X......', // 16
  '......XRXTTI>>>>>>>>><<XRX......', // 17 hands
  '......XXXTI>>>>>>>>>><<XXX......', // 18
  '.........X>>>>>>>G>>><<X........', // 19
  '.........X>>>>>>>>>>><<X........', // 20
  '.........X>>>>>>>>>>><<X........', // 21
  '........X>>>>>>>>>>>>><<X.......', // 22 long coat
  '........X>>>>>>>G>>>>><<X.......', // 23
  '........X>>>>>>>>>>>>><<X.......', // 24
  '.......X>>>>>>>>>>>>>><<<X......', // 25
  '.......XGGGGGGGGGGGGGGGGGX......', // 26 gold hem
  '.......XXXXXXXXXXXXXXXXXXX......', // 27
  '..........XUUUX..XUUUX..........', // 28
  '..........XXXXX..XXXXX..........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF58_V2_GRID = REF58_V2_GRID;
