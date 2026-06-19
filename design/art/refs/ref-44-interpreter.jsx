// SPDX-License-Identifier: MIT
// REF-44 - The Interpreter, 32x32, idle, transparent bg.
// Holds two small signs - one in each raised hand - of DIFFERENT make:
// frame-left a Forest-green token (the source complaint), frame-right a
// Teal token (the translated complaint). Between them his hands gesture
// the conversion, turning the one into the other - `impl From<A> for B`.
// One type's value flows in, another type's value comes out, losslessly.
// Crypt robe, the two token colors are the only accents (forest -> teal),
// no magic violet (translation is craft, not spellcraft).
// Mission: from_error - implement `From` to convert one error into another.
//
// Legend:
//   X coalblack (outline, sign edges)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (belt band)   G old gold (belt buckle)
//   E forest (source token)   H spring meadow (source token hi)
//   T main teal (target token)   I bright teal (target token hi)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF44_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX............', //  8 jaw
  '.XEEX...XYPPPRRRRPYX...XTTX.....', //  9 left token (E) | right token (T)
  'XEHEX..XYYYYYYYYYYYYX..XTITX....', // 10 source hi (H) | target hi (I)
  'XEHEX.XYYYOOOOOOOOYYYX.XTITX....', // 11
  'XEEEXPXYYYOOOWWWWOOOYXPXTTTX....', // 12 hands grip tokens (P)
  '.XXXPRPYOOOWWWWWWOOOYPRPXXX.....', // 13 arms raise both signs (P/R)
  '....PPPYOOOOOOOOOOOOYPPP........', // 14 hands
  '......XYYOOOWWWWWWOOYYX.........', // 15 chest lit
  '......XXNGGGGGGGGGGNXX..........', // 16 belt band (N) buckle (G)
  '......XKYYOOOOOOOOYYKX..........', // 17 robe lower
  '......XKYYOOOWWWWOOYYKX.........', // 18
  '.....XKKYYOOOOOOOOOOYYKKX.......', // 19 robe widens
  '.....XKYYYOOOWWWWWWOOYYKX.......', // 20
  '.....XKYYYOOOOOOOOOOOYYKX.......', // 21
  '.....XKYYYOOOOOOOOOOOYYKX.......', // 22
  '....XKKYYYOOOOOOOOOOOYYKKX......', // 23 widest
  '....XKYYFFFFFFFFFFFFFFYKX.......', // 24 hem
  '....XKFFFFFFXXX.XXXFFFFFKX......', // 25 leg split
  '....XXZZZZZXX.....XXZZZZX.......', // 26 boots
  '.....XZUUUZX.......XZUUZX.......', // 27
  '.....XXXXXXX.......XXXXXX.......', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF44 = function REF44({ scale = 8 }) {
  return <PixelArt grid={REF44_GRID} scale={scale} title="REF-44 The Interpreter" />;
};
window.REF44_GRID = REF44_GRID;
window.REF44_ROLES = ['X','Y','O','W','R','P','B','S','V','N','G','E','H','T','I','U','F'];
