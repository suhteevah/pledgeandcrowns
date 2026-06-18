// SPDX-License-Identifier: MIT
// REF-40 - The Lanternkeeper, 32x32, idle, transparent bg.
// Holds the Lifetime Lantern aloft on a raised arm: an Old-gold lantern,
// Coalblack-framed, with a small STEADY Mage-glow flame inside kept alive.
// The flame must outlive the hand that carries it - a borrow valid for
// 'a lives as long as the keeper keeps it lit. Crypt robe; the single
// contained flame is the one magic accent (well under 5%), glassed in
// gold so it never escapes its lifetime.
// Mission: lifetimes - a borrow kept alive for 'a.
//
// Legend:
//   X coalblack (outline, lantern frame, chain)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   N antique brass (lantern cap/base)   Z bronze (lantern ring, boots)   G old gold (lantern body)
//   % mage glow (contained flame)   * royal arcane (flame base shadow)
//   U bog umber (boots)   F pine (hem)
//   . transparent

const REF40_GRID = [
  '...............NN...............', //  0 lantern hang ring
  '...............XX...............', //  1 chain link
  '..............XGGX..............', //  2 lantern cap (G)
  '.............XNGGNX.............', //  3 cap brass (N)
  '.............XG..GX.............', //  4 lantern shoulder
  '...XXXXXXXX...XG%%GX............', //  5 head top | flame top (%)
  '..XSSSVVSSSX..XG%%GX............', //  6 hair | flame
  '.XSSVVVVVVSSX.XG*%*GX...........', //  7 | flame base (* shadow)
  '.XSPPPPPPPPSX.XNGGGNX...........', //  8 forehead | lantern base (N)
  '.XPPPPPPPPRRX..XXXX.............', //  9 | lantern bottom
  '.XPBPPPPBPRWX..PPP..............', // 10 brow/eyes | raised hand
  '.XPPRRRRRRRPX.PRRP..............', // 11 | hand cups handle (P/R)
  '.XXPRRRRRRPXX.PPPP..............', // 12 jaw | wrist
  '.XYPPPRRRRPYXX..................', // 13 collar
  '.XYYYYYYYYYYYX..................', // 14 shoulders
  '.XYYYOOOOOOOOYYYX...............', // 15 robe upper
  '.XYYOOOWWWWOOOOYOX..............', // 16 robe lit
  '.XYOOOWWWWWWOOOOYOX.............', // 17
  '.XYOOOOOOOOOOOOOYOX.............', // 18
  '.XYOOOWWWWWWOOOOYOX.............', // 19
  '.XXNGGGGGGGGGGGNXXX.............', // 20 belt band (N/G)
  '.XYYOOOOOOOOOOYYX...............', // 21 robe lower
  '.XYYOOOWWWWWOOYYX...............', // 22
  '.XYYOOOOOOOOOOYYX...............', // 23
  '.XYYOOOOOOOOOOYYX...............', // 24
  '.XYFFFFFFFFFFFFYX...............', // 25 hem
  '.XFFFFFXX..XXFFFFX..............', // 26 leg split
  '.XZZZZX....XZZZZX...............', // 27 boots
  '.XZUUZX....XZUUZX...............', // 28
  '.XXXXXX....XXXXXX...............', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF40 = function REF40({ scale = 8 }) {
  return <PixelArt grid={REF40_GRID} scale={scale} title="REF-40 The Lanternkeeper" />;
};
window.REF40_GRID = REF40_GRID;
window.REF40_ROLES = ['X','Y','O','W','R','P','B','S','V','N','Z','G','%','*','U','F'];
