// SPDX-License-Identifier: MIT
// REF-122 - Guildhall floor, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/tiles_village_v3.png (via the REF-76 combined atlas, indices 26-27).
//
// Ground for the Guildhall Quarter (Act 3): a chequered civic floor, one
// aged-paper square and one stone-grey square, each a whole tile with a
// one-pixel joint on its right and bottom edge. Lay them as a chequer
// (light where tile x + tile y is even), never at random. The Guildhall
// cast wears oxblood and reads on both squares.
//
// Legend: V aged paper  C parchment cream (lit corner)
//         S stone grey  B basalt (joint, specks)

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
    'VSSSSSSSSSSSSSSB',
    'VSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSBSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSBSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'SSSSSSSSSSSSSSSB',
    'BBBBBBBBBBBBBBBB',
  ],
};

window.REF122_TILES = REF122_TILES;
window.REF122_COLS = REF122_COLS;
