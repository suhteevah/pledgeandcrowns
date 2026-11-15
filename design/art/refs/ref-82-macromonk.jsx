// SPDX-License-Identifier: MIT
// REF-82 - The Macro Monk on the Library kit (REF-79), 32x32, idle.
// APPROVED by Matt 2026-11-15. Not wired yet.
//
// Silent copyist. Hood pulled lower than anyone else in the Library, head
// bowed, hands lost in the joined sleeves. Lesson prop (macro_rules): a
// carved woodblock stamp hanging at the belt, its face one small pattern
// repeated - write the pattern once, press it as many times as you need.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow
//         S stone grey  B basalt  N antique brass  Z bronze

const REF82_GRID = [
  '................................', //  0
  '..............XXXX..............', //  1 hood peak
  '............XXYOOYXX............', //  2
  '...........XYYOOYYYYX...........', //  3
  '..........XYYOYYYYYYKX..........', //  4
  '..........XYYOYYYYYYKX..........', //  5 hood drawn low
  '..........XYO*KKKK*YKX..........', //  6
  '..........XYO*CKKC*YKX..........', //  7 eyes, bowed
  '..........XYYO*KK*YYKX..........', //  8
  '.........XYYYO*VV*YYYKX.........', //  9
  '........XYYYOCVVVVSYYYKX........', // 10
  '........XYOYYCV*%VSYYOKX........', // 11 sigil
  '........XYOOYCV**VSYOOKX........', // 12 sleeves
  '........XYYOOYCVVSYOOYKX........', // 13
  '........XYYYOOOOOOOOYYKX........', // 14 sleeves joined, no hands
  '........XYYYKKKKKKKKYYKX........', // 15
  '........XYSBXXXXXXXXBSYX........', // 16 chain, woodblock
  '........XYYOXNZNZNZXYYKX........', // 17 repeated pattern
  '........XYYOXZNZNZNXYYKX........', // 18
  '........XYYOXNZNZNZXYYKX........', // 19
  '........XYYOXXXXXXXXYKKX........', // 20
  '........XYYOYCVVVVSYYKKX........', // 21
  '........XYOYYCVVVVSYYKKX........', // 22
  '........XYOYYCVVVVSYYYKX........', // 23
  '........XYOYYYCVVSYYYYKX........', // 24
  '........XYOYYYYYYYYYKYKX........', // 25
  '........XYOYYYYYYYYYKYKX........', // 26
  '........XOOOOOOOOOOOOOKX........', // 27 hem
  '........XYYYYYYYYYYYYYKX........', // 28
  '........XXXXXXXXXXXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF82_GRID = REF82_GRID;
