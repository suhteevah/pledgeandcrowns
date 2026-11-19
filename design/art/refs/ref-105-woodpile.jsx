// SPDX-License-Identifier: MIT
// REF-105 - Woodpile, Act 1 Hearthstone Village prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/woodpile.png.
//
// Nine log ends stacked four, three, two (the Firewood Quest counts
// logs), brass and old-gold rings on bronze. Bottom-centre anchor;
// solid over the stack.
//
// Legend: X coalblack  N antique brass  G old gold  Z bronze

const REF105_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '................................', //  7
  '................................', //  8
  '................................', //  9
  '................................', // 10
  '................................', // 11
  '................................', // 12
  '................................', // 13
  '................................', // 14
  '................................', // 15
  '...........XXXX..XXXX...........', // 16 top row, two logs
  '..........XNGGNXXNGGNX..........', // 17
  '..........XGZGNXXGZGNX..........', // 18
  '..........XNGNZXXNGNZX..........', // 19
  '........XXXXXXXXXXXXXXXX........', // 20 three logs
  '.......XNGGNXXNGGNXXNGGNX.......', // 21
  '.......XGZGNXXGZGNXXGZGNX.......', // 22
  '.......XNGNZXXNGNZXXNGNZX.......', // 23
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 24 four logs
  '....XNGGNXXNGGNXXNGGNXXNGGNX....', // 25
  '....XGZGNXXGZGNXXGZGNXXGZGNX....', // 26
  '....XNGNZXXNGNZXXNGNZXXNGNZX....', // 27
  '.....XXXX..XXXX..XXXX..XXXX.....', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF105_GRID = REF105_GRID;
