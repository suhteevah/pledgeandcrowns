// REF-27 - The Auditor, 32x32, idle, transparent bg.
// Re-encoded 2026-06-20 from the Matt-approved claude.ai/design sprite
// (NPC Sprites artboard, npc-roster.jsx). Faithful raster of the approved
// design. Tall, thin, ramrod-straight; open ledger with the single Alarm
// scarlet (!) Err seal - the documented sub-1% exception.
// Mission: result_question_mark - propagate the error with ?.

const REF27_GRID = [
  '...........SS...................', //  0
  '...........SSS..................', //  1
  '..........XSSSX.................', //  2
  '..........XPRWX.................', //  3
  '..........XPRWX.................', //  4
  '..........GLRXX.................', //  5
  '.........NXPRWX.................', //  6
  '.........NXPXWX.................', //  7
  '..........NPRWX.................', //  8
  '..........XXXXX.................', //  9
  '.........XVVVVVX................', // 10
  '.........XXXXXXX................', // 11
  '.........XWWOYKX................', // 12
  '........XNNNNNNNX...............', // 13
  '........XCXXXXXCX...............', // 14
  '........XCX!XCCCX...............', // 15
  '.......RXCCCXCCCXR..............', // 16
  '.......RXCCCXCCCX...............', // 17
  '........XCCCXCCCX...............', // 18
  '........XCCCXCCCX...............', // 19
  '........XNNNNNNNX...............', // 20
  '.........XWWOYKX................', // 21
  '.........XWWOYKX................', // 22
  '.........XWWOYKX................', // 23
  '.........XWWOYKX................', // 24
  '.........XWWOYKX................', // 25
  '.........XWWOYKX................', // 26
  '.........XWWOYKX................', // 27
  '.........XWWOYKX................', // 28
  '.........XKKKKKX................', // 29
  '.........XXXXXXX................', // 30
  '................................', // 31
];

window.REF27 = function REF27({ scale = 8 }) {
  return <PixelArt grid={REF27_GRID} scale={scale} title="REF-27 The Auditor" />;
};
window.REF27_GRID = REF27_GRID;
window.REF27_ROLES = ['X','S','V','P','R','W','O','K','N','G','L','C','!'];
