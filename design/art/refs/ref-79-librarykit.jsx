// SPDX-License-Identifier: MIT
// REF-79 - Costume kit exemplar, Act 9 Forbidden Library, 32x32.
// APPROVED by Matt 2026-11-15 as a kit exemplar. NOT wired to any NPC: the Library cast (REF-80..85, REF-93) is drawn on it.
//
// Kit rule for the Library cast: a deep hood and a Crypt robe that falls
// straight to the floor with no flare and no visible feet, an aged-paper
// scapular down the front (cream on its lit edge, stone grey on the other,
// so it holds against pale flagstone), royal-arcane hood lining and one
// sigil, a face lost in the hood but for two eye pixels, sleeves meeting at
// clasped hands, a chain at the belt.
// Silhouette = a hooded column. Violet is 14 px here, under the 5% cap (51).
// No alarm scarlet on any cast sprite.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow  P pink quartz
//         S stone grey  B basalt

const REF79_GRID = [
  '................................', //  0
  '..............XXXX..............', //  1 hood peak
  '............XXYOOYXX............', //  2
  '...........XYYOOYYYYX...........', //  3
  '..........XYYOYYYYYYKX..........', //  4
  '..........XYO*KKKK*YKX..........', //  5 hood mouth, violet lining
  '..........XYO*CKKC*YKX..........', //  6 eyes
  '..........XYO*KKKK*YKX..........', //  7
  '..........XYYO*KK*YYKX..........', //  8
  '.........XYYYO*VV*YYYKX.........', //  9 shoulders, scapular starts
  '........XYYYOCVVVVSYYYKX........', // 10
  '........XYOYYCV*%VSYYOKX........', // 11 sigil
  '........XYOOYCV**VSYOOKX........', // 12 sleeves
  '........XYYOOYCVVSYOOYKX........', // 13
  '........XYYYOOPPPPOOYYKX........', // 14 clasped hands
  '........XYYYKKCVVSKKYYKX........', // 15
  '........XYSBSBSBSBSBSBYX........', // 16 belt chain
  '........XYYSYCVVVVSYYYKX........', // 17 chain end hangs
  '........XYYBYCVVVVSYYYKX........', // 18
  '........XYYSYCVVVVSYYYKX........', // 19
  '........XYYOYCVVVVSYYKKX........', // 20
  '........XYYOYCVVVVSYYKKX........', // 21
  '........XYOYYCVVVVSYYKKX........', // 22
  '........XYOYYCVVVVSYYYKX........', // 23
  '........XYOYYYCVVSYYYYKX........', // 24 scapular ends
  '........XYOYYYYYYYYYKYKX........', // 25
  '........XYOYYYYYYYYYKYKX........', // 26
  '........XOOOOOOOOOOOOOKX........', // 27 hem
  '........XYYYYYYYYYYYYYKX........', // 28
  '........XXXXXXXXXXXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF79_GRID = REF79_GRID;
