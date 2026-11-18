// SPDX-License-Identifier: MIT
// REF-115 - Chicken coop, Act 1 Hearthstone Village prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A small brass-wood coop on legs with a wine roof to match the
// cottages, a dark doorway, a ramp, and one cream hen beside it (one
// alarm-scarlet pixel for her comb). For the Chicken Coop mission.
// Bottom-centre anchor; solid over the coop, not the hen.
//
// Legend: X coalblack  N antique brass  Z bronze  G old gold
//         O oxblood  W wineflesh  Y crypt
//         C parchment cream  V aged paper  ! alarm scarlet

const REF115_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..............XXXX..............', //  3 roof
  '.............XWOOYX.............', //  4
  '............XWYYYYYX............', //  5
  '...........XWOOOOOOYX...........', //  6
  '..........XWOOOOOOOOYX..........', //  7
  '.........XWYYYYYYYYYYYX.........', //  8
  '........XWOOOOOOOOOOOOYX........', //  9
  '.......XWOOOOOOOOOOOOOOYX.......', // 10
  '......XWYYYYYYYYYYYYYYYYYX......', // 11
  '.....XWOOOOOOOOOOOOOOOOOOYX.....', // 12
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 13 coop
  '.......XNNNZNNNNNNNNZNNNX.......', // 14
  '.......XNNNZNNNNNNNNZNNNX.......', // 15
  '.......XNNNZNNNNNNNNZNNNX.......', // 16
  '.......XNNNZNXXXXXXNZNNNX.......', // 17
  '.......XNNNZNXXXXXXNZNNNX.......', // 18
  '.......XNNNZNXXXXXXNZNNNX.......', // 19
  '.......XNNNZNXXXXXXNZNNNX.......', // 20
  '.......XNNNZNXXXXXXNZNNNX.......', // 21
  '.......XNNNZNXXXXXXNZNNNX.......', // 22
  '.......XNNNZNXXXXXXNZNNNX.......', // 23
  '.......XZZZZZXXXXXXZZZZZX.!X....', // 24 hen
  '.......XXXXXXXXXXXXXXXXXXXCCX...', // 25
  '........XZX.XZZZZX...XZX.XCCCX..', // 26 ramp
  '........XZX..XZZZZX..XZX.XVCCX..', // 27
  '........XZX...XZZZZX.XZX..XGX...', // 28
  '........XXX....XZZZZXXXX........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF115_GRID = REF115_GRID;
