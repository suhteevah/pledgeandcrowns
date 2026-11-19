// SPDX-License-Identifier: MIT
// REF-112 - Lantern post, The Commons district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/lantern_post.png.
//
// A basalt iron post with a brass-framed lantern and a teal flame (the
// Lifetime Lantern mission is given here). One specular-white pixel.
// Bottom-centre anchor; solid only at the foot.
//
// Legend: X coalblack  B basalt  S stone grey  N antique brass
//         G old gold  L brass leaf  I bright teal  M mist teal
//         # specular white

const REF112_GRID = [
  '................................', //  0
  '...............XX...............', //  1
  '.............XXXXXX.............', //  2 cap
  '.............XNNNNX.............', //  3
  '............XXXXXXXX............', //  4 lantern
  '............XLGGGGGX............', //  5
  '............XNIIIINX............', //  6
  '............XN#MMMNX............', //  7
  '............XNMMMMNX............', //  8
  '............XNMMMMNX............', //  9
  '............XNMMMMNX............', // 10
  '............XNIIIINX............', // 11
  '............XNNNNNNX............', // 12
  '............XXXXXXXX............', // 13
  '..............XSBX..............', // 14 post
  '..............XSBX..............', // 15
  '..............XSBX..............', // 16
  '..............XSBX..............', // 17
  '..............XSBX..............', // 18
  '..............XSBX..............', // 19
  '..............XSBX..............', // 20
  '..............XSBX..............', // 21
  '..............XSBX..............', // 22
  '..............XSBX..............', // 23
  '..............XSBX..............', // 24
  '..............XSBX..............', // 25
  '..............XSBX..............', // 26
  '...........XXXXXXXXXX...........', // 27 foot
  '...........XSSSSSSSSX...........', // 28
  '...........XBBBBBBBBX...........', // 29
  '...........XXXXXXXXXX...........', // 30
  '................................', // 31
];

window.REF112_GRID = REF112_GRID;
