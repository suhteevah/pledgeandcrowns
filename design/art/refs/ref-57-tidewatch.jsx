// SPDX-License-Identifier: MIT
// REF-57 - The Tidewatch, 32x32, idle, transparent bg.
// Stands beside a tall tide-gauge post (frame-right) cut with notched
// marks up its length. As the tide rises she reads the gauge and the count
// climbs - a single Old-gold tick mark advancing notch by notch up the
// post. Many waves push the water; every one of them nudges the SAME mark
// up by one, and no two pushes ever land on the same notch or lose a count.
// That is an `AtomicUsize`: a counter many threads can `fetch_add` at once,
// each increment whole and uninterrupted, the tally always exact. Crypt
// coat, Teal water line at the post foot, Old-gold tick, no magic.
// Mission: atomic - increment a shared counter with AtomicUsize.
//
// Legend:
//   X coalblack (outline, post body, notch cuts)
//   K inkblood (coat deep shadow)
//   Y crypt (coat deep)   O oxblood (coat primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi, notch ticks)
//   Z bronze (post wood, boots)   N antique brass (belt, post cap)
//   G old gold (climbing tick mark)   L brass leaf (tick spec)
//   U bog umber (boot shadow)   F pine (hem)
//   D deep teal (water shadow)   T main teal (tide water)   I bright teal (water hi)
//   . transparent
//
// The Old-gold tick (G with L spec) standing partway up the notched post
// is the atomic counter's current value; the Teal water (T/I) at the foot
// is the rising tide whose every push fetch-adds the mark one notch higher.

const REF57_GRID = [
  '................................', //  0
  '..........XXXXXXXX....XNNX......', //  1 head top | post cap (N)
  '.........XSSSVVSSSX...XZZX......', //  2 hair | post body (Z)
  '........XSSVVVVVVSSX..XZVX......', //  3 | notch tick (V)
  '........XSPPPPPPPPSX..XZZX......', //  4 forehead | post
  '........XPPPPPPPPRRX..XZVX......', //  5 | notch tick
  '........XPBPPPPBPRWX..XZZX......', //  6 brow/eyes | post
  '........XPPRRRRRRRPX..XGLX......', //  7 | climbing tick (G/L spec)
  '.......XXPRRRRRRPXX...XGZX......', //  8 jaw | tick base
  '.......XYPPPRRRPYX..PPXZVX......', //  9 collar | hand reads (P) | notch
  '......XYYYYYYYYYYYX.PRPXZZX.....', // 10 shoulders | hand | post
  '.....XYYYOOOOOOOOYYXXXXXZVX.....', // 11 coat upper | arm to post
  '.....XYOOOWWWWWOOOYYX..XZZX.....', // 12 coat lit | post
  '.....XYOOOOOOOOOOOYOX..XZVX.....', // 13 | post
  '.....XYOOOWWWWWWOOOYX..XZZX.....', // 14 | post
  '.....XXNGGGGGGGGNXXX...XZVX.....', // 15 belt (N) buckle (G) | post
  '.....XYYOOOOOOOOYYKX...XZZX.....', // 16 coat lower | post
  '.....XYYOOOWWWWOOYYKX..XZVX.....', // 17 | post
  '....XKYYOOOOOOOOOYYKX..XZZX.....', // 18 | post
  '....XKYYOOOWWWWWOOYYKX.XZVX.....', // 19 | post
  '....XKKYOOOOOOOOOOYYKKXXZZX.....', // 20 widest | post
  '....XKYYFFFFFFFFFFFFYKXDTTD.....', // 21 hem | water shadow at foot (D/T)
  '....XKFFFFFXXX..XXXFFFDTITD.....', // 22 leg split | tide water (T/I)
  '....XXZZZZXX....XXZZZDTTTTD.....', // 23 boots | water
  '.....XZUUZX......XZUUDTIITD.....', // 24 | water hi
  '.....XXXXXX......XXXXDTTTTD.....', // 25 soles | water
  '...................DDTTTDD......', // 26 water settle
  '................................', // 27
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF57 = function REF57({ scale = 8 }) {
  return <PixelArt grid={REF57_GRID} scale={scale} title="REF-57 The Tidewatch" />;
};
window.REF57_GRID = REF57_GRID;
window.REF57_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','D','T','I'];
