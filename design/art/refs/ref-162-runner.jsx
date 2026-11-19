// SPDX-License-Identifier: MIT
// REF-162 - Carpet runner, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// An oxblood runner with a gold edge and wineflesh ring motifs, laid
// over a plaza's ground from the street mouth to the landmark.
// runner_mid tiles vertically; runner_end is its fringed south end.
// For the Library (to the Sealed Stacks) and the Throne (to the dais).
// Walkable. Lay it one tile wide, or two side by side for the Throne.
//
// Legend: O oxblood  W wineflesh  Y crypt  G old gold
//         N antique brass  X coalblack

const REF162_COLS = 2;

const REF162_TILES = {
  runner_mid: [
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOWWOOOOONG',
    'GNOOOOWOOWOOOONG',
    'GNOOOOWOOWOOOONG',
    'GNOOOOOWWOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOYOOOOYOOONG',
    'GNOOOOYOOYOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
  ],
  runner_end: [
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOWWOOOOONG',
    'GNOOOOWOOWOOOONG',
    'GNOOOOOWWOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNOOOOOOOOOOOONG',
    'GNNNNNNNNNNNNNNG',
    'GGGGGGGGGGGGGGGG',
    'GXGXGXGXGXGXGXGX',
    '.G.G.G.G.G.G.G.G',
  ],
};

window.REF162_TILES = REF162_TILES;
window.REF162_COLS = REF162_COLS;
