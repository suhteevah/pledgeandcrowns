// SPDX-License-Identifier: MIT
// REF-146 - Wharf dock planks and harbour water, 16x16 tiles, 4-column sheet.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// Ground for the Wharf (Act 7). dock_a and dock_b are weathered
// aged-paper boards running north to south with stone-grey seams and
// bronze nail heads; mix them freely. water_a and water_b are the same
// teal water with its glints moved two pixels: swap them every half
// second for a slow swell. Water is not walkable. Pale boards on
// purpose: the Coast cast wears cobalt.
//
// Legend: V aged paper  S stone grey  Z bronze
//         T main teal  I bright teal  D deep teal

const REF146_COLS = 4;

const REF146_TILES = {
  dock_a: [
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VZVVVSVVVVVVZVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVSVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVSVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
    'VZVVVSVVVVVVZVVS',
    'VVVVVSVVVVVVVVVS',
    'VVVVVSVVVVVVVVVS',
  ],
  dock_b: [
    'VVVVVVVVVVSVVVVS',
    'VVVVVVVVVVSVVVVS',
    'VVZVVVVVVVSVVZVS',
    'VVVVVVVVVVSVVVVS',
    'VVVVSVVVVVSVVVVS',
    'VVVVVVVVVVSVVVVS',
    'VVVVVVVVVVSVVVVS',
    'SSSSSSSSSSSVVVVS',
    'VVVVVVVVVVSVVVVS',
    'VVVVVVVVVVSVVVVS',
    'VVVVVVVVVVSVVVVS',
    'VVVVVVVSVVSVVVVS',
    'VVVVVVVVVVSVVVVS',
    'VVZVVVVVVVSVVZVS',
    'VVVVVVVVVVSVVVVS',
    'VVVVVVVVVVSVVVVS',
  ],
  water_a: [
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTIIITTTTTTTTTT',
    'TTTTTTTTTTTTTDDT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTTTIIIITTT',
    'TDDTTTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTIITTTTTTTTTT',
    'TTTTTTTTTTTDDDTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TIIITTTTTTIITTTT',
    'TTTTTTTDDTTTTTTT',
    'TTTTTTTTTTTTTTTT',
  ],
  water_b: [
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTIIITTTTTTTT',
    'TTTTTTTTTTTDDTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTIIIITTTTT',
    'TTTDDTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTTIITTTTTTTT',
    'TTTTTTTTTDDDTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
    'TTTIIITTTTTTIITT',
    'TTTTTDDTTTTTTTTT',
    'TTTTTTTTTTTTTTTT',
  ],
};

window.REF146_TILES = REF146_TILES;
window.REF146_COLS = REF146_COLS;
