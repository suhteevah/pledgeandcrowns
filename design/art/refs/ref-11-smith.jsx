// REF-11 — The Smith, 32×32, idle, transparent bg.
// Stocky village blacksmith. Burgundy apron (heraldic uniform), bronze beard,
// iron hammer at right side. Mission: mut_binding — name what changes.

// Legend:
//   X coalblack (outline)        K inkblood (deepest red shadow)
//   Y crypt (apron deep shadow)  O oxblood (apron primary)
//   W wineflesh (apron mid)      R dusty rose (skin mid)   P pink quartz (skin hi)
//   F pine (undershirt shadow)   Z bronze (haft, beard shadow, boots)
//   N antique brass (beard mid, belt buckle)  G old gold (buckle hi)
//   L brass leaf (hammer-head specular strike face)
//   U bog umber (boot shadow)    B basalt (hammer head core, brow)
//   S stone grey (hammer head hi)
//   . transparent

const REF11_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..........XXXXXXXX..............', //  3 head top
  '.........XZZZZZZZX..............', //  4 hair shadow
  '........XZZNNNNZZZX.............', //  5 hair mid
  '........XZNNNNNNNNX.............', //  6
  '........XPPPPPPRRWX.............', //  7 forehead skin
  '........XXPPRRRRRWX.............', //  8 brow line
  '........X.PRRRRRRRX.............', //  9
  '........XX.X.X.X.XX.............', // 10 eyes line
  '........XPPPPRRRRWX.............', // 11 mid face
  '.......XZNNNNNNNNNNX............', // 12 beard top
  '.......XZNNNZZZNNNZX............', // 13 beard mid
  '........XXNNZZZZNNX.............', // 14 beard tip
  '........XZZZZZZZZZX.............', // 15 collar
  '.......XYOOOOOOOOOYX...XSSX.....', // 16 apron top + hammer head top
  '......XYOOWWWOOOOOOYX..XBBSX....', // 17 apron upper, hammer body
  '......XYOWWWWOOOOOOYX..XBBLSX...', // 18 hammer strike-face highlight
  '......XYOWWWWOOOOOYYX..XBBSSX...', // 19
  '......XYOOWWOOOOOYYYX...XSSX....', // 20 apron mid
  '......XYOOOOOOOOOYYYX...XZSX....', // 21 hammer haft starts
  '......XYOOOOOOOOYYYYX...XZZX....', // 22
  '......XXXNGGGGGGNXXXX...XZZX....', // 23 belt with buckle
  '......XYYOOOOOOOOOYYX...XZZX....', // 24 apron lower
  '......XYYOOWWOOWOOYYX...XZZX....', // 25
  '......XYYOOWWOOWOOYYX...XZZX....', // 26
  '......XYFFFFFFFFFFFYX...XZZX....', // 27 trouser tops
  '......XFFFFXXXXFFFFFX...........', // 28 leg split
  '......XZZZZX..XZZZZZX...........', // 29 boot tops
  '......XZUUZX..XZUUZZX...........', // 30 boot mid
  '......XXXXXX..XXXXXXX...........', // 31 boot soles
];

window.REF11 = function REF11({ scale = 8 }) {
  return <PixelArt grid={REF11_GRID} scale={scale} title="REF-11 The Smith" />;
};
window.REF11_GRID = REF11_GRID;
window.REF11_ROLES = ['X','K','Y','O','W','R','P','F','Z','N','G','L','U','B','S'];
