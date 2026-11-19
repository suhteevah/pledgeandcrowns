// SPDX-License-Identifier: MIT
// REF-130 - Tower court floor, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// Ground for the Mage's Tower district (Act 4): pale aged-paper slabs,
// one whole slab per tile, and a second tile with a teal diamond inlaid
// in it. Use the inlaid tile for about one tile in five, or in a ring
// round the tower. Pale on purpose: the Tower cast wears cobalt-deep
// robes. Teal carries the magic here; violet stays on the landmark.
//
// Legend: V aged paper  C parchment cream (lit corner)  S stone grey
//         T main teal  I bright teal

const REF130_COLS = 2;

const REF130_TILES = {
  court_plain: [
    'CVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVSVVVS',
    'VVVVVVVVVVVVVVVS',
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
  court_rune: [
    'CVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVTVVVVVVVS',
    'VVVVVVTITVVVVVVS',
    'VVVVVTVVVTVVVVVS',
    'VVVVTVVVVVTVVVVS',
    'VVVTVVVIVVVTVVVS',
    'VVTIVVITIVVITVVS',
    'VVVTVVVIVVVTVVVS',
    'VVVVTVVVVVTVVVVS',
    'VVVVVTVVVTVVVVVS',
    'VVVVVVTITVVVVVVS',
    'VVVVVVVTVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'VVVVVVVVVVVVVVVS',
    'SSSSSSSSSSSSSSSS',
  ],
};

window.REF130_TILES = REF130_TILES;
window.REF130_COLS = REF130_COLS;
