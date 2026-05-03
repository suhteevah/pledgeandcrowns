// REF-16 — The Oracle, 32×32, idle, transparent bg.
// Veiled village seer cupping a duality-orb. Mission: match_option —
// the orb's brass-leaf upper half is `Some`, its inkblood lower half
// is `None`; both arms of the match made literal.

// Legend:
//   X coalblack (outline)       Y crypt (robe primary, hair, veil shadow)
//   K inkblood (robe deep shadow, orb None-half)
//   O oxblood (robe mid)        W wineflesh (robe lit edge, mouth)
//   R dusty rose (skin mid)     P pink quartz (skin hi)
//   B basalt (eyes through veil hint)   S stone grey (trim)
//   N antique brass (orb ring shadow)   G old gold (orb ring)
//   L brass leaf (orb Some-half)
//   . transparent

const REF16_GRID = [
  '................................', //  0
  '............XXXXXX..............', //  1 head top
  '...........XYYYYYYX.............', //  2 hair crown
  '..........XYYYYYYYYX............', //  3
  '..........XPPPPPPPPX............', //  4 forehead
  '..........XYYYYYYYYX............', //  5 veil band (crypt across eyes)
  '..........XPPRRRRRPX............', //  6 cheek line below veil
  '..........XPPRRRRWPX............', //  7
  '..........XXPPRRWWXX............', //  8 chin
  '...........XYYYYYYX.............', //  9 hair under chin sides
  '..........XYYOOOOYYX............', // 10 robe collar
  '.........XYOOOOOOOOYX...........', // 11 shoulders
  '........XYOOOWWWOOOOYX..........', // 12
  '........XYOOOWWWOOOOYX..........', // 13
  '........XYOOWWWWWOOOYX..........', // 14
  '........XYOOWXGGGXWOYX..........', // 15 orb ring top in cupped hands
  '........XYOOXGLLLGXOYX..........', // 16 orb upper half = brass leaf
  '........XYOWXGLLLGXOYX..........', // 17
  '........XYOOXNKKKNXOYX..........', // 18 orb lower half = inkblood
  '........XYOOXGKKKGXOYX..........', // 19
  '........XYOOOXGGGXOOYX..........', // 20 ring bottom
  '........XYOOOOOOOOOOYX..........', // 21
  '........XYKOOOOOOOOOYX..........', // 22 right-side shadow row
  '........XYKKOOOOOOOKYX..........', // 23
  '........XYKKOOOOOOKKYX..........', // 24
  '........XYKKKOOOKKKKYX..........', // 25 robe widens
  '.......XYKKKKKKKKKKKKYX.........', // 26
  '.......XYKKKKKKKKKKKKYX.........', // 27
  '.......XYKKKKKKKKKKKKYX.........', // 28
  '.......XSSSSSSSSSSSSSSX.........', // 29 stone grey hem trim
  '.......XSSSSSSSSSSSSSSX.........', // 30
  '.......XXXXXXXXXXXXXXXX.........', // 31
];

window.REF16 = function REF16({ scale = 8 }) {
  return <PixelArt grid={REF16_GRID} scale={scale} title="REF-16 The Oracle" />;
};
window.REF16_GRID = REF16_GRID;
window.REF16_ROLES = ['X','Y','K','O','W','R','P','B','S','N','G','L'];
