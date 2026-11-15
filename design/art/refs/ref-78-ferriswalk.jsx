// SPDX-License-Identifier: MIT
// REF-78 - Ferris walk frames, 32x32 each, 2-column sheet (64x32).
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/ferris_walk.png (atlas, 2 cols x 1 row).
//
// Built on the approved REF-02 Ferris. Rows 0 to 23 - eye stalks, shell, claws
// and the leg roots - are REF-02 unchanged. Only the two rows of leg tips
// move: in walk_0 the upper tips step one pixel right and the lower one pixel
// left, and walk_2 is the reverse, so the legs ripple as a crab's do.
//
// A crab scuttles the same way whichever way it goes, so one cycle serves all
// four directions: [walk_0, REF-02 idle, walk_2, REF-02 idle] at 150 ms per
// frame.
//
// Legend: X coalblack  K inkblood  W wineflesh  R dusty rose  P pink quartz
//         G old gold  N antique brass  C parchment cream  B basalt

const REF78_COLS = 2;

const REF78_TILES = {
  walk_0: [
    '................................',
    '................................',
    '................................',
    '...XX.....................XX....',
    '..XCBX...................XBCX...',
    '..XCBX...................XBCX...',
    '...XX.....................XX....',
    '....XX...................XX.....',
    '....X.....................X.....',
    '....X..XXXXXXXXXXXXXXXX...X.....',
    '....XXXWWWGWWWWWGWWWWWWXXXX.....',
    '....XWWPWWWGWWWGWWWGWWWWWX......',
    '...XWPWWWWWWWWWWWWWWWWWWWWX.....',
    '..XWWGWWWGWWWWGWWWWWGWWWWWWX....',
    '..XWWWWWWWWWWWWWWWWWWWWWWWWX....',
    '.XWWWWGWWWWWGWWWGWWWWWGWWWWWX...',
    '.XWWWWWWWWWWWWWWWWWWWWWWWWWWX...',
    'XNGXWWWWWWWWWWWWWWWWWWWWWWWWXGNX',
    'XNGGXWWRRRRRRRRRRRRRRRRRRWWXGGNX',
    'XNGGXRRRRRRRRRRRRRRRRRRRRRRXGGNX',
    'XNGNXRRRRPPRRRRRRRRRRPPRRRRXNGNX',
    '.XNNXRRRRRRRRRRRRRRRRRRRRRRXNNX.',
    '..XXXKKWWWKKKKKWWWWKKKKKWWKKXXX.',
    '....XXX.X.X..X.X.X..X.X..X.XX...',
    '......X.X.X..X.X.X..X.X.X..X....',
    '.....X.X..X.X.X..X.X.X..X.......',
    '................................',
    '................................',
    '................................',
    '................................',
    '................................',
    '................................',
  ],
  walk_2: [
    '................................',
    '................................',
    '................................',
    '...XX.....................XX....',
    '..XCBX...................XBCX...',
    '..XCBX...................XBCX...',
    '...XX.....................XX....',
    '....XX...................XX.....',
    '....X.....................X.....',
    '....X..XXXXXXXXXXXXXXXX...X.....',
    '....XXXWWWGWWWWWGWWWWWWXXXX.....',
    '....XWWPWWWGWWWGWWWGWWWWWX......',
    '...XWPWWWWWWWWWWWWWWWWWWWWX.....',
    '..XWWGWWWGWWWWGWWWWWGWWWWWWX....',
    '..XWWWWWWWWWWWWWWWWWWWWWWWWX....',
    '.XWWWWGWWWWWGWWWGWWWWWGWWWWWX...',
    '.XWWWWWWWWWWWWWWWWWWWWWWWWWWX...',
    'XNGXWWWWWWWWWWWWWWWWWWWWWWWWXGNX',
    'XNGGXWWRRRRRRRRRRRRRRRRRRWWXGGNX',
    'XNGGXRRRRRRRRRRRRRRRRRRRRRRXGGNX',
    'XNGNXRRRRPPRRRRRRRRRRPPRRRRXNGNX',
    '.XNNXRRRRRRRRRRRRRRRRRRRRRRXNNX.',
    '..XXXKKWWWKKKKKWWWWKKKKKWWKKXXX.',
    '....XXX.X.X..X.X.X..X.X..X.XX...',
    '....X.X.X..X.X.X..X.X.X..X......',
    '.......X.X..X.X.X..X.X.X..X.....',
    '................................',
    '................................',
    '................................',
    '................................',
    '................................',
    '................................',
  ],
};

window.REF78_TILES = REF78_TILES;
window.REF78_COLS = REF78_COLS;
