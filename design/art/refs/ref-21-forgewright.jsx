// REF-21 — The Forgewright, 32×32, idle, transparent bg.
// Wiry burgundy-aproned smith specialist gripping iron tongs in front,
// clamping a small glowing workpiece. Mission: borrow_mut. The single
// gold workpiece pixel is the value being mutated through the deref.

// Legend:
//   X coalblack (outline, tongs cage, workpiece border)
//   K inkblood (apron deep shadow)
//   Y crypt (apron primary)        O oxblood (apron mid)
//   W wineflesh (apron lit edge)   R dusty rose (skin mid)
//   P pink quartz (skin hi)
//   B basalt (tongs core, brow line, eyes)
//   S stone grey (tongs hi)
//   Z bronze (hair, beard shadow, haft, boots)
//   N antique brass (hair mid, beard, belt)
//   G old gold (workpiece core, buckle)
//   L brass leaf (workpiece specular — the mutation point)
//   U bog umber (boot deep)
//   . transparent

const REF21_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..........XXXXXXXX..............', //  3 head top
  '.........XZZZNNNZZX.............', //  4 hair shadow
  '........XZNNNNNNNNZX............', //  5 hair mid
  '........XZNNNNNNNNNX............', //  6
  '........XPPPPPPPRWWX............', //  7 forehead
  '........XBPPPRRRRRWX............', //  8 brow line
  '........XX.X.XX.X.XX............', //  9 eyes
  '........X.PPRRRRRRWX............', // 10
  '........XZNNNNNNNNZX............', // 11 short beard top
  '........XXZNNNNNZZXX............', // 12 beard tip
  '.........XYYOOOOYYX.............', // 13 collar
  '........XYYYOOOOOOYX............', // 14 apron upper
  '........XYYOOWWWOOOX............', // 15 apron lit row
  '........XYYOOWWWOOOX............', // 16
  '........XSXXXXXXXXSX............', // 17 tongs cage upper
  '........XSXBBGGBBXSX............', // 18 tongs jaws + workpiece border
  '........XSXBLGGGBXSX............', // 19 workpiece core w/ specular L
  '........XSXBBGGBBXSX............', // 20 workpiece lower
  '........XSXXXXXXXXSX............', // 21 tongs cage lower
  '........XYYOOOOOOOYX............', // 22 apron mid
  '........XYYOOWWOOOYX............', // 23
  '........XXNGGGGGGNXX............', // 24 belt + buckle
  '........XYYOOOOOOOYX............', // 25 apron lower
  '........XYYOOWWOOOYX............', // 26
  '........XYYOOOOOOOYX............', // 27 apron hem
  '........XZZZXXXXZZZX............', // 28 leg split / trouser tops
  '........XZZZX..XZZZX............', // 29 boot tops
  '........XZUUX..XZUUX............', // 30 boot mid
  '........XXXXX..XXXXX............', // 31 soles
];

window.REF21 = function REF21({ scale = 8 }) {
  return <PixelArt grid={REF21_GRID} scale={scale} title="REF-21 The Forgewright" />;
};
window.REF21_GRID = REF21_GRID;
window.REF21_ROLES = ['X','K','Y','O','W','R','P','B','S','Z','N','G','L','U'];
