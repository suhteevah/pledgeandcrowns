// SPDX-License-Identifier: MIT
// REF-163 - Tall bookshelf, Act 9 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/tall_bookshelf.png.
//
// A full-height bronze bookcase, four shelves of books in the whole
// palette with a brass-leaf mark on each spine. Stand them in rows
// either side of the runner. Bottom-centre anchor; solid over the
// case.
//
// Legend: X coalblack  Z bronze  N antique brass  U bog umber
//         L brass leaf  O oxblood  W wineflesh  Y crypt  G old gold
//         E forest  F pine  < cobalt deep  > cobalt

const REF163_GRID = [
  '..XNNNNNNNNNNNNNNNNNNNNNNNNNNX..', //  0 cornice
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', //  1
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', //  2
  '...XUUUUUUUUUUUUUUUUUUUUUUUUX...', //  3 shelves
  '...XUU<<EEWWUU>>GGOOUUWW<<OOX...', //  4
  '...XOOL<LELWYYL>LGLOFFLWL<LOX...', //  5
  '...XLO<<EEWWLY>>GGOOLFWW<<OOX...', //  6
  '...XOO<<EEWWYY>>GGOOFFWW<<OOX...', //  7
  '...XOO<<EEWWYY>>GGOOFFWW<<OOX...', //  8
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', //  9
  '...XUUUUUUUUUUUUUUUUUUUUUUUUX...', // 10
  '...X>>GGOOUUWW<<OOUU<<EEWWUUX...', // 11
  '...XL>LGLOFFLWL<LOOOL<LELWYYX...', // 12
  '...X>>GGOOLFWW<<OOLO<<EEWWLYX...', // 13
  '...X>>GGOOFFWW<<OOOO<<EEWWYYX...', // 14
  '...X>>GGOOFFWW<<OOOO<<EEWWYYX...', // 15
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 16
  '...XUUUUUUUUUUUUUUUUUUUUUUUUX...', // 17
  '...X<<OOUU<<EEWWUU>>GGOOUUWWX...', // 18
  '...XL<LOOOL<LELWYYL>LGLOFFLWX...', // 19
  '...X<<OOLO<<EEWWLY>>GGOOLFWWX...', // 20
  '...X<<OOOO<<EEWWYY>>GGOOFFWWX...', // 21
  '...X<<OOOO<<EEWWYY>>GGOOFFWWX...', // 22
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 23
  '...XUUUUUUUUUUUUUUUUUUUUUUUUX...', // 24
  '...XWWUU>>GGOOUUWW<<OOUU<<EEX...', // 25
  '...XLWYYL>LGLOFFLWL<LOOOL<LEX...', // 26
  '...XWWLY>>GGOOLFWW<<OOLO<<EEX...', // 27
  '...XWWYY>>GGOOFFWW<<OOOO<<EEX...', // 28
  '...XWWYY>>GGOOFFWW<<OOOO<<EEX...', // 29
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', // 30
  '................................', // 31
];

window.REF163_GRID = REF163_GRID;
