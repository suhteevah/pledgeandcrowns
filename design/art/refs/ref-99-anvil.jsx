// SPDX-License-Identifier: MIT
// REF-99 - Anvil on a stump, Act 6 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// The Forge's working prop: a stone-grey anvil with its horn to the left
// and a lit aged-paper face, on a bronze-wood stump with a brass-leaf cut
// top. One specular-white pixel on the face. Bottom-centre anchor; solid
// over the stump's footprint so the smith can stand beside it.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         N antique brass  G old gold  Z bronze  U bog umber
//         # specular white

const REF99_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '................................', //  7
  '................................', //  8
  '....XXXXXXXXXXXXXXXXXXXXX.......', //  9 anvil face
  '....XVV#VVVVVVVVVVVVVVVVX.......', // 10
  '.....XSSSSSSSSSSSSSSSSSBX.......', // 11 horn
  '.......XXSSSSSSSSSSSSSBBX.......', // 12
  '.........XXSSSSSSSSSBBXX........', // 13
  '............XSSSSSBX............', // 14 waist
  '............XSVSSSBX............', // 15
  '............XSSSSSBX............', // 16
  '.........XXXXSSSSSBXXXX.........', // 17 foot
  '.........XSSSSSSSSSSBBX.........', // 18
  '.......XXXXXXXXXXXXXXXXXX.......', // 19
  '.......XNNNNGNNNNNNGNNNNX.......', // 20 stump, cut top
  '.......XNGNNNNNNGNNNNNNZX.......', // 21
  '.......XZZZZZZZZZZZZZZZUX.......', // 22 bark
  '.......XZZUZZZZZZZUZZZZUX.......', // 23
  '.......XZZUZZZZZZZUZZZZUX.......', // 24
  '.......XZZUZZZZZZZUZZZZUX.......', // 25
  '.......XZZZZZZUZZZZZZZZUX.......', // 26
  '.......XZZZZZZUZZZZZZZZUX.......', // 27
  '.......XZZZZZZUZZZZZZZZUX.......', // 28
  '.......XUUUUUUUUUUUUUUUUX.......', // 29
  '........XXXXXXXXXXXXXXXX........', // 30
  '................................', // 31
];

window.REF99_GRID = REF99_GRID;
