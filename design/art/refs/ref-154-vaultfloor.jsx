// SPDX-License-Identifier: MIT
// REF-154 - Vault floor, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices 32-33).
//
// Ground for the Vault (Act 8): dark basalt slabs, one per tile, with a
// coalblack joint and one old-gold stud at the corner; the second tile
// carries a thin antique-brass ring inlay (use it for about one tile
// in twelve). Second drawing (owner agreed 2026-11-19): the first ring
// was a double gold-and-brass line on one tile in six and read as
// busy. The darkest ground in the game on purpose: the Vault cast
// wears stone-grey plate and would vanish on a grey floor.
//
// Legend: B basalt  X coalblack  S stone grey  G old gold
//         N antique brass

const REF154_COLS = 2;

const REF154_TILES = {
  vault_plain: [
    'GBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBSBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBSBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'XXXXXXXXXXXXXXXX',
  ],
  vault_ring: [
    'GBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBNNBBBBBBX',
    'BBBBBBNBBNBBBBBX',
    'BBBBBNBBBBNBBBBX',
    'BBBBNBBBBBBNBBBX',
    'BBBNBBBBBBBBNBBX',
    'BBBNBBBBBBBBNBBX',
    'BBBBNBBBBBBNBBBX',
    'BBBBBNBBBBNBBBBX',
    'BBBBBBNBBNBBBBBX',
    'BBBBBBBNNBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'BBBBBBBBBBBBBBBX',
    'XXXXXXXXXXXXXXXX',
  ],
};

window.REF154_TILES = REF154_TILES;
window.REF154_COLS = REF154_COLS;
