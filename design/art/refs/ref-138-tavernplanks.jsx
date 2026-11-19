// SPDX-License-Identifier: MIT
// REF-138 - Tavern plank floor, 16x16 tiles, 2-column sheet.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Ground for Tavern Row (Act 5): bronze floorboards, four to a tile,
// with bog-umber seams, butt joints and knots and a few brass-lit
// flecks. Mix the two tiles freely. Dark and warm on purpose: the
// Tavern cast wears parchment-cream shirtsleeves.
//
// Legend: Z bronze  U bog umber  N antique brass

const REF138_COLS = 2;

const REF138_TILES = {
  plank_a: [
    'ZZZZZZZZZZZNZZZZ',
    'ZZNZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZUZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZZZZUZZZZZZZZZ',
    'ZZZZZZUZZZZNZZZZ',
    'ZNZZZZUZZZZZZZZZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZZZZZZZZZZZNZZ',
    'ZZZZZNZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZZZZZZZZZUZZZZ',
    'ZZNZZZZZZZZUZZZZ',
    'ZZZZZZZZZZZUZZNZ',
    'UUUUUUUUUUUUUUUU',
  ],
  plank_b: [
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZNZZZZZZZZZZ',
    'ZZZZZZZZZZUUZZZZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZUZZZZZZZZZZZZ',
    'ZZZUZZZZZZZZNZZZ',
    'ZZZUZZZNZZZZZZZZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZZZZZZZZZZZZZZ',
    'ZNZZZZZUUZZZZZZZ',
    'ZZZZZZZUUZZZZZZZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZZZZZZZZZZZUZZ',
    'ZZZZZZZZNZZZZUZZ',
    'ZZZZZZZZZZZZZUZZ',
    'UUUUUUUUUUUUUUUU',
  ],
};

window.REF138_TILES = REF138_TILES;
window.REF138_COLS = REF138_COLS;
