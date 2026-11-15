// SPDX-License-Identifier: MIT
// REF-49 v2 - The Sifter on the Forge kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: brass goggles, basalt apron, bare arms (REF-72), with a stone-grey
// bandana in place of the leather cap. Lesson prop (iter_filter): a wide
// round sieve held level in both hands across the chest, broader than the
// shoulders, with a few old-gold grains falling through below it - only what
// passes the predicate gets past.
//
// Legend: X coalblack  B basalt  S stone grey  V aged paper
//         P pink quartz  R dusty rose  W wineflesh
//         N antique brass  Z bronze  U bog umber  G old gold  L brass leaf

const REF49_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXXXXXX...........', //  2 bandana
  '..........XSSVVSSSSSSX..........', //  3
  '..........XSVSSSSSSSBX..........', //  4
  '.........XXNNNNNNNNNNXX.........', //  5 goggle strap
  '.........XNXLGXNNXLGXNX.........', //  6 lenses
  '.........XXNXXXNNXXXNXX.........', //  7
  '..........XPPPPRPPPRWX..........', //  8
  '..........XPPPRRRRPRWX..........', //  9
  '...........XPRRRRRRWX...........', // 10
  '.......XXXXXXBBBBBBXXXXXX.......', // 11
  '.....XXPPPRXBBSBBBBBBXPPRRXX....', // 12 bare shoulders
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 13 sieve rim
  '...XPXZNZNZNZNZNZNZNZNZNZNXPX...', // 14 mesh, hands
  '...XRXXXXXXXXXXXXXXXXXXXXXXRX...', // 15
  '....XXX.G..XBBBBBBBBBX.G..XXX...', // 16 grains falling
  '.........G.XBSBBBBBBBX..G.......', // 17
  '...........XBSBBBBBBBX..........', // 18
  '........G..XNNNNGNNNNX.G........', // 19 belt
  '...........XBBBBBBBBBX..........', // 20
  '...........XBBBBBBBBBX..........', // 21
  '...........XBBBBBBBBBX..........', // 22
  '...........XXXXXXXXXXX..........', // 23 apron hem
  '...........XZZZZXZZZZX..........', // 24
  '...........XZZZX.XZZZX..........', // 25
  '...........XNZZX.XNZZX..........', // 26
  '...........XUUUX.XUUUX..........', // 27
  '..........XUUUUX.XUUUUX.........', // 28
  '..........XXXXXX.XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF49_V2_GRID = REF49_V2_GRID;
