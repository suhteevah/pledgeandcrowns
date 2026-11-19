// SPDX-License-Identifier: MIT
// REF-109 - Barrels, Market Row district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/barrels.png.
//
// Three bronze barrels with basalt iron bands and brass-lit staves, one
// standing behind the other two. Also meant for the Tavern and the
// Wharf. Bottom-centre anchor; solid over the group.
//
// Legend: X coalblack  Z bronze  N antique brass  G old gold
//         U bog umber  B basalt

const REF109_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '............XXXXXXXX............', //  5 back barrel
  '...........XNGGGGGGNX...........', //  6
  '..........XZNNNNNNNNZX..........', //  7
  '..........XBBBBBBBBBBX..........', //  8
  '..........XZNZZNZZNZZX..........', //  9
  '..........XZNZZNZZNZZX..........', // 10
  '..........XZNZZNZZNZZX..........', // 11
  '..........XZNZZNZZNZZX..........', // 12
  '..........XBBBBBBBBBBX..........', // 13
  '..........XZNZZNZZNZZX..........', // 14
  '..........XZNZZNZZNZZX..........', // 15
  '.....XXXXXXXXZUUZUUXXXXXXXX.....', // 16 front barrels
  '....XNGGGGGGNXBBBBXNGGGGGGNX....', // 17
  '...XZNNNNNNNNZXXXXZNNNNNNNNZX...', // 18
  '...XBBBBBBBBBBX..XBBBBBBBBBBX...', // 19
  '...XZNZZNZZNZZX..XZNZZNZZNZZX...', // 20
  '...XZNZZNZZNZZX..XZNZZNZZNZZX...', // 21
  '...XZNZZNZZNZZX..XZNZZNZZNZZX...', // 22
  '...XZNZZNZZNZZX..XZNZZNZZNZZX...', // 23
  '...XBBBBBBBBBBX..XBBBBBBBBBBX...', // 24
  '...XZNZZNZZNZZX..XZNZZNZZNZZX...', // 25
  '...XZNZZNZZNZZX..XZNZZNZZNZZX...', // 26
  '...XUUZUUZUUZUX..XUUZUUZUUZUX...', // 27
  '....XBBBBBBBBX....XBBBBBBBBX....', // 28
  '.....XXXXXXXX......XXXXXXXX.....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF109_GRID = REF109_GRID;
