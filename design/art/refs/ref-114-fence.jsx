// SPDX-License-Identifier: MIT
// REF-114 - Fence, The Commons and Hearthstone prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Two brass-wood rails on two pointed bronze posts. The rails run to
// both edges, so pieces placed side by side join into one fence.
// Bottom-centre anchor; solid along the rails.
//
// Legend: X coalblack  N antique brass  Z bronze

const REF114_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '.......X...............X........', //  7 post tops
  '......XNX.............XNX.......', //  8
  '.....XXXXX...........XXXXX......', //  9
  '.....XNZZX...........XNZZX......', // 10
  '.....XNZZX...........XNZZX......', // 11
  '.....XNZZX...........XNZZX......', // 12
  'XXXXXXNZZXXXXXXXXXXXXXNZZXXXXXXX', // 13 upper rail
  'NNNNNXNZZXNNNNNNNNNNNXNZZXNNNNNN', // 14
  'ZZZZZXNZZXZZZZZZZZZZZXNZZXZZZZZZ', // 15
  'XXXXXXNZZXXXXXXXXXXXXXNZZXXXXXXX', // 16
  '.....XNZZX...........XNZZX......', // 17
  '.....XNZZX...........XNZZX......', // 18
  '.....XNZZX...........XNZZX......', // 19
  'XXXXXXNZZXXXXXXXXXXXXXNZZXXXXXXX', // 20 lower rail
  'NNNNNXNZZXNNNNNNNNNNNXNZZXNNNNNN', // 21
  'ZZZZZXNZZXZZZZZZZZZZZXNZZXZZZZZZ', // 22
  'XXXXXXNZZXXXXXXXXXXXXXNZZXXXXXXX', // 23
  '.....XNZZX...........XNZZX......', // 24
  '.....XNZZX...........XNZZX......', // 25
  '.....XNZZX...........XNZZX......', // 26
  '.....XNZZX...........XNZZX......', // 27
  '.....XXXXX...........XXXXX......', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF114_GRID = REF114_GRID;
