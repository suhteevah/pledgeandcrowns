// SPDX-License-Identifier: MIT
// REF-47 v2 - The Cellarer on the Tavern kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: Tavern shirtsleeves and bronze apron, no robe (REF-71), with a bronze
// knit cap for the cold cellar, clean-shaven. Lesson prop (unwrap_or_else): a
// cask at frame-left, brew showing at its rim, and a full ladle in the
// frame-right hand. The default is only drawn when it is asked for.
//
// Legend: X coalblack  P pink quartz  R dusty rose  W wineflesh  B basalt
//         V aged paper  C parchment cream  G old gold  L brass leaf
//         N antique brass  Z bronze  U bog umber

const REF47_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '..............XXXXXXXXXX........', //  2 knit cap
  '.............XZZNNZZZZZZX.......', //  3
  '.............XPPPPPPPPRRX.......', //  4
  '.............XPPPPPPPPPRX.......', //  5
  '.............XPXXPPPPXXRX.......', //  6 brows
  '.............XPPXPPPPXPRX.......', //  7 eyes
  '.............XPPPPRRPPPWX.......', //  8
  '.............XPPRRRRRRPWX.......', //  9
  '..............XXPRRRRWXX........', // 10
  '...........XXXCCCVVVVCCCVXXX....', // 11 shoulders
  '.........XXCCCCZZZZZZZZCCCVVXX..', // 12 apron bib
  '........XCCCCVXNNZZZZZZXCCCVVX..', // 13
  'XXXXXXXXXCCCVVXNZZZZZZZXCCVVVX..', // 14 cask rim
  'XGGLGGGXXPPRXXXNZZZZZZZXXXPPRX..', // 15 brew level, forearms
  'XNNNNNNXXPRRX.XNZZZZZZUX.XPRRX..', // 16 hoop
  'XZZUZZUX.XXX..XNZZZZZZUX..XXX...', // 17 staves
  'XZZUZZUX.....XXNZZZZZZUXX.XNX...', // 18 ladle handle
  'XZZUZZUX....XBXNZZGGZZUXBXXNX...', // 19
  'XZZUZZUX....XBXNZZZZZZUXBXXGGGX.', // 20 ladle cup
  'XZZUZZUX....XBXNZZZZZZUXBX.XXX..', // 21
  'XNNNNNNX....XBXXXXXXXXXXBX......', // 22 hoop, apron hem
  'XZZUZZUX....XBBBBBXXBBBBBX......', // 23
  'XZZUZZUX....XBBBBX..XBBBBX......', // 24
  'XZZUZZUX....XBBBBX..XBBBBX......', // 25
  'XUUUUUUX....XUUUUX..XUUUUX......', // 26
  'XXXXXXXX...XUUUUUX..XUUUUUX.....', // 27
  '...........XXXXXXX..XXXXXXX.....', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF47_V2_GRID = REF47_V2_GRID;
