// SPDX-License-Identifier: MIT
// REF-85 - The Bookbinder on the Library kit (REF-79), 32x32, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Binder. Sleeves rolled so the forearms show, and a wide aged-paper apron
// from the waist over the scapular. Lesson prop (maybe_uninit): an open
// quire of blank leaves with a needle and a trailing wine thread - pages
// that exist but have nothing written on them yet.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow  P pink quartz
//         S stone grey  W wineflesh

const REF85_GRID = [
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
  '........XYYYOCVVVVSYYYKX........', // 10
  '........XYOYYCV*%VSYYOKX........', // 11 sigil
  '........XYOOXXXXXXXXOOKX........', // 12 open quire
  '........XYYPXCCCXCCCXPKX........', // 13 blank leaves, bare forearms
  '........XYYPXCCCXCCSXPKX........', // 14 needle
  '........XYYOXXXXXXXXWYKX........', // 15 thread
  '........XYYCVVVVVVVSWYKX........', // 16 apron
  '........XYYCVVVVVVVSYYKX........', // 17
  '........XYYCVVVVVVVSYYKX........', // 18
  '........XYYCVVVVVVVSYYKX........', // 19
  '........XYYCVVVVVVVSYKKX........', // 20
  '........XYYCVVVVVVVSYKKX........', // 21
  '........XYOCVVVVVVVSYKKX........', // 22
  '........XYOCVVVVVVVSYYKX........', // 23
  '........XYOCVVVVVVVSYYKX........', // 24
  '........XYOCVVVVVVVSYYKX........', // 25 apron ends
  '........XYOYYYYYYYYYKYKX........', // 26
  '........XOOOOOOOOOOOOOKX........', // 27 hem
  '........XYYYYYYYYYYYYYKX........', // 28
  '........XXXXXXXXXXXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF85_GRID = REF85_GRID;
