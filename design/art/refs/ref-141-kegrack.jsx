// SPDX-License-Identifier: MIT
// REF-141 - Keg rack, Act 5 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/keg_rack.png.
//
// Two kegs lying end-on in a bronze rack, one above the other, each
// with a brass head and a gold tap. Bottom-centre anchor; solid over
// the rack.
//
// Legend: X coalblack  Z bronze  N antique brass  G old gold
//         U bog umber

const REF141_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '..............XXXX..............', //  5 upper keg
  '...XXX....XXXNNNNNZXXX....XXX...', //  6
  '...XZX...XNNNNXXXXZZZZX...XZX...', //  7
  '...XZX..XNNNXGGGNNNXZZZX..XZX...', //  8
  '...XZX.XXNNXGGGNNNNNXZZXX.XZX...', //  9
  '...XZX.XNNNXGNNXXNNZXUUUX.XZX...', // 10
  '...XZX.XXZZXNNNNNZZZXUUXX.XZX...', // 11
  '...XZX..XZZZXNNNZZZXUUUX..XZX...', // 12
  '...XZX...XZZZZXXXXUUUUX...XZX...', // 13
  '...XZX....XXXZUGNUUXXX....XZX...', // 14
  '...XZX........XXXX........XZX...', // 15
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 16
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 17 lower keg
  '...XZX....XXXNNNNNZXXX....XZX...', // 18
  '...XZX...XNNNNXXXXZZZZX...XZX...', // 19
  '...XZX..XNNNXGGGNNNXZZZX..XZX...', // 20
  '...XZX.XXNNXGGGNNNNNXZZXX.XZX...', // 21
  '...XZX.XNNNXGNNXXNNZXUUUX.XZX...', // 22
  '...XZX.XXZZXNNNNNZZZXUUXX.XZX...', // 23
  '...XZX..XZZZXNNNZZZXUUUX..XZX...', // 24
  '...XZX...XZZZZXXXXUUUUX...XZX...', // 25
  '...XZX....XXXZUGNUUXXX....XZX...', // 26
  '...XZX........XXXX........XZX...', // 27
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 28
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF141_GRID = REF141_GRID;
