// SPDX-License-Identifier: MIT
// REF-80 - The Shelfmark on the Library kit (REF-79), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Catalogue keeper. Lesson prop (raw_pointer): a long pointer rod ending in
// a blank call-number card. The rod is the address, not the book - the card
// says where, and nothing on it says what is there.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow  P pink quartz
//         S stone grey  B basalt  N antique brass

const REF80_GRID = [
  '................................', //  0
  '..............XXXX.....XXXXX....', //  1 hood peak, card
  '............XXYOOYXX...XCCCX....', //  2
  '...........XYYOOYYYYX..XCCCX....', //  3
  '..........XYYOYYYYYYKX.XXXXX....', //  4
  '..........XYO*KKKK*YKX...N......', //  5 rod
  '..........XYO*CKKC*YKX...N......', //  6 eyes
  '..........XYO*KKKK*YKX...N......', //  7
  '..........XYYO*KK*YYKX...N......', //  8
  '.........XYYYO*VV*YYYKX..N......', //  9
  '........XYYYOCVVVVSYYYKX.N......', // 10
  '........XYOYYCV*%VSYYOKX.N......', // 11 sigil
  '........XYOOYCV**VSYOOKX.N......', // 12
  '........XYYOOYCVVSYYOOOXPN......', // 13 right hand on the rod
  '........XYYYOOPPCVSYYOKX.N......', // 14 left hand at the chest
  '........XYYYKKKCVSYYYYKX.N......', // 15
  '........XYSBSBSBSBSBSBYX.N......', // 16 belt chain
  '........XYYSYCVVVVSYYYKX.N......', // 17
  '........XYYBYCVVVVSYYYKX.N......', // 18
  '........XYYSYCVVVVSYYYKX.N......', // 19
  '........XYYOYCVVVVSYYKKX.N......', // 20
  '........XYYOYCVVVVSYYKKX.N......', // 21
  '........XYOYYCVVVVSYYKKX.N......', // 22
  '........XYOYYCVVVVSYYYKX.N......', // 23
  '........XYOYYYCVVSYYYYKX.N......', // 24
  '........XYOYYYYYYYYYKYKX.N......', // 25
  '........XYOYYYYYYYYYKYKX.N......', // 26
  '........XOOOOOOOOOOOOOKX.N......', // 27 hem
  '........XYYYYYYYYYYYYYKX.N......', // 28
  '........XXXXXXXXXXXXXXXX.N......', // 29
  '................................', // 30
  '................................', // 31
];

window.REF80_GRID = REF80_GRID;
