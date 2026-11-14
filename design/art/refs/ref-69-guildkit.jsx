// SPDX-License-Identifier: MIT
// REF-69 - Costume kit exemplar, Act 3 Guildhall Quarter (Guildmaster), 32x32.
// APPROVED by Matt 2026-11-13 (tranche 1) as a kit exemplar. NOT wired to any NPC; tranche 2 redraws this Act's cast on it.
//
// Kit rule for the Guildhall cast: flat oxblood beret, aged-paper ruff, long
// oxblood coat with a gold chain of office and gold hem. Silhouette = wide hat
// over an A-line coat. Zone roles: Oxblood, Wineflesh, Aged paper, Old gold.
//
// Legend: X coalblack  Y crypt  O oxblood  W wineflesh  K inkblood
//         P pink quartz  R dusty rose  S stone grey  V aged paper  C cream
//         G old gold  L brass leaf  N antique brass  B basalt

const REF69_GRID = [
  '................................', //  0
  '................................', //  1
  '.........XXXXXXXXXXXX...........', //  2 beret
  '........XOOWWWWOOOOOYX..........', //  3
  '........XOWWWOOOOGLOYX..........', //  4 guild badge
  '.........XXXXXXXXXXXXX..........', //  5 brim
  '..........XSPPPPPPRSX...........', //  6
  '..........XSPXPPPXRSX...........', //  7 eyes
  '..........XPPPPRPPRWX...........', //  8
  '..........XXPRRRRRWXX...........', //  9 jaw
  '.........XVVVVCCVVVVVX..........', // 10 ruff
  '.......XXOOVVVVVVVVOOYXX........', // 11 shoulders
  '......XOOWOOGOOOOOGOOOYYX.......', // 12 chain of office
  '......XOWWOOOGOOOGOOOOYYX.......', // 13
  '......XOWWOOOOGLGOOOOOYYX.......', // 14 medallion
  '......XOWOOOOOOGOOOOOOYYX.......', // 15
  '......XPXOOOOOOOOOOOOOYXPX......', // 16 hands
  '......XRXOOWOOOOOOOOOOYXRX......', // 17
  '.......XXNNNNNGLNNNNNNXX........', // 18 belt
  '.......XOOWOOOOOOOOOOYYX........', // 19 coat skirt
  '.......XOOWOOOOOOOOOOYYX........', // 20
  '......XOOWWOOOOOOOOOOOYYX.......', // 21
  '......XOOWOOOOOOOOOOOOYYX.......', // 22
  '......XGGGGGGGGGGGGGGGGGX.......', // 23 gold hem
  '......XXXXXXXXXXXXXXXXXXX.......', // 24
  '.........XYYYX...XYYYX..........', // 25 legs
  '.........XYYKX...XYYKX..........', // 26
  '.........XBBBX...XBBBX..........', // 27 boots
  '........XBBBBX...XBBBBX.........', // 28
  '........XXXXXX...XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF69_GRID = REF69_GRID;
