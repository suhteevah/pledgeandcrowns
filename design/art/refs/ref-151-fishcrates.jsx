// SPDX-License-Identifier: MIT
// REF-151 - Fish crates, Act 7 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Three open brass-wood crates, one stacked on two, each packed with
// silver fish on bright-teal ice. Bottom-centre anchor; solid over the
// stack.
//
// Legend: X coalblack  N antique brass  G old gold  Z bronze
//         S stone grey  V aged paper  I bright teal  M mist teal

const REF151_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '........XXXXXXXXXXXXXXXX........', //  6 top crate
  '........XISVVISVVISVVIIX........', //  7
  '........XIVSMIVSMIVSMIIX........', //  8
  '........XIIIIIIIIIIIIIIX........', //  9
  '........XXXXXXXXXXXXXXXX........', // 10
  '........XGGGGGGGGGGGGGGX........', // 11
  '........XNNNZNNNNZNNNNNX........', // 12
  '........XNNNZNNNNZNNNNNX........', // 13
  '........XNNNZNNNNZNNNNNX........', // 14
  '........XZZZZZZZZZZZZZZX........', // 15
  '........XXXXXXXXXXXXXXXX........', // 16
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 17 ground crates
  '.XISVVISVVISVVIXXISVVISVVISVVIX.', // 18
  '.XIVSMIVSMIVSMIXXIVSMIVSMIVSMIX.', // 19
  '.XIIIIIIIIIIIIIXXIIIIIIIIIIIIIX.', // 20
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 21
  '.XGGGGGGGGGGGGGXXGGGGGGGGGGGGGX.', // 22
  '.XNNNZNNNNZNNNNXXNNNZNNNNZNNNNX.', // 23
  '.XNNNZNNNNZNNNNXXNNNZNNNNZNNNNX.', // 24
  '.XNNNZNNNNZNNNNXXNNNZNNNNZNNNNX.', // 25
  '.XNNNZNNNNZNNNNXXNNNZNNNNZNNNNX.', // 26
  '.XNNNZNNNNZNNNNXXNNNZNNNNZNNNNX.', // 27
  '.XZZZZZZZZZZZZZXXZZZZZZZZZZZZZX.', // 28
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 29
  '................................', // 30
  '................................', // 31
];

window.REF151_GRID = REF151_GRID;
