// SPDX-License-Identifier: MIT
// REF-55 - The Lighthouse Keeper, 32x32, idle, transparent bg.
// Stands beside a small lighthouse (frame-right) whose lantern room casts
// a single Teal-and-Old-gold beam out to sea. The light is SHARED - every
// ship on the coast relies on it - but only ONE keeper may tend the lamp
// at a time. So he carries the ONE key to the lantern room and holds it up:
// to touch the light you must first take the key, and while you hold it no
// one else can. That is `Arc<Mutex<T>>`: many owners share the same lamp
// (Arc), but the Mutex hands out a single key (the lock) so only one
// thread mutates it at once. Crypt coat, Teal beam, Old-gold key, no magic.
// Mission: arc_mutex - share state across threads with Arc<Mutex<T>>.
//
// Legend:
//   X coalblack (outline, lighthouse seams, key teeth)
//   K inkblood (coat deep shadow)
//   Y crypt (coat deep)   O oxblood (coat primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi, tower band)
//   Z bronze (tower base, boots)   N antique brass (lantern frame, belt)
//   G old gold (key, lamp core)   L brass leaf (key spec)
//   U bog umber (boot shadow)   F pine (hem)
//   D deep teal (beam shadow)   T main teal (lamp glow, beam)   I bright teal (beam hi)
//   . transparent
//
// The single Old-gold key (G with L spec) held up in his hand is the Mutex
// lock - one key, one keeper at the lamp. The Teal beam (T/I) fanning from
// the lantern room is the shared light every ship depends on (the Arc).

const REF55_GRID = [
  '...................NNNN.........', //  0 lantern room frame
  '..........XXXXXXXX.NGGN.IIII....', //  1 head top | lamp core (G) | beam hi
  '.........XSSSVVSSSXNTGTN.IIII...', //  2 hair | lamp glow (T) | beam
  '........XSSVVVVVVSSNTTTNT.IID...', //  3 | beam fans out (T)
  '........XSPPPPPPPPSNTTTND..D....', //  4 forehead | beam tail
  '........XPPPPPPPPRRNNNNN........', //  5 | lantern sill (N)
  '........XPBPPPPBPRWXVVVV........', //  6 brow/eyes | tower band (V)
  '.......GXPPRRRRRRRPXZZZZ........', //  7 raised key (G) | tower body (Z)
  '......GLGXPRRRRRRPXXZNZZ........', //  8 key bow (G/L) | tower seam (N)
  '......XGXYPPPRRRPYXXZZZZ........', //  9 key shaft | tower
  '......XGXYYYYYYYYYYXVVVV........', // 10 key teeth | tower band
  '.....XXGXXYOOOOOOOYYZZZZ........', // 11 key end | tower
  '.......XYYYOOOWWWOOOYXZNZZ......', // 12 coat upper | tower seam
  '......XYYOOOWWWWWOOOYYXZZZ......', // 13 coat lit | tower
  '......XYOOOOOOOOOOOOOYXVVVV.....', // 14 | tower band
  '......XYOOOWWWWWWOOOOYXZZZZ.....', // 15 | tower
  '......XXNGGGGGGGGNXXXXZNZZ......', // 16 belt (N) buckle (G) | tower
  '......XYYOOOOOOOOYYKXXZZZZ......', // 17 coat lower | tower
  '......XYYOOOWWWWOOYYKXXVVVV.....', // 18 | tower band
  '.....XKYYOOOOOOOOOYYKXXZZZZ.....', // 19 | tower base flare
  '.....XKYYOOOWWWWWOOYYKXZZZZZ....', // 20 | tower base
  '.....XKKYOOOOOOOOOOYYKKZZZZZ....', // 21 widest | base
  '.....XKYYFFFFFFFFFFFFYKXZZZZ....', // 22 hem | base
  '.....XKFFFFFXXX..XXXFFFFKXXXX...', // 23 leg split | base ground
  '.....XXZZZZXX....XXZZZZZX.......', // 24 boots
  '......XZUUZX......XZUUUZX.......', // 25
  '......XXXXXX......XXXXXXX.......', // 26 soles
  '................................', // 27
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF55 = function REF55({ scale = 8 }) {
  return <PixelArt grid={REF55_GRID} scale={scale} title="REF-55 The Lighthouse Keeper" />;
};
window.REF55_GRID = REF55_GRID;
window.REF55_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','D','T','I'];
