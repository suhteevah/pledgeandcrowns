// REF-28 - The Chronicler, 32x32, idle, transparent bg.
// Robed historian, hands clasped behind the back (emphatically NOT
// writing). To his right a lectern bears an open tome whose entry is
// written by a brass clockwork armature - the derive machine - holding
// a self-driving quill. Derived, not hand-authored.
// Mission: derive_debug - #[derive(Debug)], the impl is generated for you.

// Legend:
//   X coalblack (outline, quill, ink rows)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin)   P pink quartz (skin hi)     B basalt (brow)
//   S stone grey (hair)   V aged paper (hair hi, undercollar)
//   Z bronze (lectern post)   N antique brass (clockwork armature)
//   G old gold (gear)   L brass leaf (gear specular, fresh-ink trail)
//   C parchment cream (tome pages)
//   . transparent

const REF28_GRID = [
  '................................', //  0
  '................................', //  1
  '.....XXXXXXXX...................', //  2 head top
  '....XSSSVVSSSX..................', //  3 hair
  '...XSSVVVVVVSSX.................', //  4
  '...XSPPPPPPPPSX.................', //  5
  '...XPPPPPPPPRRX.................', //  6 forehead
  '...XPBPPPPBPRWX.................', //  7 brow / eyes
  '...XPPRRRRRRRPX.................', //  8
  '...XXPRRRRRRPXX...XNX...........', //  9 jaw; armature arm (N)
  '...XVPPPRRRRPVX..XNGNX..........', // 10 undercollar V; gear (G)
  '..XYYYYYYYYYYYYX.XNGLNX.........', // 11 collar; gear specular (L)
  '.XYYYOOOOOOOOYYYXXNNNXX.........', // 12 shoulders; quill mount
  '.XYOOOWWWWOOOOYOX.XNX.XX........', // 13 robe; quill shaft (X)
  '.XYOOWWWWWWOOOYOX.....XLX.......', // 14 quill nib + fresh ink (L)
  '.XYOOOWWWWOOOOYOX..ZCCCCCCZ.....', // 15 tome top edge (C) on lectern (Z)
  '.XYOOOOOOOOOOOYOX..ZCXXXXCZ.....', // 16 page tally header (X)
  '.XYOOOOOOOOOOOYOX..ZCXBBBCZ.....', // 17 field row 1 (B fields)
  '.XYOOOOOOOOOOOYOX..ZCXBBBCZ.....', // 18 field row 2
  '.XYOOOOOOOOOOOYOX..ZCXBBBCZ.....', // 19 field row 3 (the struct fields)
  '.XYOOOWWWWWOOOYOX..ZCCCCCCZ.....', // 20 tome bottom edge
  '.XYOOOOOOOOOOOYOX..ZZZZZZZZ.....', // 21 lectern shelf
  '.XYYOOOOOOOOOOYYX....ZZ.........', // 22 (hands are behind the back)
  '.XYYOOWWWWWOOOYYX....ZZ.........', // 23 lectern post
  '.XYYOOOOOOOOOOYYX....ZZ.........', // 24
  '.XYYOOOOOOOOOOYYX....ZZ.........', // 25
  '.XYYYOOOOOOOOYYYX....ZZ.........', // 26 hem
  '.XYYOOOOOOOOOOYYX...XZZX........', // 27 lectern foot
  '.XXFFFXX..XXFFFXX..XXXXXX.......', // 28 leg split
  '..XZZZX....XZZZX................', // 29 boots
  '..XZUUZX..XZUUZX................', // 30
  '..XXXXXX..XXXXXX................', // 31 soles
];

window.REF28 = function REF28({ scale = 8 }) {
  return <PixelArt grid={REF28_GRID} scale={scale} title="REF-28 The Chronicler" />;
};
window.REF28_GRID = REF28_GRID;
window.REF28_ROLES = ['X','Y','O','W','R','P','B','S','V','Z','N','G','L','C','F'];
