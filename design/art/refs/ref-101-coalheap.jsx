// SPDX-License-Identifier: MIT
// REF-101 - Coal heap, Act 6 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A low mound of basalt lumps with coalblack gaps and stone-grey glints,
// one live coal near the middle (one alarm-scarlet pixel beside one
// antique-brass pixel). Bottom-centre anchor, solid over the mound.
//
// Legend: X coalblack  B basalt  S stone grey  N antique brass
//         ! alarm scarlet

const REF101_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '................................', //  4
  '................................', //  5
  '................................', //  6
  '................................', //  7
  '................................', //  8
  '................................', //  9
  '................................', // 10
  '................................', // 11
  '................................', // 12
  '................................', // 13
  '................................', // 14
  '................................', // 15
  '.............XXXXXX.............', // 16 top of the mound
  '...........XBBSBBBBBX...........', // 17
  '.........XBBBBXBBSBBBBX.........', // 18
  '........XBSBBBXBBBBXBBBX........', // 19
  '.......XBBBXBBBBSBBXBBSBX.......', // 20
  '......XBXBBBBXBBBBBXBBBBBX......', // 21
  '.....XBBBBSBXBBBNBBXBBSBBBX.....', // 22
  '....XBBXBBBBBXBB!NBBXBBBBXBX....', // 23 the live coal
  '...XBSBBBXBBBBBXBBBBBXBBSBBBX...', // 24
  '...XBBBXBBBBSBBBXBBBXBBBBBXBX...', // 25
  '..XBBXBBBBBXBBBBBBXBBSBBXBBBBX..', // 26
  '..XBBBBBXBBBSBBXBBBBBXBBBBBBBX..', // 27
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF101_GRID = REF101_GRID;
