// REF-02 — Ferris guide, 32×32, transparent bg.
// Burgundy crab (Wineflesh + Old gold + Crypt + Dusty rose belly).
// Distinct from official Ferris: heraldic burgundy/gold, not orange.

// Legend:
//   X coalblack (outline)         Y crypt (deep shell shadow)
//   W wineflesh (shell mid)       R dusty rose (belly / soft hi)
//   P pink quartz (rim hi)        K inkblood (under-shadow)
//   N antique brass (claw shadow) G old gold (claw hi / shell ridges)
//   L brass leaf (specular tip)   B basalt (eye outline) C parchment cream (eye white)
//   . transparent

const REF02_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '...XX.....................XX....', //  3 eye stalks
  '..XCBX...................XBCX...', //  4
  '..XCBX...................XBCX...', //  5
  '...XX.....................XX....', //  6
  '....XX...................XX.....', //  7 stalk down
  '....X.....................X.....', //  8
  '....X..XXXXXXXXXXXXXXXX...X.....', //  9 shell top
  '....XXXWWWGWWWWWGWWWWWWXXXX.....', // 10
  '....XWWPWWWGWWWGWWWGWWWWWX......', // 11 highlights + ridges
  '...XWPWWWWWWWWWWWWWWWWWWWWX.....', // 12
  '..XWWGWWWGWWWWGWWWWWGWWWWWWX....', // 13
  '..XWWWWWWWWWWWWWWWWWWWWWWWWX....', // 14
  '.XWWWWGWWWWWGWWWGWWWWWGWWWWWX...', // 15
  '.XWWWWWWWWWWWWWWWWWWWWWWWWWWX...', // 16
  'XNGXWWWWWWWWWWWWWWWWWWWWWWWWXGNX', // 17 claws emerge
  'XNGGXWWRRRRRRRRRRRRRRRRRRWWXGGNX', // 18 belly band starts
  'XNGGXRRRRRRRRRRRRRRRRRRRRRRXGGNX', // 19
  'XNGNXRRRRPPRRRRRRRRRRPPRRRRXNGNX', // 20
  '.XNNXRRRRRRRRRRRRRRRRRRRRRRXNNX.', // 21
  '..XXXKKWWWKKKKKWWWWKKKKKWWKKXXX.', // 22 underside outline
  '....XXX.X.X..X.X.X..X.X..X.XX...', // 23 little legs
  '.....X.X.X..X.X.X..X.X.X..X.....', // 24
  '......X.X..X.X.X..X.X.X..X......', // 25 leg tips
  '................................', // 26
  '................................', // 27
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF02 = function REF02({ scale = 8 }) {
  return <PixelArt grid={REF02_GRID} scale={scale} title="REF-02 Ferris guide" />;
};
window.REF02_GRID = REF02_GRID;
window.REF02_ROLES = ['X','Y','W','R','P','K','N','G','L','B','C'];
