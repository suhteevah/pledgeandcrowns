// SPDX-License-Identifier: MIT
// REF-44 v2 - The Interpreter on the Tavern kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/interpreter_idle_0.png.
//
// Kit: Tavern shirtsleeves and bronze, no robe (REF-71). The house clerk: slim,
// brass spectacles, a short bronze waistcoat in place of the apron. Lesson
// prop (from_error): a card held up in each hand - the forest-green complaint
// as it arrived on the frame-left, and the same complaint converted to the
// house's own old-gold form on the frame-right. Forest = 7 px, on the card only.
//
// Legend: X coalblack  P pink quartz  R dusty rose  W wineflesh  B basalt
//         S stone grey  V aged paper  C parchment cream  G old gold
//         L brass leaf  N antique brass  Z bronze  U bog umber
//         E forest  H spring meadow

const REF44_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX...........', //  2
  '..........XSSVVVVVVSSX..........', //  3 hair
  '..........XSPPPPPPPPSX..........', //  4
  '..........XPNNNPPNNNRX..........', //  5 spectacles
  '..........XPNXNNNNXNRX..........', //  6 eyes behind the lenses
  '..........XPNNNRRNNNWX..........', //  7
  '..........XPPPPRRPPPWX..........', //  8
  '...........XPRRXXRRWX...........', //  9
  '.XXXXX......XXRRRRXX.....XXXXX..', // 10 the two cards
  '.XEHEX.....XCCVVVVCCX....XGLGX..', // 11 source | converted
  '.XEEEX...XXCCCZZZZCCCXX..XGGGX..', // 12
  '..XPX...XCCCCXNZZZZZXCCCVX.XPX..', // 13 raised hands
  '..XRXXXXCCCCVXNZZZZZXCCVVXXXRX..', // 14
  '...XCCCCCCCVVXNZZGZZXCVVVVVCX...', // 15 sleeves
  '....XXXXXXXXXXNZZZZZXXXXXXXX....', // 16
  '..........XCVXNZZZZZXVVX........', // 17 waistcoat
  '..........XCVXXXXXXXXVVX........', // 18
  '..........XNNNNNGNNNNNNX........', // 19 belt
  '..........XBBBBBXXBBBBBX........', // 20 trousers
  '..........XBBBBX..XBBBBX........', // 21
  '..........XBBBBX..XBBBBX........', // 22
  '..........XBBBBX..XBBBBX........', // 23
  '..........XBBBBX..XBBBBX........', // 24
  '..........XBBBBX..XBBBBX........', // 25
  '..........XUUUUX..XUUUUX........', // 26 boots
  '.........XUUUUUX..XUUUUUX.......', // 27
  '.........XXXXXXX..XXXXXXX.......', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF44_V2_GRID = REF44_V2_GRID;
