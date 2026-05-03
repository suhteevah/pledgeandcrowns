// REF-17 — The Herald, 32×32, idle, transparent bg.
// Court announcer in three-banded tabard (each band = a struct field) and
// rolled scroll at his side. Mission: struct_basic.

// Legend:
//   X coalblack (outline, band rules)   Y crypt (oxblood band shadow)
//   O oxblood (top band primary)        W wineflesh (top band lit row)
//   N antique brass (gold band shadow, scroll seal, hair shadow)
//   G old gold (gold band primary, buckle)
//   L brass leaf (gold band lit row, hem stitch)
//   F pine (forest band shadow)         E forest (forest band primary)
//   H spring meadow (forest band lit row)
//   J mossbed (undertunic shadow under tabard)
//   Z bronze (hair, belt, scroll seal shadow)
//   B basalt (mouth shadow)             S stone grey (boots)
//   V aged paper (scroll core shadow)   C parchment cream (scroll core)
//   R dusty rose (skin mid)             P pink quartz (skin hi)
//   . transparent

const REF17_GRID = [
  '................................', //  0
  '..........XXXXXXXX..............', //  1 head top
  '.........XZZZZZZZZX.............', //  2 hair shadow row
  '........XZNNNNNNNNZX............', //  3 hair
  '........XPPPPPPPPRRX............', //  4 forehead
  '........X.PPRRRRRRWX............', //  5
  '........XX.X.XX.X.XX............', //  6 eyes
  '........X.PPRRRRRRWX............', //  7
  '........XXPPRRRRRRWX............', //  8
  '........XXXPRRRRRWXX............', //  9 chin/jaw
  '.........XJJJJJJJJX.............', // 10 collar (undertunic)
  '........XJJJJJJJJJJX............', // 11
  '.......XWOOOOOOOOOOOWX..XCCX....', // 12 top band lit row + scroll start
  '.......XOOOOOOOOOOOOOX..XVCX....', // 13 top band primary
  '.......XYYOOOOOOOOOYYX..XVCX....', // 14
  '.......XXXXXXXXXXXXXXX..XVNX....', // 15 black rule between bands; scroll seal
  '.......XLGGGGGGGGGGGLX..XZZX....', // 16 gold band lit row
  '.......XGGGGGGGGGGGGGX..XZNX....', // 17 gold band primary
  '.......XNNGGGGGGGGGNNX...XX.....', // 18
  '.......XXXXXXXXXXXXXXX..........', // 19 black rule
  '.......XHEEEEEEEEEEEHX..........', // 20 forest band lit row
  '.......XEEEEEEEEEEEEEX..........', // 21 forest band primary
  '.......XFFEEEEEEEEEFFX..........', // 22
  '.......XLLLLLLLLLLLLLX..........', // 23 brass-leaf hem stitch
  '.......XZZZZZGGGZZZZZX..........', // 24 belt + buckle
  '.......XJJJJJJJJJJJJJX..........', // 25 lower undertunic
  '.......XJJJJJJJJJJJJJX..........', // 26
  '.......XJJJJXXXXJJJJJX..........', // 27 hem split at legs
  '........XSSSX..XSSSX............', // 28 boot tops
  '........XSBSX..XSBSX............', // 29 boots mid
  '........XSSSX..XSSSX............', // 30
  '........XXXXX..XXXXX............', // 31 soles
];

window.REF17 = function REF17({ scale = 8 }) {
  return <PixelArt grid={REF17_GRID} scale={scale} title="REF-17 The Herald" />;
};
window.REF17_GRID = REF17_GRID;
window.REF17_ROLES = ['X','Y','O','W','N','G','L','F','E','H','J','Z','B','S','V','C','R','P'];
