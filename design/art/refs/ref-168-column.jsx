// SPDX-License-Identifier: MIT
// REF-168 - Column, Act 10 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/column.png.
//
// A pale fluted column with a square capital and base and a gold
// band at each end. Stand them in two rows down the Throne plaza to
// make an aisle. Bottom-centre anchor; solid over the base.
//
// Legend: X coalblack  V aged paper  C parchment cream  S stone grey
//         G old gold

const REF168_GRID = [
  '.......XXXXXXXXXXXXXXXXXX.......', //  0 capital
  '.......XCCCCCCCCCCCCCCCCX.......', //  1
  '.......XVVVVVVVVVVVVVVVVX.......', //  2
  '.......XSSSSSSSSSSSSSSSSX.......', //  3
  '.......XXXXXXXXXXXXXXXXXX.......', //  4
  '.........XGGGGGGGGGGGGX.........', //  5
  '.........XXXXXXXXXXXXXX.........', //  6 shaft
  '..........XCVVSVVSVSSX..........', //  7
  '..........XCVVSVVSVSSX..........', //  8
  '..........XCVVSVVSVSSX..........', //  9
  '..........XCVVSVVSVSSX..........', // 10
  '..........XCVVSVVSVSSX..........', // 11
  '..........XCVVSVVSVSSX..........', // 12
  '..........XCVVSVVSVSSX..........', // 13
  '..........XCVVSVVSVSSX..........', // 14
  '..........XCVVSVVSVSSX..........', // 15
  '..........XCVVSVVSVSSX..........', // 16
  '..........XCVVSVVSVSSX..........', // 17
  '..........XCVVSVVSVSSX..........', // 18
  '..........XCVVSVVSVSSX..........', // 19
  '..........XCVVSVVSVSSX..........', // 20
  '..........XCVVSVVSVSSX..........', // 21
  '..........XCVVSVVSVSSX..........', // 22
  '..........XCVVSVVSVSSX..........', // 23
  '..........XCVVSVVSVSSX..........', // 24
  '.........XXXXXXXXXXXXXX.........', // 25
  '.........XGGGGGGGGGGGGX.........', // 26
  '......XXXXXXXXXXXXXXXXXXXX......', // 27 base
  '......XCCCCCCCCCCCCCCCCCCX......', // 28
  '......XSSSSSSSSSSSSSSSSSSX......', // 29
  '......XXXXXXXXXXXXXXXXXXXX......', // 30
  '................................', // 31
];

window.REF168_GRID = REF168_GRID;
