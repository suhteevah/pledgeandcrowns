// REF-26 - The Quartermaster, 32x32, idle, transparent bg.
// Re-encoded 2026-06-20 from the Matt-approved claude.ai/design sprite
// (NPC Sprites artboard, npc-roster.jsx). Faithful raster of the approved
// design, replacing the first-pass hand-authored grid.
// Mission: slice_basic - sum a borrowed slice you do not own.

const REF26_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXX.................', //  2
  '.........XSVSSSX................', //  3
  '.........XSVSSSX................', //  4
  '.........XPPPRWX................', //  5
  '.........XPPPRWX................', //  6
  '.........XPXPXWX................', //  7
  '.........XPPPRWX................', //  8
  '.........XWPRRWX................', //  9
  '.........XWRRNWX................', // 10
  '.....XYYYYYYYYYYYYX.............', // 11
  '.....XWWWWYYYYKKKKX.............', // 12
  '.....XWWWWYYYYKKKKX...UUUUUUUUU.', // 13
  '.....XWWWWYYYYKKKKX.BSUXLGGXLNU.', // 14
  '.....XWWWWYYYYKKKKX.B.UXGGZX.NU.', // 15
  '.....XWWWWYYYYKKKKXBX.UUUUUU.NU.', // 16
  '.....XWWWWYYYYKKKKXR..UXLGGX.NU.', // 17
  '.....XWWWWYYYYKKKKX...UXGGZX.NU.', // 18
  '.....XWWWWYYYYKKKKX...UUUUUU.NU.', // 19
  '.....XBBBBBNBBBBBVX...UXLGGX.NU.', // 20
  '.....XWWWWYYYYKKKKX...UXGGZXNNU.', // 21
  '.....XWWWWYYYYKKKKX...UUUUUU..U.', // 22
  '.....XWWWWYYYYKKKKX...UXGZGX..U.', // 23
  '.....XWWWWYYYYKKKKX...UXZZZX..U.', // 24
  '.....XWWWWYYYYKKKKX...UUUUUU..U.', // 25
  '.....XWWWWYYYYKKKKX...UXGZGX..U.', // 26
  '.....XWWWWYYYYKKKKX...UXZZZX..U.', // 27
  '.....XKKKKKKKKKKKKX...UUUUUU..U.', // 28
  '.......XZZX...XZZX....UUUUUUUUU.', // 29
  '.......XXXX...XXXX..............', // 30
  '................................', // 31
];

window.REF26 = function REF26({ scale = 8 }) {
  return <PixelArt grid={REF26_GRID} scale={scale} title="REF-26 The Quartermaster" />;
};
window.REF26_GRID = REF26_GRID;
window.REF26_ROLES = ['X','Y','K','W','S','V','P','R','N','G','L','Z','B','U'];
