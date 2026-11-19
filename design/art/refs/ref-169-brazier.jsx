// SPDX-License-Identifier: MIT
// REF-169 - Brazier, Act 10 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A basalt iron fire bowl with a gold rim on a tripod, gold-ramp
// flames over a cream bed and three alarm-scarlet tips. Place them in
// pairs between the columns. Bottom-centre anchor; solid at the
// tripod.
//
// Legend: X coalblack  B basalt  S stone grey  G old gold
//         L brass leaf  N antique brass  C parchment cream
//         ! alarm scarlet

const REF169_GRID = [
  '................................', //  0
  '...............!................', //  1 flames
  '.............!.G.!..............', //  2
  '.............GNG.G..............', //  3
  '...........N.GGLNG.N............', //  4
  '...........GNLGLGLNG............', //  5
  '.........N.GGLLLGLGG.N..........', //  6
  '.........GNLGLLLLLGLNG..........', //  7
  '........NGGLLLLLLLLLGGN.........', //  8
  '........GLGLLLLLLLLLGLGN........', //  9
  '........GLLLCCCCCCCCLLGG........', // 10
  '.....XLGGGGGGGGGGGGGGGGGGGX.....', // 11 bowl
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 12
  '......XBBBBBBBBBBBBBBBBBBX......', // 13
  '.......XBBBBBBBBBBBBBBBBX.......', // 14
  '........XBBBBBBBBBBBBBBX........', // 15
  '.........XBBBBBBBBBBBBX.........', // 16
  '..........XBBBBBBBBBBX..........', // 17 tripod
  '...........XBBBBBBBBX...........', // 18
  '.............BBSSBB.............', // 19
  '.............BBSSBB.............', // 20
  '............BB.SS.BB............', // 21
  '............BB.SS.BB............', // 22
  '...........BB..SS..BB...........', // 23
  '...........BB..SS..BB...........', // 24
  '..........BB...SS...BB..........', // 25
  '..........BB...SS...BB..........', // 26
  '.........BB....SS....BB.........', // 27
  '.........BB....SS....BB.........', // 28
  '........BB.....SS.....BB........', // 29
  '.......XXXX...XXXX...XXXX.......', // 30
  '................................', // 31
];

window.REF169_GRID = REF169_GRID;
