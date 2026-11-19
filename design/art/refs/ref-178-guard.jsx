// SPDX-License-Identifier: MIT
// REF-178 - Townsfolk: guard, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-19. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A stone-grey helm with a nose guard, grey sleeves, a wine tabard
// with a brass-leaf badge. Third drawing: the head is kept, the body
// is new. He stands to attention, arms straight down and feet
// together, and the spear is now two pixels thick. For street
// mouths, gates and the Throne.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         P pink quartz  R dusty rose  W wineflesh  O oxblood
//         L brass leaf  N antique brass  Z bronze  U bog umber

const REF178_GRID = [
  '.........................XX.....', //  0 spear
  '........................XSVX....', //  1
  '...........XXXXXXXXXX..XSSVVX...', //  2 helm
  '..........XVVSSSSSSSBX.XSSVVX...', //  3
  '..........XVSSSSSSSSBX.XXSVXX...', //  4
  '..........XSSSSXXSSSBX..XNZX....', //  5
  '..........XSXXPXXPXXBX..XNZX....', //  6
  '..........XSPXPXXPXPBX..XNZX....', //  7
  '..........XSPPPXXPPPBX..XNZX....', //  8
  '..........XPPPRRRRPPWX..XNZX....', //  9
  '..........XPPPRWWRPPRX..XNZX....', // 10
  '...........XXPRRRRWXX...XNZX....', // 11
  '........XXSSWWWWWWWWSBXXXNZX....', // 12 tabard
  '.......XSSSWWWWWWWWWWSBBXNZX....', // 13
  '.......XSSSXWWWWWWWWXSBBXNZX....', // 14
  '.......XSSSXWWWLLWWWXSBBXNZX....', // 15
  '.......XSSSXWWWWWWWWXSBBXNZX....', // 16
  '.......XSSSXZZZLLZZZXSBBXNZX....', // 17
  '.......XSSBXWWWWWWWOXSBBXNZX....', // 18
  '.......XPPRXWWWWWWWOXPPRXNZX....', // 19
  '.......XPPRXWWWWWWWOXPPRXNZX....', // 20
  '........XXXXWWWWWWWOXXXXXNZX....', // 21
  '...........XXXXXXXXXX...XNZX....', // 22
  '...........XSSSSSSSSX...XNZX....', // 23 greaves
  '...........XSSSXXSSSX...XNZX....', // 24
  '...........XSSBXXSSBX...XNZX....', // 25
  '...........XSSBXXSSBX...XNZX....', // 26
  '...........XUUUXXUUUX...XNZX....', // 27
  '..........XUUUUXXUUUUX..XNZX....', // 28
  '..........XXXXXXXXXXXX..XNZX....', // 29
  '........................XXXX....', // 30
  '................................', // 31
];

window.REF178_GRID = REF178_GRID;
