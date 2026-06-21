// REF-28 - The Chronicler, 32x32, idle, transparent bg.
// Re-encoded 2026-06-20 from the Matt-approved claude.ai/design sprite
// (NPC Sprites artboard, npc-roster.jsx). Faithful raster of the approved
// design. Tall, lanky, stooped, long hair; hands clasped behind the back
// while a brass clockwork armature drives a self-writing quill on the
// lectern tome - derived, not hand-authored.
// Mission: derive_debug - #[derive(Debug)], the impl is generated for you.

const REF28_GRID = [
  '................................', //  0
  '................................', //  1
  '........SSSSSS..................', //  2
  '.......SXSSSSS..................', //  3
  '.......SXPRRWS..................', //  4
  '.......SXPRRWS..................', //  5
  '.......SXPRXWS..................', //  6
  '.......SXPRRWS..................', //  7
  '.......SXPXXWS..................', //  8
  '.......SXXXXXS..................', //  9
  '.......S.VZZVS..................', // 10
  '.......SXXXXXX............NNL...', // 11
  '.......XRRWYKX.............GG...', // 12
  '.......XRRWYKX............XN....', // 13
  '......XRRWYKX...XZZZZZZZZXZNZX..', // 14
  '......XRRWYKX...XCCCCCXCXCCCCX..', // 15
  '......XRRWYKX...XZXXXZXCLLXXCX..', // 16
  '.....XRRWYKX....XCXXXCXCXXXCCX..', // 17
  '.....XRRWYKX....XZXXXZXCCCCCCX..', // 18
  '.....XRRWYKX....XCCCCCXCCCCCCX..', // 19
  '.....RRRWYKX....XCCCCCXCCCCCCX..', // 20
  '.....PRRWYKX....XCCCCCXCCCCCCX..', // 21
  '.....XRRWYKX....XXXXXXXXXXXXXX..', // 22
  '.....XRRWYKX.........UUU........', // 23
  '.....XRRWYKX.........UUU........', // 24
  '.....XRRWYKX.........UUU........', // 25
  '.....XRRWYKX.........UUU........', // 26
  '.....XRRWYKX.........UUU........', // 27
  '.....XRRWYKX.......ZZZZZZZZ.....', // 28
  '.....XKKKKKX.......UUUUUUUU.....', // 29
  '.....XXXXXXX....................', // 30
  '................................', // 31
];

window.REF28 = function REF28({ scale = 8 }) {
  return <PixelArt grid={REF28_GRID} scale={scale} title="REF-28 The Chronicler" />;
};
window.REF28_GRID = REF28_GRID;
window.REF28_ROLES = ['X','S','P','R','W','V','Z','N','L','G','K','U','C'];
