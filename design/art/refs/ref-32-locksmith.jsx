// SPDX-License-Identifier: MIT
// REF-32 - The Locksmith, 32x32, idle, transparent bg.
// A Crypt-robed tradesman holds up a ring of gold keys in the right hand,
// one key raised and singled out - the one key that fits the lock. The lock
// either opens (Some) or it does not (None).
// Mission: if_let - match one variant, ignore the rest.
//
// Legend:
//   X coalblack (outline, key-ring metal core)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (collar stud, key shadow)   Z bronze (key-ring band)
//   G old gold (keys)   L brass leaf (raised key spec - the one that fits)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF32_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSSVVSSSX.............', //  3 hair
  '........XSSVVVVVVSSX............', //  4
  '........XSPPPPPPPPSX...GLG......', //  5 forehead | raised key bow (G/L)
  '........XPPPPPPPPRRX...GXG......', //  6 raised key (the one that fits)
  '........XPBPPPPBPRWX...GXG......', //  7 brow / eyes | key shaft
  '........XPPRRRRRRRPX....G.......', //  8 key bit
  '........XXPRRRRRRPXX...GG.......', //  9 jaw | key teeth
  '........XNPPPRRRRPNX............', // 10 collar studs (N)
  '.......XYYYYYYYYYYYYX...........', // 11 shoulders
  '......XYYYOOOOOOOOYYYX..........', // 12 robe upper lit
  '......XYOOOWWWWOOOOYOXPPPP......', // 13 robe lit | right hand rising
  '......XYOOWWWWWWOOOYOXPRRPX.....', // 14 hand grips ring
  '......XYOOOWWWWOOOOYOXPPPPX.....', // 15
  '......XYOOOOOOOOOOOYOX.ZGZ......', // 16 key-ring band top (Z)
  '......XYOOOOOOOOOOOYOXZGNGZ.....', // 17 ring + hanging keys (G)
  '......XXYYOOOOOOOOYYXXZGXGZ.....', // 18 ring lower
  '......XYYOOOWWWWOOOYYX.GXG......', // 19 key 1 hangs
  '......XYYOOOOOOOOOOYYX.GXG......', // 20 key 2 hangs
  '......XYYOOOOOOOOOOYYX.GGG......', // 21 key bits
  '......XXXNGGGGGGGGNXXX..........', // 22 belt buckle (G)
  '......XYYOOOOOOOOOOYYX..........', // 23 robe lower
  '......XYYOOOWWWWOOOYYX..........', // 24
  '......XYYOOOOOOOOOOYYX..........', // 25
  '......XYYOOOOOOOOOOYYX..........', // 26
  '......XYFFFFFFFFFFFFYX..........', // 27 hem
  '......XFFFFXX..XXFFFFX..........', // 28 leg split
  '......XZZZZX....XZZZZX..........', // 29 boots
  '......XZUUZX....XZUUZX..........', // 30
  '......XXXXXX....XXXXXX..........', // 31 soles
];

window.REF32 = function REF32({ scale = 8 }) {
  return <PixelArt grid={REF32_GRID} scale={scale} title="REF-32 The Locksmith" />;
};
window.REF32_GRID = REF32_GRID;
window.REF32_ROLES = ['X','Y','O','W','R','P','B','S','V','N','Z','G','L','U','F'];
