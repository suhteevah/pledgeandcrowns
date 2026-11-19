// SPDX-License-Identifier: MIT
// REF-138 - Tavern plank floor, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices 30-31).
//
// Ground for Tavern Row (Act 5): wide bronze floorboards, two to a
// tile, with a bog-umber seam between them, one butt joint and one
// brass-lit fleck per board. Second drawing: the first had four narrow
// boards a tile with knots and read as busy; the owner agreed. Mix the
// two tiles freely. Dark and warm on purpose: the Tavern cast wears
// parchment-cream shirtsleeves.
//
// Legend: Z bronze  U bog umber  N antique brass

const REF138_COLS = 2;

const REF138_TILES = {
  plank_a: [
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZNZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZZZZZZZZUZZZZZ',
    'ZZZZZZZZZZUZZZZZ',
    'ZZZZZZZZZZUZZZZZ',
    'ZZZZZZZZZZUZZZZZ',
    'ZZZZZZZZZZUZZZNZ',
    'ZZZZZZZZZZUZZZZZ',
    'ZZZZZZZZZZUZZZZZ',
    'UUUUUUUUUUUUUUUU',
  ],
  plank_b: [
    'ZZZZUZZZZZZZZZZZ',
    'ZZZZUZZZZZZZZZZZ',
    'ZZZZUZZZZZZZZZZZ',
    'ZZZZUZZZZZZZZZZZ',
    'ZZZZUZZZZZZNZZZZ',
    'ZZZZUZZZZZZZZZZZ',
    'ZZZZUZZZZZZZZZZZ',
    'UUUUUUUUUUUUUUUU',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZNZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'ZZZZZZZZZZZZZZZZ',
    'UUUUUUUUUUUUUUUU',
  ],
};

window.REF138_TILES = REF138_TILES;
window.REF138_COLS = REF138_COLS;
