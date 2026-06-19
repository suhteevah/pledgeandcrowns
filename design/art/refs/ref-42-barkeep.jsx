// SPDX-License-Identifier: MIT
// REF-42 - The Barkeep, 32x32, idle, transparent bg.
// Stands behind a Bronze bar/counter, a mug in each hand. The frame-left
// mug stands UPRIGHT and FULL - Old-gold ale brimming, a Brass-leaf foam
// head: the Ok(value). The frame-right mug is TIPPED and spilling - empty,
// its ale gone, a single Alarm-scarlet drip reading as the Err. To use the
// drink the player must `match` on the Result and handle BOTH arms: the
// full mug (Ok) and the spilled mug (Err). Crypt apron-tunic, no magic.
// Mission: result_match - `match` on a `Result`, handle Ok and Err.
//
// Legend:
//   X coalblack (outline, bar edge, mug rims)
//   K inkblood (apron deep shadow)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi, apron strings)
//   Z bronze (bar top, mug bodies)   N antique brass (mug bands)
//   G old gold (ale fill)   L brass leaf (foam head, ale spec)
//   U bog umber (bar shadow)   F pine (hem)
//   ! alarm scarlet (single Err drip - <=1%)
//   . transparent
//
// Ale = the gold ramp (G fill, L foam). The lone ! pixel is the spilled
// mug's last drip, the Err the player must not forget to handle.

const REF42_GRID = [
  '................................', //  0
  '...........XXXXXXXX.............', //  1 head top
  '..........XSSSVVSSSX............', //  2 hair
  '.........XSSVVVVVVSSX...........', //  3
  '.........XSPPPPPPPPSX...........', //  4 forehead
  '.........XPPPPPPPPRRX...........', //  5
  '.........XPBPPPPBPRWX...........', //  6 brow/eyes
  '.........XPPRRRRRRRPX...........', //  7
  '........XXPRRRRRRPXX............', //  8 jaw
  '........XYPPPRRRRPYX............', //  9 collar
  '.LLL...XYYYYYYYYYYYYX...XLX.....', // 10 foam head (L) | tipped mug spill start
  '.XGX...XYYYOOOOOOOOYYX.XNGNX....', // 11 full mug body | tipped mug (rim down)
  '.NGN..XYYYOOOWWWWOOOYYX.XGGX....', // 12 mug bands (N) | ale spilling out
  '.ZGZ.PXYOOOWWWWWWOOOYXP.XZX.....', // 13 hands cup the mugs (P) | empty mug
  '.ZGZPRPYOOOOOOOOOOOOYPRP.!......', // 14 full ale (G) | spilled drip (!)
  '.ZZZPPPYYOOOWWWWWOOYYPPP........', // 15 mug feet | hands (P)
  '....XXNGGGGGGGGGGGGNXX..........', // 16 apron waistband (N/G accent? -> N)
  '....XKYYOOOOOOOOOOYYKX..........', // 17 apron over tunic
  '....XKYYOOOWWWWWWOOYYKX.........', // 18
  '...XKKYYOOOOOOOOOOOOYYKKX.......', // 19 apron widens onto bar
  '..XZZZZZZZZZZZZZZZZZZZZZZX......', // 20 bar top edge (Z)
  '..XUUUUUUUUUUUUUUUUUUUUUUX......', // 21 bar front (U)
  '..XUZZUZZUZZUZZUZZUZZUZZUX......', // 22 bar plank seams
  '..XUUUUUUUUUUUUUUUUUUUUUUX......', // 23
  '..XUUUUUUUUUUUUUUUUUUUUUUX......', // 24
  '..XZZZZZZZZZZZZZZZZZZZZZZX......', // 25 bar base trim
  '......XKYFFFFFFFFFFYKX..........', // 26 hem below bar
  '......XKFFFFFXX.XFFFFKX.........', // 27 leg split
  '......XXZZZZXX..XXZZZZX.........', // 28 boots
  '.......XZUUZX....XZUUZX.........', // 29
  '.......XXXXXX....XXXXXX.........', // 30 soles
  '................................', // 31
];

window.REF42 = function REF42({ scale = 8 }) {
  return <PixelArt grid={REF42_GRID} scale={scale} title="REF-42 The Barkeep" />;
};
window.REF42_GRID = REF42_GRID;
window.REF42_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','!'];
