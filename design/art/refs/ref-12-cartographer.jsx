// REF-12 — The Cartographer, 32×32, idle, transparent bg.
// Slim traveller in cobalt cloak; rolled map at left hip with a forked road
// glyph showing through. Brass compass in right hand. Mission: if_else_sign.

// Legend:
//   X coalblack (outline)        Y crypt (hair shadow)
//   B basalt (hair mid)          S stone grey (hair hi, compass needle hi)
//   < cobalt deep (cloak shadow) > cobalt (cloak mid)
//   R dusty rose (skin mid)      W wineflesh (skin shadow)   P pink quartz (skin hi)
//   Z bronze (strap, belt, map ends)
//   N antique brass (strap hi, map roll)
//   G old gold (compass ring, buckle, road crossroads marker)
//   L brass leaf (compass ring hi)
//   V aged paper (map paper roll)   C parchment cream (unrolled map flap)
//   E forest (road glyph on the map)
//   U bog umber (boot)
//   . transparent

const REF12_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..........XXXXXXXX..............', //  3 hair top
  '.........XYYYBBBBYX.............', //  4
  '........XYYBBSBBBBYX............', //  5 hair w/ highlight
  '........XYBBSBBBBBYX............', //  6
  '........XPRPPPPRRWX.............', //  7 forehead
  '........X.PPRRRRRWX.............', //  8 face top
  '........XX.XX.XX.XX.............', //  9 eyes line
  '........X.PPRRRRRWX.............', // 10
  '.........XPPRRRRWX..............', // 11 chin
  '.........XYBBYYBBYX.............', // 12 hair around neck
  '........X<<<<<<<<<<X............', // 13 cloak collar
  '.......X<<>>><<<<<<<X...........', // 14 cloak shoulders w/ left hi
  '......X<<>>>><<<<<<<<X..........', // 15
  '......X<>>>><<<<<<<<<X.....XX...', // 16 compass starts
  '......X<<>>><<<<<<<<<X....XGGX..', // 17 compass ring outer
  '......X<<>><<<<<<<<<<X...XGLLGX.', // 18 compass ring w/ hi
  '......X<<>><<<<<<<<<<X...XLPSGX.', // 19 north tick (P) + needle hi (S)
  '......X<<>><<<XZZNNNVX...XGLLGX.', // 20 strap to map roll
  '......X<<>><<<XNNNVVVX....XGGX..', // 21 map roll mid
  '......X<<>><<<XCEXCEX......XX...', // 22 unrolled flap, road forks
  '......X<<>>><<XCEEECX...........', // 23 road E + crossroads G hint
  '......X<<<>><<XCEGECX...........', // 24
  '......X<<<<<<<XCCEEX............', // 25 flap edge
  '......X<XGGGGGGGGGGGX...........', // 26 belt buckle row
  '......X<<>>><<<<<<<<<X..........', // 27 lower cloak
  '......X<<>>><<<<<<<<<X..........', // 28
  '......XXXX<<<<<<<<XXXX..........', // 29 hem
  '......XZZX..XX..XXZZX...........', // 30 boots peek
  '......XUUX..XX..XXUUX...........', // 31 boot soles
];

window.REF12 = function REF12({ scale = 8 }) {
  return <PixelArt grid={REF12_GRID} scale={scale} title="REF-12 The Cartographer" />;
};
window.REF12_GRID = REF12_GRID;
window.REF12_ROLES = ['X','Y','B','S','<','>','R','W','P','Z','N','G','L','V','C','E','U'];
