// SPDX-License-Identifier: MIT
// REF-148 - Bollard, Act 7 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A basalt iron mooring bollard with three turns of hayfield rope and
// the line running off to the right. Place them along the dock edge.
// Bottom-centre anchor; solid over the bollard.
//
// Legend: X coalblack  B basalt  S stone grey  Q hayfield
//         C parchment cream

const REF148_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '.........XXXXXXXXXXXXXX.........', //  5 cap
  '.........XSSSSSSSSSSSSX.........', //  6
  '.........XBBBBBBBBBBBBX.........', //  7
  '.........XBBBBBBBBBBBBX.........', //  8
  '.........XXXXXXXXXXXXXX.........', //  9
  '............XSBBBBBX............', // 10
  '............XSBBBBBX............', // 11
  '............XSBBBBBX............', // 12
  '............XSBBBBBX............', // 13
  '.........XXXXXXXXXXXXXX.........', // 14 rope turns
  '.........XCQQQQQQQQQQQX.........', // 15
  '.........XXXXXXXXXXXXXX.........', // 16
  '.........XXXXXXXXXXXXXX.........', // 17
  '.........XCQQQQQQQQQQQX.........', // 18
  '.........XXXXXXXXXXXXXX.........', // 19
  '.........XXXXXXXXXXXXXX.........', // 20
  '.........XCQQQQQQQQQQQX.........', // 21
  '.........XXXXXXXXXXXXXXQQ.......', // 22
  '............XSBBBBBX...XXQQ.....', // 23
  '............XSBBBBBX.....XXQQ...', // 24
  '............XSBBBBBX.......XXQQ.', // 25
  '........XXXXXXXXXXXXXXXX.....XX.', // 26 foot
  '........XSSSSSSSSSSSSSSX........', // 27
  '........XBBBBBBBBBBBBBBX........', // 28
  '........XBBBBBBBBBBBBBBX........', // 29
  '........XXXXXXXXXXXXXXXX........', // 30
  '................................', // 31
];

window.REF148_GRID = REF148_GRID;
