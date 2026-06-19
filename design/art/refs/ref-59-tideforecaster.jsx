// SPDX-License-Identifier: MIT
// REF-59 - The Tideforecaster, 32x32, idle, transparent bg.
// Stands at the sea's edge, spyglass raised to one eye, a tide-almanac
// tucked under the other arm, GAZING out to a Teal horizon - waiting for a
// tide the almanac promises but that has not yet arrived. She does not hold
// the water in her hands; she holds the PROMISE of it. The tide is coming;
// when it does she will read it, but until then she watches, ready. That is
// an `async fn`: it returns a future - a value that is not here yet but will
// be, awaited rather than blocked-for. Crypt coat, Teal sea and horizon
// line, Old-gold spyglass trim, a Cobalt night-sky sliver, no magic.
// Mission: async_fn - write an async fn that returns a future you await.
//
// Legend:
//   X coalblack (outline, spyglass body)
//   K inkblood (coat deep shadow)
//   Y crypt (coat deep)   O oxblood (coat primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (spyglass body, boots)   N antique brass (belt, spyglass trim)
//   G old gold (spyglass cap, buckle)   L brass leaf (lens spec)
//   U bog umber (boot shadow)   F pine (hem)
//   C parchment cream (almanac pages)
//   D deep teal (sea shadow)   T main teal (sea)   I bright teal (sea hi, horizon)
//   < cobalt deep (night sky)   > cobalt (sky band)
//   . transparent
//
// The Teal sea (T/I) and the horizon line she watches is the future not yet
// arrived; the raised Bronze spyglass (Z->G, L lens spec) is the await - she
// is poised on the promised tide. The almanac (C) is the fn signature that
// declares the future will come.

const REF59_GRID = [
  '<><><><><><><><><><><><><><><><>', //  0 night sky band (cobalt)
  '................................', //  1
  '..........XXXXXXXX.....GG.......', //  2 head top | spyglass cap (G)
  '.........XSSSVVSSSX...GLLZ......', //  3 hair | lens spec (L)
  '........XSSVVVVVVSSX.GZZZX......', //  4 | spyglass body (Z)
  '........XSPPPPPPPPSX.NZX........', //  5 forehead | trim (N)
  '........XPBPPPPRR.PPNZX.........', //  6 eye to lens | spyglass to eye
  '........XPPPPPPRRPPXZX..........', //  7 | spyglass shaft
  '........XPPRRRRRRRPXX...........', //  8 jaw
  '.......XXPRRRRRRPXX.............', //  9
  '.......XYPPPRRRPYX..............', // 10 collar
  '......XYYYYYYYYYYYX.............', // 11 shoulders
  '.....XYYYOOOOOOOOYYXCCCC........', // 12 coat upper | almanac (C) under arm
  '.....XYOOOWWWWWOOOYYCXXCC.......', // 13 coat lit | almanac pages
  '.....XYOOOOOOOOOOOYOCVVCC.......', // 14 | almanac line (V)
  '.....XYOOOWWWWWWOOOYCXXCC.......', // 15 | almanac
  '.....XXNGGGGGGGGNXXXCVVCC.......', // 16 belt (N) buckle (G) | almanac
  '.....XYYOOOOOOOOYYKXCCCC........', // 17 coat lower | almanac foot
  '....XKYYOOOWWWWOOYYKX...........', // 18
  '....XKYYOOOOOOOOOYYKX...........', // 19
  '....XKYYOOOWWWWWOOYYKX..........', // 20
  '....XKKYOOOOOOOOOOYYKKX.........', // 21 widest
  '....XKYYFFFFFFFFFFFFYKX.........', // 22 hem
  '....XKFFFFFXXX..XXXFFFFKX.......', // 23 leg split
  '....XXZZZZXX....XXZZZZZX........', // 24 boots
  '.....XZUUZX......XZUUUZX........', // 25
  'DDDDDXXXXXXDDDDDDXXXXXXXDDDDDDDD', // 26 sea shadow at feet (D)
  'DTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT', // 27 sea (T)
  'TIITTTTIITTTTTIITTTTTIITTTTTIITT', // 28 sea sparkle (I)
  'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII', // 29 horizon line (I bright)
  'TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT', // 30 sea body
  'DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD', // 31 deep sea (D)
];

window.REF59 = function REF59({ scale = 8 }) {
  return <PixelArt grid={REF59_GRID} scale={scale} title="REF-59 The Tideforecaster" />;
};
window.REF59_GRID = REF59_GRID;
window.REF59_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','C','D','T','I','<','>'];
