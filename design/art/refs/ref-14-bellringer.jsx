// REF-14 — The Bellringer, 32×32, idle, transparent bg.
// Wiry villager pulling a bell-rope; lower curve of bell in top of frame.
// Mission: loop_break — the loop runs until you break it.

// Legend:
//   X coalblack (outline)        Y crypt (deep shadow)
//   B basalt (trouser shadow, bell deep shadow line)
//   S stone grey (rope core highlight, ring marker)
//   M mist teal (sound-ring hint, ≤4 px each side)
//   F pine (tunic shadow)        E forest (tunic mid)
//   N antique brass (rope mid, hair mid, buckle mid)
//   Z bronze (rope shadow, boots, hair shadow, belt)
//   G old gold (bell, buckle, belt buckle hi)
//   L brass leaf (bell rim hi, sound shimmer)
//   U bog umber (bell mouth deep)
//   R dusty rose (skin mid)      W wineflesh (skin shadow)   P pink quartz (skin hi)
//   . transparent

const REF14_GRID = [
  '................................', //  0
  '......XXXXXXXXXXXXXXXXXXXX......', //  1 bell outer rim
  '.....XGGGGGGGLGGGGGGGGGGGGX.....', //  2 bell body
  '.....XGUUUUUUUUUUUUUUUUUGGX.....', //  3 bell mouth deep
  '......XXXXXXNNXXXXXXXXXXXX......', //  4 bell rim base + rope hangs
  '...M..........XNX..........M....', //  5 sound shimmer + rope start
  '..MM..........XNX..........MM...', //  6
  '...M..........XZX...........M...', //  7
  '..............XNX...............', //  8 rope continues
  '..........XXXXXNXXXX............', //  9 hands gripping rope above head
  '.........XZZNNNNNZZZX...........', // 10 head/hat top + grip
  '.........XZNNNNNNNNZX...........', // 11 hair top
  '........XPPPRRRRRRWWX...........', // 12 forehead
  '........X.PPRRRRRRWX............', // 13 eyes line space
  '........XX.X.X.X.XXX............', // 14 eyes
  '........X.PPRRRRRRWX............', // 15
  '........XXXPPRRRRWXX............', // 16 chin/neck
  '.........XEEEEEEEEX.............', // 17 tunic collar
  '........XEEEEEEEEEEX............', // 18 shoulders
  '.......XEEEEEEFEEEEEX...........', // 19 chest w/ shadow
  '.......XEEEEEFFFEEEEX...........', // 20
  '.......XEEEEFFFFEEEEX...........', // 21
  '.......XZZZZGNGZZZZZX...........', // 22 belt + buckle
  '.......XEEEEEFFFEEEEX...........', // 23 lower tunic
  '.......XEEEEFFFFEEEEX...........', // 24
  '.......XFFFFXXXXFFFFX...........', // 25 tunic hem split
  '........XBBBX..XBBBX............', // 26 trousers
  '........XBBBX..XBBBX............', // 27
  '........XBBBX..XBBBX............', // 28
  '........XZZZX..XZZZX............', // 29 boot tops
  '........XZUUX..XZUUX............', // 30 boot mid
  '........XXXXX..XXXXX............', // 31 soles
];

window.REF14 = function REF14({ scale = 8 }) {
  return <PixelArt grid={REF14_GRID} scale={scale} title="REF-14 The Bellringer" />;
};
window.REF14_GRID = REF14_GRID;
window.REF14_ROLES = ['X','Y','B','S','M','F','E','N','Z','G','L','U','R','W','P'];
