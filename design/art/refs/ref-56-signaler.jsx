// SPDX-License-Identifier: MIT
// REF-56 - The Signaler, 32x32, idle, transparent bg.
// Stands on the seawall with TWO semaphore flags - one raised high to
// frame-left, one lowered to frame-right - spelling a message out across
// the water to the next station down the line. He holds the SENDING end;
// far off, a receiver reads each flag as it arrives, in order, one at a
// time. He never walks the message over himself - he hands it to the line
// and the line carries it. That is an `mpsc` channel: the sender drops a
// message in, the receiver pulls it out the far end, values flowing one
// way down the wire. Crypt coat, Teal flag faces (sea signal), no magic.
// Mission: mpsc_channel - pass values between threads over a channel.
//
// Legend:
//   X coalblack (outline, flag poles)
//   K inkblood (coat deep shadow)
//   Y crypt (coat deep)   O oxblood (coat primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   Z bronze (boots)   N antique brass (belt, flag finials)
//   G old gold (buckle)   L brass leaf (finial spec)
//   U bog umber (boot shadow)   F pine (hem)
//   D deep teal (flag shadow)   T main teal (flag face)   I bright teal (flag hi)
//   . transparent
//
// The two Teal flags (T/I) are the channel ends - the raised flag is the
// message being SENT, streaming frame-left toward the receiver down the
// line. One sender, one wire, values flowing one direction.

const REF56_GRID = [
  'L............................L..', //  0 flag finial specs (L) at pole tops
  'NDTI........................ITN.', //  1 raised flag (left) | lowered flag (right)
  'XDTTI.......XXXXXXXX.......TTDX.', //  2 flag face (T/I) | head top | flag
  'XDTTTI.....XSSSVVSSSX....ITTTDX.', //  3
  'XDTTTI....XSSVVVVVVSSX...ITTTDX.', //  4
  'XDTTI.....XSPPPPPPPPSX....ITTDX.', //  5
  'XDTI.PRP..XPPPPPPPPRRX.PRP.ITDX.', //  6 hands grip poles (P)
  'X.X.PPRP..XPBPPPPBPRWX.PRPP.X.X.', //  7 brow/eyes
  '....PPPP..XPPRRRRRRRPX.PPPP.....', //  8 forearms
  '.....XX..XXPRRRRRRPXX..XX.......', //  9 jaw
  '........XYPPPRRRPYX.............', // 10 collar
  '.......XYYYYYYYYYYYX............', // 11 shoulders
  '......XYYYOOOOOOOOYYX...........', // 12 coat upper
  '.....XYYOOOWWWWWOOOYYX..........', // 13 coat lit
  '.....XYOOOOOOOOOOOOOYX..........', // 14
  '.....XYOOOWWWWWWOOOOYX..........', // 15
  '.....XXNGGGGGGGGNXXXX...........', // 16 belt (N) buckle (G)
  '.....XYYOOOOOOOOYYKX............', // 17 coat lower
  '.....XYYOOOWWWWOOYYKX...........', // 18
  '....XKYYOOOOOOOOOYYKX...........', // 19
  '....XKYYOOOWWWWWOOYYKX..........', // 20
  '....XKKYOOOOOOOOOOYYKKX.........', // 21 widest
  '....XKYYFFFFFFFFFFFFYKX.........', // 22 hem
  '....XKFFFFFXXX..XXXFFFFKX.......', // 23 leg split
  '....XXZZZZXX....XXZZZZZX........', // 24 boots
  '.....XZUUZX......XZUUUZX........', // 25
  '.....XXXXXX......XXXXXXX........', // 26 soles
  '................................', // 27
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF56 = function REF56({ scale = 8 }) {
  return <PixelArt grid={REF56_GRID} scale={scale} title="REF-56 The Signaler" />;
};
window.REF56_GRID = REF56_GRID;
window.REF56_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','F','D','T','I'];
