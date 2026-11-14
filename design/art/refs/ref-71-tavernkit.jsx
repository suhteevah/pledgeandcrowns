// SPDX-License-Identifier: MIT
// REF-71 - Costume kit exemplar, Act 5 Tavern of Tribulations (Barkeep), 32x32.
// APPROVED by Matt 2026-11-13 (tranche 1) as a kit exemplar. NOT wired to any NPC; tranche 2 redraws this Act's cast on it.
//
// Kit rule for the Tavern cast: shirtsleeves in parchment cream, a bronze
// apron, bare forearms, no robe. Silhouette = stout and wide, bald or capped.
// Zone roles: Bronze, Antique brass, Basalt, cream. No magic violet.
//
// Legend: X coalblack  P pink quartz  R dusty rose  W wineflesh  B basalt
//         C parchment cream  V aged paper
//         N antique brass  Z bronze  U bog umber  G old gold

const REF71_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...........XXXXXXXXXX...........', //  3 bald crown
  '..........XPPPPPPPPRRX..........', //  4
  '..........XPPPPPPPPPRX..........', //  5
  '..........XPXXPPPPXXRX..........', //  6 brows
  '..........XPPXPPPPXPRX..........', //  7 eyes
  '..........XPPPPRRPPPWX..........', //  8
  '..........XBBBBBBBBBBX..........', //  9 moustache
  '..........XBPPRRRRPPBX..........', // 10
  '...........XXPRRRRWXX...........', // 11 chin
  '........XXXCCCVVVVCCCVXXX.......', // 12 shoulders
  '......XXCCCCZZZZZZZZCCCVVXX.....', // 13 apron bib
  '.....XCCCCVXNNZZZZZZXCCCVVX.....', // 14
  '.....XCCCVVXNZZZZZZZXCCVVVX.....', // 15 rolled sleeves
  '.....XPPRXXXNZZZZZZZXXXPPRX.....', // 16 forearms
  '.....XPRRX.XNZZZZZZUX.XPRRX.....', // 17
  '......XXX..XNZZZZZZUX..XXX......', // 18
  '..........XXNZZZZZZUXX..........', // 19
  '.........XBXNZZGGZZUXBX.........', // 20 apron pocket
  '.........XBXNZZZZZZUXBX.........', // 21
  '.........XBXNZZZZZZUXBX.........', // 22
  '.........XBXXXXXXXXXXBX.........', // 23 apron hem
  '.........XBBBBBXXBBBBBX.........', // 24 trousers
  '.........XBBBBX..XBBBBX.........', // 25
  '.........XBBBBX..XBBBBX.........', // 26
  '.........XUUUUX..XUUUUX.........', // 27 boots
  '........XUUUUUX..XUUUUUX........', // 28
  '........XXXXXXX..XXXXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF71_GRID = REF71_GRID;
