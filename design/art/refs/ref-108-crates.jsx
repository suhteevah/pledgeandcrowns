// SPDX-License-Identifier: MIT
// REF-108 - Crate stack, Market Row district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// Three brass-wood crates, two on the ground and one on top, each with
// a bronze frame and one diagonal brace. Also meant for the Guildhall
// and the Wharf. Bottom-centre anchor; solid over the stack.
//
// Legend: X coalblack  N antique brass  G old gold  Z bronze

const REF108_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '.........XXXXXXXXXXXX...........', //  4 top crate
  '.........XGGGGGGGGGGX...........', //  5
  '.........XZZNNNNNNNZX...........', //  6
  '.........XZNZNNNNNNZX...........', //  7
  '.........XZNNZNNNNNZX...........', //  8
  '.........XZNNNZNNNNZX...........', //  9
  '.........XZNNNNZNNNZX...........', // 10
  '.........XZNNNNNZNNZX...........', // 11
  '.........XZNNNNNNZNZX...........', // 12
  '.........XZNNNNNNNZZX...........', // 13
  '.........XZZZZZZZZZZX...........', // 14
  '.........XXXXXXXXXXXX...........', // 15
  '...XXXXXXXXXXXXXX...............', // 16 ground crates
  '...XGGGGGGGGGGGGX...............', // 17
  '...XZZNNNNNNNNNZXXXXXXXXXXXXX...', // 18
  '...XZNZNNNNNNNNZXXGGGGGGGGGGX...', // 19
  '...XZNNZNNNNNNNZXXZZNNNNNNNZX...', // 20
  '...XZNNNZNNNNNNZXXZNZNNNNNNZX...', // 21
  '...XZNNNNZNNNNNZXXZNNZNNNNNZX...', // 22
  '...XZNNNNNZNNNNZXXZNNNZNNNNZX...', // 23
  '...XZNNNNNNZNNNZXXZNNNNZNNNZX...', // 24
  '...XZNNNNNNNZNNZXXZNNNNNZNNZX...', // 25
  '...XZNNNNNNNNZNZXXZNNNNNNZNZX...', // 26
  '...XZNNNNNNNNNZZXXZNNNNNNNZZX...', // 27
  '...XZZZZZZZZZZZZXXZZZZZZZZZZX...', // 28
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF108_GRID = REF108_GRID;
