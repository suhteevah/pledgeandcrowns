// SPDX-License-Identifier: MIT
// REF-84 - The Courier on the Library kit (REF-79), 32x32, idle.
// APPROVED by Matt 2026-11-15. Not wired yet.
//
// Tome-carrier. The only one in the Library whose robe is kilted up: the hem
// stops short and the boots show, because this one walks between the stacks.
// Lesson prop (send_sync): a book-satchel on a strap across the chest, shut
// with a wax seal - what may be carried across, and what must stay shut.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow  P pink quartz
//         S stone grey  B basalt  N antique brass  W wineflesh  U bog umber

const REF84_GRID = [
  '................................', //  0
  '..............XXXX..............', //  1 hood peak
  '............XXYOOYXX............', //  2
  '...........XYYOOYYYYX...........', //  3
  '..........XYYOYYYYYYKX..........', //  4
  '..........XYO*KKKK*YKX..........', //  5
  '..........XYO*CKKC*YKX..........', //  6 eyes
  '..........XYO*KKKK*YKX..........', //  7
  '..........XYYO*KK*YYKX..........', //  8
  '.........XYYYO*VV*YYYKX.........', //  9
  '........XYYYOCVVVVSYNYKX........', // 10 strap from the shoulder
  '........XYOYYCV*%VSNYOKX........', // 11 sigil
  '........XYOOYCV**VNYOOKX........', // 12
  '........XYYOOYCVVNYOOYKX........', // 13
  '........XYYYOOPPNPOOYYKX........', // 14 hands
  '........XYYYKKKNVSKKYYKX........', // 15
  '........XYSBSBNBSBSBSBYX........', // 16 belt chain
  '........XXXXXXXVVVSYYYKX........', // 17 satchel
  '........XXNNNNXVVVSYYYKX........', // 18
  '........XXNWWNXVVVSYYKKX........', // 19 wax seal
  '........XXNWWNXVVVSYYKKX........', // 20
  '........XXNNNNXVVVSYYKKX........', // 21
  '........XXXXXXXVVVSYYYKX........', // 22
  '........XYOYYCVVVVSYYYKX........', // 23
  '........XYOYYYCVVSYYYYKX........', // 24
  '........XOOOOOOOOOOOOOKX........', // 25 hem, kilted up
  '........XXXXXXXXXXXXXXXX........', // 26
  '.........XUUX......XUUX.........', // 27 boots
  '.........XUUX......XUUX.........', // 28
  '.........XXXX......XXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF84_GRID = REF84_GRID;
