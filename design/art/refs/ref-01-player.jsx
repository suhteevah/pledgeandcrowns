// REF-01 — Player character (default), 32×32, idle, transparent bg.
// Cobalt deep tunic, brown leather belt, satchel, wooden staff with old-gold
// crystal. Pale skin (Pink quartz / Dusty rose / Wineflesh). Outline Coalblack.
// Lighting: top-left 45°.

// Legend:
//   X coalblack (outline)        K inkblood (deep shadow on red)
//   P pink quartz (skin hi)      R dusty rose (skin mid)   W wineflesh (skin shadow)
//   < cobalt deep (tunic shadow) > cobalt (tunic mid)      I bright teal (tunic hi accent)
//   Z bronze (belt/staff shadow) N antique brass (belt/boot mid)
//   G old gold (crystal mid)     L brass leaf (crystal hi)
//   U bog umber (boot shadow)    Y crypt (hair shadow)
//   B basalt (hair mid)          S stone grey (hair hi/staff hi)
//   .  transparent

const REF01_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..........XXXXXX................', //  3 hair top
  '.........XYYYYBBX...............', //  4
  '........XYYBBBBBBX..............', //  5
  '........XYBSBBBBBX..............', //  6 hair w/ highlight
  '........XBSBBBBBBX.S............', //  7
  '........XXRPPPPRWX.S............', //  8 face top (skin)
  '........X.PPRRRRWX.S............', //  9
  '........XX..XXR.XX.S............', // 10 eyes line
  '.........XPPPPRWX..S............', // 11 face mid
  '.........XPRRRWWX..L............', // 12 chin
  '.........XXWXXWXX..LG...........', // 13 neck
  '........X<<<I<<<X..LGL..........', // 14 collar
  '.......X<<>>>><<<X..L...........', // 15 shoulders
  '.......X<>>I>>><<X..S...........', // 16
  '......X<<>>>>>><<<X.S...........', // 17 chest
  '......X<>>I>>>>><<X.S...........', // 18
  '......X<<>>>>>>><<X.S...........', // 19
  '......X<<>>>>>>><<X.S...........', // 20 waist top
  '.......XXXNNZNNXXX..S...........', // 21 belt
  '......X<>NGNGNGNN<X.S...........', // 22 belt buckle row + satchel hint
  '.....X<<>>>NZN>>><X.S...........', // 23
  '.....X<<>>>NZN>>><X.S...........', // 24 hips
  '.....X<<>><<X<>><<X.S...........', // 25 leg split
  '.....XXX<<XXXXX<<XX.S...........', // 26
  '......XKK<X...XK<KX.B...........', // 27 boots top
  '......XNNZX...XNNZX.X...........', // 28 boots mid
  '......XZZUX...XZZUX.............', // 29 boots toe
  '......XXXXX...XXXXX.............', // 30 boot sole
  '................................', // 31
];

window.REF01 = function REF01({ scale = 8 }) {
  return <PixelArt grid={REF01_GRID} scale={scale} title="REF-01 Player character" />;
};
window.REF01_GRID = REF01_GRID;
window.REF01_ROLES = ['X','Y','B','S','P','R','W','<','>','I','N','Z','G','L','U','K'];
