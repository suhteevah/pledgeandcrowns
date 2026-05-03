// REF-23 — The Pilgrim, 32×32, idle, transparent bg.
// Hooded road-walker in a Pine cloak. Lantern in right hand (the default),
// staff in left. Lantern flame is the only bright on the sprite.
// Mission: option_unwrap_or — the absent path lit a default.

// Legend:
//   X coalblack (outline, lantern cage)
//   J mossbed (cloak deep shadow)
//   F pine (cloak primary)        E forest (cloak mid)
//   H spring meadow (cloak lit edge)
//   K inkblood (hood interior shadow)
//   R dusty rose (jaw skin mid)   P pink quartz (jaw skin hi)
//   W wineflesh (jaw skin shadow)
//   B basalt (lantern body)       S stone grey (eye glint, staff ferrule, dust)
//   V aged paper (chin scarf)
//   Z bronze (chain, staff, hair-shadow if any)
//   N antique brass (hood-glow tint, scarf shadow)
//   G old gold (lantern glass)
//   L brass leaf (flame core specular — the default value)
//   U bog umber (boots)
//   . transparent

const REF23_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 hood top
  '.........XFFFFFFFFX.............', //  3 hood
  '........XFFEEEEEEFFX............', //  4
  '........XFEEKKKKKEFX............', //  5 hood interior shadow K
  '........XFKKKKKKKKFX............', //  6
  '........XFKKKKSKKKFX............', //  7 single S eye glint
  '........XFKKKKKKKKFX............', //  8
  '........XFEEKKKKEENX............', //  9 hood lower edge w/ N glow tint right
  '........XVVVPPPPRRWX............', // 10 chin scarf + lower jaw
  '........XXVVPPRRWWXX............', // 11
  '........XFFFFFFFFFFX............', // 12 cloak collar
  '......XXFFFEEEEEFFFXX...........', // 13 cloak shoulders
  '.....XFHFFEEEEEEFFFFXX..........', // 14 cloak with H lit-edge highlight left
  '.....XFFFFEEEEEEEFFFFX..XZX.....', // 15 staff begins right
  '.....XFFFFEEEEEEEFFFFX..XZX.....', // 16
  '.....XFFFFEEEEEEEFFFFX..XZX.....', // 17
  '.....XFFFFEEEEEEEFFFFX..XSX.....', // 18 staff ferrule S
  '.....XFFFFEEEEEEEFFFFX..XZX.....', // 19
  '.....XFFFFEEEEEEEFFFFX..........', // 20 lantern hangs below opposite hand
  '.....XFFFFFEEEEEFFFFFX..XXXX....', // 21 chain begins
  '.....XJJJFFFFFFFFFJJJX..XZZX....', // 22 chain Z
  '.....XJJJJFFFFFFFFJJJX.XXBBXX...', // 23 lantern cage top
  '.....XJJJJJFFFFFJJJJJX.XBGGBX...', // 24 lantern body + glass
  '.....XJJJJJFFFFFJJJJJX.XBLGBX...', // 25 flame core L (specular)
  '.....XJJJJJJJJJJJJJJJX.XBGGBX...', // 26
  '.....XJJJJJJJJJJJJJJJX.XXBBXX...', // 27 lantern cage bottom
  '.....XJJJSSSSSSSSSJJJX..........', // 28 dust hem
  '......XZZZX....XZZZX............', // 29 boot tops
  '......XZUUX....XZUUX............', // 30 boot mid
  '......XXXXX....XXXXX............', // 31 soles
];

window.REF23 = function REF23({ scale = 8 }) {
  return <PixelArt grid={REF23_GRID} scale={scale} title="REF-23 The Pilgrim" />;
};
window.REF23_GRID = REF23_GRID;
window.REF23_ROLES = ['X','J','F','E','H','K','R','P','W','B','S','V','Z','N','G','L','U'];
