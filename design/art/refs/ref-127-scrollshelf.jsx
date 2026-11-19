// SPDX-License-Identifier: MIT
// REF-127 - Scroll shelf, Act 3 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A bronze shelf unit: a row of books with coloured spines on top,
// four rolled scrolls seen end-on in the middle, two ledgers lying flat
// below. Also meant for the Mage's Tower and the Library. Bottom-centre
// anchor; solid over the unit.
//
// Legend: X coalblack  Z bronze  N antique brass  U bog umber
//         W wineflesh  O oxblood  E forest  G old gold  L brass leaf
//         C parchment cream  V aged paper  < cobalt deep  > cobalt

const REF127_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', //  3
  '....XNNNNNNNNNNNNNNNNNNNNNNX....', //  4
  '....XUUUUUUUUUUUUUUUUUUUUUUX....', //  5
  '....XUWW<<EEGGOO>>WWEE<<GGUX....', //  6 books
  '....XULWL<LELGLOL>LWLEL<LGUX....', //  7
  '....XUWW<<EEGGOO>>WWEE<<GGUX....', //  8
  '....XUWW<<EEGGOO>>WWEE<<GGUX....', //  9
  '....XUWW<<EEGGOO>>WWEE<<GGUX....', // 10
  '....XNNNNNNNNNNNNNNNNNNNNNNX....', // 11
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 12
  '....XUUUUUUUUUUUUUUUUUUUUUUX....', // 13
  '....XUUXXXUUXXXUUXXXUUXXXUUX....', // 14 scrolls
  '....XUXCVCXXCVCXXCVCXXCVCXUX....', // 15
  '....XUXVXVXXVXVXXVXVXXVXVXUX....', // 16
  '....XUXCVCXXCVCXXCVCXXCVCXUX....', // 17
  '....XUUXXXUUXXXUUXXXUUXXXUUX....', // 18
  '....XNNNNNNNNNNNNNNNNNNNNNNX....', // 19
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 20
  '....XUUUUUUUUUUUUUUUUUUUUUUX....', // 21
  '....XUUUUUUUUUUUUXXXXXXXXXUX....', // 22 ledgers
  '....XUUUUUUUUUUUUX>>>>>>>XUX....', // 23
  '....XUXXXXXXXXXXXX<<<<<<<XUX....', // 24
  '....XUXWWWWWWWWWXX<<<G<<<XUX....', // 25
  '....XUXOOOOOOOOOXX<<<<<<<XUX....', // 26
  '....XUXCCCCCCCCCXXCCCCCCCXUX....', // 27
  '....XUXXXXXXXXXXXXXXXXXXXXUX....', // 28
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF127_GRID = REF127_GRID;
