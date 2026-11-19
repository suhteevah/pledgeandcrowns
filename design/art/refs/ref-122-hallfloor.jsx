// SPDX-License-Identifier: MIT
// REF-122 - Guildhall floor, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices 26-27).
//
// Ground for the Guildhall Quarter (Act 3): a chequered civic floor of
// aged-paper slabs, each a whole tile with a one-pixel joint on its
// right and bottom edge. One square is plain; the other carries an
// inlaid stone-grey line frame. Lay them as a chequer (plain where
// tile x + tile y is even), never at random. Second drawing (owner
// agreed 2026-11-19): the first had a solid stone-grey square, and the
// full-contrast check was the loudest ground in the game and drowned
// the props. The Guildhall cast wears oxblood and reads on both.
//
// Legend: V aged paper  C parchment cream (lit corner)
//         S stone grey (joint, inlay, speck)

const REF122_COLS = 2;

const REF122_TILES = {
  hall_light: [
    'CCVVVVVVVVVVVVVS',
    'CVVVVVVVVVVVVVVS',
    'CVVVVVVVVVVVVVVS',
    'CVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVSVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVSVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'SSSSSSSSSSSSSSSS',
  ],
  hall_dark: [
    'CCVVVVVVVVVVVVVS',
    'CVVVVVVVVVVVVVVS',
    'VVSSSSSSSSSSSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSVVVVVVVVVSVVS',
    'VVSSSSSSSSSSSVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'SSSSSSSSSSSSSSSS',
  ],
};

window.REF122_TILES = REF122_TILES;
window.REF122_COLS = REF122_COLS;
