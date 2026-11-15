// SPDX-License-Identifier: MIT
// REF-33 v2 - The Porter on the Guildhall kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: Guildhall oxblood (REF-69), cut for labour: a small cap, a hip-length
// tunic with a bronze belt and pine hem, pale hose and bronze boots, so the
// guild's lowest rank still reads as guild. Lesson prop (while_let): one crate
// lifted clear overhead in both hands - the item just popped - which also
// gives him the only raised-arms silhouette in the quarter.
//
// Legend: X coalblack  Y crypt  O oxblood  W wineflesh
//         P pink quartz  R dusty rose  S stone grey  V aged paper
//         G old gold  N antique brass  Z bronze  U bog umber  F pine

const REF33_V2_GRID = [
  '.......XXXXXXXXXXXXXXXXX........', //  0 crate
  '.......XNUUUUUUGUUUUUUNX........', //  1 brackets, carry strap
  '.......XUUUUUUUGUUUUUUUX........', //  2
  '.......XUUZUUUUGUUUUZUUX........', //  3
  '.......XNUUUUUUGUUUUUUNX........', //  4
  '.......XXXXXXXXXXXXXXXXX........', //  5
  '......XPX.XXXXXXXXXX.XPX........', //  6 hands, cap
  '......XRX.XOOWWOOOYX.XRX........', //  7
  '......XOX.XSPPPPPRSX.XOX........', //  8 raised sleeves
  '......XOX.XPXPPPXPRX.XOX........', //  9 eyes
  '......XOX.XPPPRRPPWX.XOX........', // 10
  '......XOXXXXPRRRRWXXXXOX........', // 11
  '......XOOOOOXXXXXXOOOOYX........', // 12 shoulders
  '.......XXOOWOOOOOOOOYXX.........', // 13
  '........XOWWOOOOOOOOYX..........', // 14
  '........XOWOOOOOOOOOYX..........', // 15
  '........XZZZZZNNZZZZZX..........', // 16 bronze belt
  '........XOWOOOOOOOOOYX..........', // 17 short tunic
  '........XOOOOOOOOOOOYX..........', // 18
  '........XFFFFFFFFFFFFX..........', // 19 pine hem
  '........XXXXXXXXXXXXXX..........', // 20
  '.........XVVVX..XVVVX...........', // 21 hose
  '.........XVVSX..XVVSX...........', // 22
  '.........XVVSX..XVVSX...........', // 23
  '.........XVSSX..XVSSX...........', // 24
  '.........XZZZX..XZZZX...........', // 25 boots
  '.........XZZUX..XZZUX...........', // 26
  '........XZZZUX..XZZZUX..........', // 27
  '........XXXXXX..XXXXXX..........', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF33_V2_GRID = REF33_V2_GRID;
