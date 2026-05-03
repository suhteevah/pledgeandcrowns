// REF-18 — The Twin, 32×32, idle, transparent bg.
// One body, two halves. Tunic bisected vertically: viewer-left = forest
// ramp, viewer-right = burgundy ramp. Mission: tuple_destructure.

// Legend:
//   X coalblack (outline, centre seam)
//   K inkblood (burgundy half deep shadow)
//   Y crypt (burgundy half hair, mid shadow)
//   O oxblood (burgundy tunic primary)
//   W wineflesh (burgundy tunic lit row)
//   J mossbed (forest half deep shadow)
//   F pine (forest half hair shadow)
//   E forest (forest tunic primary)
//   H spring meadow (forest tunic lit row)
//   Z bronze (cool-warm hair on viewer-left)
//   N antique brass (belt clasp, ring shadow)
//   G old gold (circlet across brow, belt clasp hi)
//   Q hayfield (right-hand ring = `b`)
//   P pink quartz (skin hi, left-hand ring = `a`)
//   R dusty rose (skin mid)
//   S stone grey (belt, boot soles spec)
//   B basalt (boots)
//   . transparent

const REF18_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XFFFFXYYYX.............', //  3 hair: left=pine, right=crypt
  '........XFZZZZXYYYYX............', //  4 hair mid
  '........XFFZZZXYYYYX............', //  5
  '........XGGGGGXGGGGX............', //  6 gold circlet across brow
  '........XPPPPPXRRRRX............', //  7 forehead
  '........X.PPRRXRRRWX............', //  8
  '........XX.X.XXX.X.X............', //  9 eye line (centre seam)
  '........X.PPRRXRRRWX............', // 10
  '........XXPPPRXRRRWX............', // 11
  '........XXXPPRXRRWXX............', // 12 chin
  '.......XHEEEEEXOOOOWX...........', // 13 collar lit row
  '......XHEEEEEEXOOOOOWX..........', // 14 shoulders
  '......XEEEHEEEXOOOWOOX..........', // 15 tunic upper
  '......XEEEEHEEXOOWWOOX..........', // 16
  '......XEEEEEEEXOOOOOOX..........', // 17
  '......XJEEEEEEXOOOOOKX..........', // 18 mid-shadow rows
  '......XJJEEEEEXOOOOKKX..........', // 19
  '......XSSSSSSGNGSSSSSSX.........', // 20 belt + central buckle (clasp = comma)
  '......XJEEEEEEXOOOOOKX..........', // 21 lower tunic
  '......XJJEEEEEXOOOOKKX..........', // 22
  '......XJEEEEEEXOOOOOKX..........', // 23
  '......XJJEEEEEXOOOOKKX..........', // 24
  '......XJJEEEEEXOOOOKKX..........', // 25
  '.....XPXJJJJJEXOOKKKXQX.........', // 26 left hand (P ring = `a`); right hand (Q ring = `b`)
  '.....XPXJJJJJEXOOKKKXQX.........', // 27
  '......XJJJJXXXXKKKKKKX..........', // 28 hem split
  '......XBBBSX..XSBBBSX...........', // 29 boots
  '......XBBBSX..XSBBBSX...........', // 30
  '......XXXXXX..XXXXXXX...........', // 31 soles
];

window.REF18 = function REF18({ scale = 8 }) {
  return <PixelArt grid={REF18_GRID} scale={scale} title="REF-18 The Twin" />;
};
window.REF18_GRID = REF18_GRID;
window.REF18_ROLES = ['X','K','Y','O','W','J','F','E','H','Z','N','G','Q','P','R','S','B'];
