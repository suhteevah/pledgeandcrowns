// SPDX-License-Identifier: MIT
// REF-35 - The Armorer, 32x32, idle, transparent bg.
// A lean Crypt-tunic smith stands by a weapon rack. In the right hand a
// stone-grey blade (Weapon variant); in the left a small teal vial (Potion
// variant). Two variants of the same enum, each carrying its own data;
// the match arm must handle both. Distinct from the Smith - no anvil.
// Mission: enum_data_match - one enum, two payloads, exhaustive match.
//
// Legend:
//   X coalblack (outline, rack posts, blade core)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (blade, hair)   V aged paper (blade hi, hair hi)
//   Z bronze (rack, hilt, boots)   N antique brass (rack pegs, hilt guard)
//   G old gold (pommel)
//   T main teal (potion fluid)   I bright teal (potion highlight)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF35_GRID = [
  '................................', //  0
  '.....ZZZZZZZZZZ.................', //  1 rack top rail (Z)
  '.....ZNZZZZZZNZ...XXXXXXXX......', //  2 rack pegs (N) | head top
  '.....Z.Z....Z.Z..XSSSVVSSSX.....', //  3 hanging slots | hair
  '.....ZSZ....ZVZ.XSSVVVVVVSSX....', //  4 spare blades on rack
  '.....ZSZ....ZSZ.XSPPPPPPPPSX....', //  5 forehead
  '.....ZSZ....ZSZ.XPPPPPPPPRRX....', //  6
  '.....ZVZ....ZSZ.XPBPPPPBPRWX....', //  7 brow / eyes
  '.....ZZZ....ZZZ.XPPRRRRRRRPX....', //  8
  '...............XXPRRRRRRPXX.....', //  9 jaw
  '...............XYPPPRRRRPYX.....', // 10 collar
  '..............XYYYYYYYYYYYYX....', // 11 shoulders
  '....XV.......XYYYOOOOOOOOYYYX...', // 12 raised blade tip (V) | robe upper
  '....XSX......XYOOOWWWWOOOOYOX...', // 13 blade
  '...PXSXP.....XYOOWWWWWWOOOYOX...', // 14 left hand + vial nearby | robe
  '...PRSRPX....XYOOOWWWWOOOYPRP...', // 15 hand grips blade | right hand out
  '...PPNPPX....XYOOOOOOOOOYPPPP...', // 16 hilt guard (N) | right hand open
  '....NGNXX....XXYYOOOOOYYXXTIX...', // 17 pommel (G) | potient vial (T/I) right
  '....XZX......XYYOOOWWWOOYYXTTX..', // 18 hilt grip (Z) | vial body
  '....XZX......XYYOOOOOOOOYYXTIX..', // 19 | vial fluid (I hi)
  '............XXXNZZZZZZNXXXXTTX..', // 20 belt | vial lower
  '............XYYOOOOOOOOYYX.XX...', // 21 robe lower | vial base
  '............XYYOOOWWWOOOYYX.....', // 22
  '............XYYOOOOOOOOOYYX.....', // 23
  '............XYYOOOOOOOOOYYX.....', // 24
  '............XYFFFFFFFFFFYYX.....', // 25 hem
  '............XFFFFFXXFFFFFFX.....', // 26 leg split
  '............XZZZZX..XZZZZX......', // 27 boots
  '............XZUUZX..XZUUZX......', // 28
  '............XXXXXX..XXXXXX......', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF35 = function REF35({ scale = 8 }) {
  return <PixelArt grid={REF35_GRID} scale={scale} title="REF-35 The Armorer" />;
};
window.REF35_GRID = REF35_GRID;
window.REF35_ROLES = ['X','Y','O','W','R','P','B','S','V','Z','N','G','T','I','U','F'];
