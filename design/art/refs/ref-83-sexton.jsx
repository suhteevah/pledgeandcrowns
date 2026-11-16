// SPDX-License-Identifier: MIT
// REF-83 - The Sexton on the Library kit (REF-79), 32x32, idle.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/sexton_idle_0.png.
//
// Candle-keeper. Three pixels shorter and two wider than the kit column.
// Lesson prop (drop_trait): a long brass candle snuffer in the left hand -
// every candle is put out in its turn - and one candle still lit at the hip,
// the one that was forgotten and never snuffed.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow  P pink quartz
//         S stone grey  B basalt  N antique brass  G old gold  L brass leaf

const REF83_GRID = [
  '................................', //  0
  '....NNN.........................', //  1 snuffer bell
  '...NGGGN........................', //  2
  '...N...N........................', //  3
  '.....N........XXXX..............', //  4 hood peak
  '.....N......XXYOOYXX............', //  5
  '.....N.....XYYOOYYYYX...........', //  6
  '.....N....XYYOYYYYYYKX..........', //  7
  '.....N....XYO*KKKK*YKX..........', //  8
  '.....N....XYO*CKKC*YKX..........', //  9 eyes
  '.....N....XYO*KKKK*YKX..........', // 10
  '.....N....XYYO*KK*YYKX..........', // 11
  '.....N..XYYYYO*VV*YYYKKX........', // 12 broad shoulders
  '.....N.XYYYYOCVVVVSYYYKKX.......', // 13
  '.....N.XYYOYYCV*%VSYYOKKX.......', // 14 sigil
  '.....N.XYYOOYCV**VSYOOKKX.......', // 15
  '.....N.XYYYOOYCVVSYOOYKKX.......', // 16
  '.....NPXYYYYOOYCVSPPOYKKX.......', // 17 hands
  '.....N.XYYYYKKKCVSKKYYKKX.......', // 18
  '.....N.XYSBSBSBSBSBSBSBYX.......', // 19 belt chain
  '.....N.XYYYSYCVVVVSGLGKKX.......', // 20 flame and its glow
  '.....N.XYYYBYCVVVVSYGYKKX.......', // 21 the candle still lit
  '.....N.XYYYSYCVVVVSYCYKKX.......', // 22
  '.....N.XYYYOYCVVVVSYCYKKX.......', // 23
  '.....N.XYYOYYYCVVSYYYYKKX.......', // 24
  '.....N.XYYOYYYYYYYYYKYKKX.......', // 25
  '.....N.XYYOYYYYYYYYYKYKKX.......', // 26
  '.....N.XOOOOOOOOOOOOOOKKX.......', // 27 hem
  '.....N.XYYYYYYYYYYYYYYKKX.......', // 28
  '.....N.XXXXXXXXXXXXXXXXXX.......', // 29
  '................................', // 30
  '................................', // 31
];

window.REF83_GRID = REF83_GRID;
