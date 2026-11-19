// SPDX-License-Identifier: MIT
// REF-178 - Townsfolk: guard, 32x32. Ambient villager, gives no mission.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A stone-grey helm with a nose guard, grey sleeves, a wine tabard
// with a brass-leaf badge, a spear in his left hand. For street
// mouths, gates and the Throne.
//
// Legend: X coalblack  S stone grey  V aged paper  B basalt
//         P pink quartz  R dusty rose  W wineflesh  O oxblood
//         L brass leaf  N antique brass  U bog umber

const REF178_GRID = [
  '...........................XSX..', //  0 spear
  '...........................XSVX.', //  1
  '...........XXXXXXXXXX......XSVX.', //  2 helm
  '..........XVVSSSSSSSBX......NX..', //  3
  '..........XVSSSSSSSSBX......NX..', //  4
  '..........XSSSSXXSSSBX......NX..', //  5
  '..........XSXXPXXPXXBX......NX..', //  6
  '..........XSPXPXXPXPBX......NX..', //  7
  '..........XSPPPXXPPPBX......NX..', //  8
  '..........XPPPRRRRPPWX......NX..', //  9
  '..........XPPPRWWRPPRX......NX..', // 10
  '...........XXPRRRRWXX.......NX..', // 11
  '........XXXSSSBBBBSSSBXXX...NX..', // 12 tabard
  '......XXSSSSWWWWWWWWSSSBBXX.NX..', // 13
  '.....XSSSSBXRRWWWWWWXSSSBBX.NX..', // 14
  '.....XSSSBBXRWWWWWWWXSSBBBX.NX..', // 15
  '.....XPPRXXXRWWWWWWWXXXPPRX.NX..', // 16
  '.....XPRRX.XRWWWWWWOX.XPRRX.NX..', // 17
  '......XXX..XRWWWWWWOX..XXX..NX..', // 18
  '..........XXRWWWWWWOXX......NX..', // 19
  '.........XBXRWWLLWWOXBX.....NX..', // 20
  '.........XBXRWWWWWWOXBX.....NX..', // 21
  '.........XBXRWWWWWWOXBX.....NX..', // 22
  '.........XBXXXXXXXXXXBX.....NX..', // 23
  '.........XSSSSSXXSSSSSX.....NX..', // 24
  '.........XSSSSX..XSSSSX.....NX..', // 25
  '.........XSSSSX..XSSSSX.....NX..', // 26
  '.........XUUUUX..XUUUUX.....NX..', // 27
  '........XUUUUUX..XUUUUUX....NX..', // 28
  '........XXXXXXX..XXXXXXX....NX..', // 29
  '............................XX..', // 30
  '................................', // 31
];

window.REF178_GRID = REF178_GRID;
