// SPDX-License-Identifier: MIT
// REF-75 - Plaza flagstone, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-14. Not yet wired into the game.
//
// Why: REF-66 cobble (stone grey pavers, basalt mortar) is right for streets
// but across a 600 px plaza it reads as a brick wall. These are large pale
// slabs (aged paper) with stone-grey joints: about a third of the joint
// density and far lower contrast, so villagers and props sit ON the plaza
// instead of competing with it. Intended use: plaza interiors in flagstone,
// with a one-tile REF-66 cobble border and cobble for the streets.
//
// Each tile is two courses of slabs. Courses end on a full joint row, so
// flag_a and flag_b can be mixed freely; the second course is open at both
// ends in both tiles, so its slabs run across tile boundaries.
//
// Legend: V aged paper (slab)  C parchment cream (lit corner)
//         S stone grey (joints, wear specks)

const REF75_COLS = 2;

const REF75_TILES = {
  flag_a: [
    'CCVVVVSCCCVVVVVS',
    'CVVVVVSCVVVVVVVS',
    'VVVVVVSVVVVVVVVS',
    'VVVVVVSVVVVSVVVS',
    'VVVVVVSVVVVVVVVS',
    'VVSVVVSVVVVVVVVS',
    'VVVVVVSVVVVVVVVS',
    'SSSSSSSSSSSSSSSS',
    'VVSCCVVVVVSCCVVV',
    'VVSCVVVVVVSCVVVV',
    'VVSVVVVVVVSVVVVV',
    'VVSVVVVSVVSVVVVV',
    'VVSVVVVVVVSVVVVV',
    'VVSVVVVVVVSVVVSV',
    'VVSVVVVVVVSVVVVV',
    'SSSSSSSSSSSSSSSS',
  ],
  flag_b: [
    'CCVVVVVVVSCCVVVS',
    'CVVVVVVVVSCVVVVS',
    'VVVVVVVVVSVVVVVS',
    'VVVVVVVVVSVVVVVS',
    'VVVVSVVVVSVVVVVS',
    'VVVVVVVVVSVVVVVS',
    'VVVVVVVVVSVVVVVS',
    'SSSSSSSSSSSSSSSS',
    'VVVVVSCCVVVVSCCV',
    'VVVVVSCVVVVVSCVV',
    'VVVVVSVVVVVVSVVV',
    'VVVVVSVVVVVVSVVV',
    'VVVVVSVVVSVVSVVV',
    'VVVVVSVVVVVVSVVV',
    'VVVVVSVVVVVVSVVV',
    'SSSSSSSSSSSSSSSS',
  ],
};

window.REF75_TILES = REF75_TILES;
window.REF75_COLS = REF75_COLS;
