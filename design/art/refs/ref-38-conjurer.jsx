// SPDX-License-Identifier: MIT
// REF-38 - The Conjurer, 32x32, idle, transparent bg.
// Conjures a Pair<T>: one small orb in each hand, IDENTICAL - same
// shape, same Mage-glow color, same Royal-arcane halo. A generic struct
// Pair<T> holds two of the same T; both fields are the one type, so both
// orbs must read as the same conjured thing. Crypt robe, the two orbs are
// the only magic accent and they match exactly (the two violet specks
// sit just under the 5% cap).
// Mission: generic_struct - a struct generic over T (Pair<T>).
//
// Legend:
//   X coalblack (outline)
//   K inkblood (robe deep shadow)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (belt band)   G old gold (belt buckle)
//   * royal arcane (orb halo)   % mage glow (orb core)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF38_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX............', //  8 jaw
  '........XYPPPRRRRPYX............', //  9 collar
  '.......XYYYYYYYYYYYYX...........', // 10 shoulders
  '......XYYYOOOOOOOOYYYX..........', // 11 robe upper
  '.....XYYYOOOWWWWOOOOYYX.........', // 12 robe lit
  '..*..XYYOOOWWWWWWOOOYYX..*......', // 13 orb halos appear out from sleeves
  '.*%*.XYOOOWWWWWWWWOOOYX.*%*.....', // 14 left orb (%) | right orb (%)
  '.*%*PXYOOOOOOOOOOOOOOYXP*%*.....', // 15 orbs at hand height
  '..*PRPYYOOOWWWWWWOOOYYPRP*......', // 16 hands cup the orbs (P/R)
  '...PPPXYOOOOOOOOOOOYXPPP........', // 17 hands
  '......XXNGGGGGGGGNXX............', // 18 belt band (N/G)
  '......XYYOOOOOOOOYYX............', // 19 robe lower
  '......XYYOOOWWWWOOYYX...........', // 20
  '.....XKYYOOOOOOOOOYYKX..........', // 21 robe widens
  '.....XKYYOOOWWWWOOOYYKX.........', // 22
  '.....XKYYOOOOOOOOOOYYKX.........', // 23
  '.....XKYYOOOOOOOOOOYYKX.........', // 24
  '.....XKYFFFFFFFFFFFFYKX.........', // 25 hem
  '.....XKFFFFFXX..XXFFFFKX........', // 26 leg split
  '.....XXZZZZXX....XXZZZZX........', // 27 boots
  '......XZUUZX......XZUUZX........', // 28
  '......XXXXXX......XXXXXX........', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF38 = function REF38({ scale = 8 }) {
  return <PixelArt grid={REF38_GRID} scale={scale} title="REF-38 The Conjurer" />;
};
window.REF38_GRID = REF38_GRID;
window.REF38_ROLES = ['X','K','Y','O','W','R','P','B','S','V','N','G','*','%','U','F'];
