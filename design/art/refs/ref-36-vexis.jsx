// SPDX-License-Identifier: MIT
// REF-36 - Vexis the Archmage, 32x32, idle, transparent bg.
// The tower master. The tallest, most imposing robe in the cast: a full
// Crypt mantle of office, Oxblood primary, Wineflesh lit folds, an Old
// gold collar yoke. In the right hand a tall staff whose finial is a
// small glowing Mage-glow orb ringed in Royal arcane - the one arcane
// accent on the figure, the trait granted to the world below.
// Mission: trait_def - define a `trait` that grants a capability to a type.
//
// Legend:
//   X coalblack (outline, staff shaft)
//   K inkblood (mantle deepest shadow)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair, beard)   V aged paper (hair/beard hi)
//   N antique brass (collar yoke, staff collar, orb ring base)
//   Z bronze (staff shaft)   G old gold (collar disc, finial cup)
//   L brass leaf (collar/finial spec)
//   * royal arcane (orb halo / shadow)   % mage glow (orb core)
//   U bog umber (boot shadow)   F pine (hem under-mantle)
//   . transparent

const REF36_GRID = [
  '.....................%%.........', //  0 orb core top
  '....................*%%*........', //  1 orb halo
  '..........XXXXXXXX..*%%%*.......', //  2 head top | orb
  '.........XSSSVVSSSX.*%%%*.......', //  3 hair | orb
  '........XSSVVVVVVSSX..*%*.......', //  4         orb base
  '........XSPPPPPPPPSX..XGX.......', //  5 forehead | finial cup
  '........XPPPPPPPPRRX..GLG.......', //  6           cup spec
  '........XPBPPPPBPRWX..NGN.......', //  7 brow/eyes | collar (N)
  '........XPPRRRRRRRPX...NX.......', //  8           staff collar
  '........XSPRRRRRRPSX...ZX.......', //  9 beard     shaft (Z)
  '........XSSVRRRRVSSX...ZX.......', // 10
  '.......XYVSSSSSSSSVYX..ZX.......', // 11 beard tip | shaft
  '......XKYYYYYYYYYYYYKX.ZX.......', // 12 mantle shoulders
  '.....XKYYNGGGGGGGGNYYKX.ZX......', // 13 collar yoke (N/G)
  '.....XKYYGLLGGGGLLGYYKX.ZX......', // 14 yoke spec (L)
  '.....XKYOONGGGGGGNOOYKX.ZX......', // 15 robe upper lit
  '.....XKYOOOWWWWWWOOOYKX.ZX......', // 16
  '.....XKYOOOWWWWWWOOOYKX.ZX......', // 17
  '.....XKYOOOOOOOOOOOOYKX.ZX......', // 18
  '.....XKYOOOWWWWWWOOOYKX.ZX......', // 19
  '.....XKXNGGGGGGGGGGNXKX.ZX......', // 20 mantle belt (G)
  '.....XKYOOOOOOOOOOOOYKX.ZX......', // 21 robe lower
  '.....XKYOOOWWWWWWOOOYKX.ZX......', // 22
  '....XKYYOOOOOOOOOOOOYYKX.ZX.....', // 23 mantle widens
  '....XKYYOOOWWWWWWOOOYYKX.ZX.....', // 24
  '....XKYYOOOOOOOOOOOOYYKX.ZX.....', // 25
  '....XKYYOOOOOOOOOOOOYYKX.ZZ.....', // 26 staff foot
  '....XKYFFFFFFFFFFFFFFYKX........', // 27 hem
  '....XKFFFFFXX..XXFFFFFKX........', // 28 leg split
  '....XXZZZZXX....XXZZZZXX........', // 29 boots
  '.....XZUUZX......XZUUZX.........', // 30
  '.....XXXXXX......XXXXXX.........', // 31 soles
];

window.REF36 = function REF36({ scale = 8 }) {
  return <PixelArt grid={REF36_GRID} scale={scale} title="REF-36 Vexis the Archmage" />;
};
window.REF36_GRID = REF36_GRID;
window.REF36_ROLES = ['X','K','Y','O','W','R','P','B','S','V','N','Z','G','L','*','%','U','F'];
