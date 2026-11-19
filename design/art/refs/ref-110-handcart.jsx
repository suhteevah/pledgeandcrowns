// SPDX-License-Identifier: MIT
// REF-110 - Handcart, Market Row district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/handcart.png.
//
// A brass-wood handcart seen from the side: one spoked wheel with a
// stone-grey hub, a handle to the left, a rest leg to the right, two
// wine-cloth bundles on top. Bottom-centre anchor; solid over the body
// and wheel.
//
// Legend: X coalblack  N antique brass  G old gold  Z bronze
//         W wineflesh  R dusty rose  O oxblood  S stone grey

const REF110_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '................................', //  7
  '...........XXXX..XXXXX..........', //  8 sacks
  '..........XWWRWXXWRWWWX.........', //  9
  '..........XWWWWXXWWWWWX.........', // 10
  '..X.......XOOOOXXOOOOOX.........', // 11
  '..ZXX...XXXXXXXXXXXXXXXXXXXX....', // 12 body
  '..XZZXX.XGGGGGGGGGGGGGGGGGGX....', // 13
  '...XXZZXXNNNNNZNNNNNNZNNNNNX....', // 14
  '.....XXZXNNNNNZNNNNNNZNNNNNX....', // 15
  '.......XXZZZZZZZZZZZZZZZZZZX....', // 16
  '........XNNNNNZNNNNNNZNNNNNX....', // 17
  '........XNNNNNZNNNNNNZNNNNNX....', // 18
  '........XZZZZZZZZZZZZZZZZZZX....', // 19
  '........XXXXXXXXXXXXXXXXXXXX....', // 20 wheel
  '.............XNNNNNNX....XZX....', // 21
  '............XNZXXXXZNX...XZX....', // 22
  '............XNXZXXZXNX...XZX....', // 23
  '............XNXXSSXXNX...XZX....', // 24
  '............XNXXSSXXNX...XZX....', // 25
  '............XNXZXXZXNX...XZX....', // 26
  '............XNZXXXXZNX...XZX....', // 27
  '.............XNNNNNNX....XZX....', // 28
  '..............XXXXXX.....XXX....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF110_GRID = REF110_GRID;
