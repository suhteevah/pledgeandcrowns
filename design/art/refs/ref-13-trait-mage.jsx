// REF-13 — The Trait Mage, 32×32, idle, transparent bg.
// Tall scholar in oxblood robe with gold trim; vertical staff with a single
// mage-glow crystal. Bright-teal eye glow hints scholar. Mission: double_function.

// Legend:
//   X coalblack (outline)        Y crypt (hair / robe deep shadow)
//   K inkblood (deep red shadow under hood lining)
//   O oxblood (robe primary)     W wineflesh (robe mid)
//   B basalt (hair mid, staff core)   S stone grey (staff hi)
//   R dusty rose (skin mid)      P pink quartz (skin hi)
//   I bright teal (eye glow)
//   N antique brass (staff ring, trim mid)   G old gold (trim primary)   L brass leaf (trim hi)
//   * royal arcane (hood lining, crystal shadow)   % mage glow (crystal core)
//   . transparent

const REF13_GRID = [
  '................................', //  0
  '................................', //  1
  '............XXXXXX..............', //  2 hood crown
  '...........XKKKKKKX...........X.', //  3 hood top + staff top
  '..........XK*KKKK*KX.........XBX', //  4 staff begins, hood inside arcane
  '.........XKKYYBBBYKX........XSBX', //  5 hair under hood
  '.........XKYBBBBYBKX........XBSX', //  6
  '.........XPPRRRRRWWX.......XGNGX', //  7 forehead
  '........XX.PPRRRRWWX......XGL%GX', //  8 eyes start, ring around crystal
  '........XPXIXPRXIXWX......XG%%LX', //  9 bright teal eye glow
  '........XPRRRRRRRRWX......XGL%GX', // 10
  '........X.PPRRRRRWX........XGNX.', // 11
  '........XX.XYYYYXX..........XBX.', // 12 chin under hood line
  '........XOOOOOOOOOX..........XBX', // 13 robe collar
  '.......XOOOWWWWOOOOX.........XBX', // 14 robe upper
  '.......XOWWWWWWOOOYX.........XBX', // 15
  '......XOOWWWWWWWOOYYX........XBX', // 16
  '......XOOWOOWWWWOOYYX........XBX', // 17
  '......XOOWOOOWWWOOYYX........XBX', // 18
  '......XOOOOOOOOOOOYYX........XBX', // 19
  '......XOOOOOOOOOOOOYX........XBX', // 20 sash row (gold belt below)
  '......XGGGGNNNNNGGGGX........XBX', // 21 gold belt
  '......XLOOOOOOOOOOOLX........XBX', // 22
  '......XLOOWWOOOWWOOLX........XBX', // 23 lower robe
  '......XLOOWWOOOWWOOLX........XBX', // 24
  '......XLOOOOOOOOOOOLX........XBX', // 25
  '......XLOOOOOWOOOOOLX........XBX', // 26
  '.....XLOOOOOOOOOOOOOLX.......XBX', // 27 robe widens
  '.....XLOOOOOOOOOOOOOLX.......XBX', // 28
  '.....XLLOOOOOOOOOOOLLX.......XBX', // 29 hem
  '.....XGGGGGGGGGGGGGGGX.......XBX', // 30 hem trim row
  '.....XXXXXXXXXXXXXXXXX.......XXX', // 31
];

window.REF13 = function REF13({ scale = 8 }) {
  return <PixelArt grid={REF13_GRID} scale={scale} title="REF-13 The Trait Mage" />;
};
window.REF13_GRID = REF13_GRID;
window.REF13_ROLES = ['X','Y','K','O','W','B','S','R','P','I','N','G','L','*','%'];
