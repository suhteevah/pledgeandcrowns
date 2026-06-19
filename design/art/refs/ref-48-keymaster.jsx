// SPDX-License-Identifier: MIT
// REF-48 - The Keymaster, 32x32, idle, transparent bg.
// Stands before a rack of labeled pigeonhole slots (frame-left/back), each
// slot a keyed cubby. In his raised hand a ring of Old-gold keys, one key
// per slot. To fetch a thing you do not search every slot - you carry the
// KEY and the key takes you straight to its cubby. That is a HashMap: each
// value parked in a slot named by its key, looked up by key in one move.
// Crypt keeper's tunic, Old-gold keys and ring, Antique-brass slot frames,
// no magic.
// Mission: hashmap_basic - insert and get by key in a HashMap.
//
// Legend:
//   X coalblack (outline, key teeth, slot edges)
//   K inkblood (tunic deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (slot interiors, boots)   N antique brass (slot frames, belt)
//   G old gold (key ring, key bows)   L brass leaf (key spec)
//   U bog umber (slot shadow, boot shadow)   F pine (hem)
//   . transparent
//
// The key ring (G) with its L spec is the one bright mass: the key the
// player holds. The brass slots are the named cubbies it unlocks.

const REF48_GRID = [
  '................................', //  0
  '.NNNNNNNN.....XXXXXXXX..........', //  1 rack top frame | head top
  '.NZZNZZZN....XSSSVVSSSX.........', //  2 slot row 1 | hair
  '.NUZNUZZN...XSSVVVVVVSSX........', //  3 slot interiors (U/Z) | hair
  '.NNNNNNNN...XSPPPPPPPPSX........', //  4 frame band | forehead
  '.NZZNZZZN...XPPPPPPPPRRX........', //  5 slot row 2
  '.NUZNUZZN...XPBPPPPBPRWX........', //  6 brow/eyes
  '.NNNNNNNN...XPPRRRRRRRPX........', //  7 frame band
  '.NZZNZZZN..XXPRRRRRRPXX.........', //  8 slot row 3 | jaw
  '.NUZNUZZN..XYPPPRRRRPYX.........', //  9 | collar
  '.NNNNNNNN..XYYYYYYYYYYYX.GGG....', // 10 frame band | shoulders | key ring top
  '...........XYYOOOOOOOOYYX.GLG...', // 11 robe upper | ring (G) w/ L
  '..........XYYYOOOWWWOOOYYX.GG...', // 12 robe lit | key bow
  '.........XYYOOOWWWWWOOOYYXPGG...', // 13 hand reaches (P) | keys
  '.........XYOOOOOOOOOOOOYXPLGX...', // 14 | hand grips ring (P) key spec
  '.........XYOOOWWWWWOOOYXPGGX....', // 15 | keys hang
  '.........XXNGGGGGGGGNXX.GXGX....', // 16 belt band (N) | key teeth (X)
  '.........XKYYOOOOOOYYKX..XGX....', // 17 | key shaft
  '........XKYYOOWWWWOOYYKX.XGX....', // 18 | key tooth
  '........XKKYOOOOOOOYYKKX..XX....', // 19 robe widens | key end
  '.......XKYYOOOWWWWWOOOYYKX......', // 20
  '.......XKYYOOOOOOOOOOOYYKX......', // 21
  '.......XKYYOOOOOOOOOOOYYKX......', // 22
  '......XKKYYOOOOOOOOOOOYYKKX.....', // 23 widest
  '......XKYYFFFFFFFFFFFFFYKX......', // 24 hem
  '......XKFFFFFXXX..XXFFFFKX......', // 25 leg split
  '......XXZZZZXX......XXZZZZX.....', // 26 boots
  '.......XZUUZX........XZUUZX.....', // 27
  '.......XXXXXX........XXXXXX.....', // 28 soles
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF48 = function REF48({ scale = 8 }) {
  return <PixelArt grid={REF48_GRID} scale={scale} title="REF-48 The Keymaster" />;
};
window.REF48_GRID = REF48_GRID;
window.REF48_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F'];
