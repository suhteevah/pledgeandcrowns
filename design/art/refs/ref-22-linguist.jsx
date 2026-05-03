// REF-22 — The Linguist, 32×32, idle, transparent bg.
// Slim scholar in a Crypt robe with parchment undercollar. Open book in
// the LEFT hand (owned String), rolled scroll in the RIGHT hand
// (borrowed &str literal). One brass-leaf speech curl above the lips.
// Mission: string_vs_str — one tongue, two voices.

// Legend:
//   X coalblack (outline, book/scroll borders)
//   K inkblood (robe deep shadow)
//   Y crypt (robe primary)        O oxblood (robe lit row)
//   W wineflesh (robe lit edge, mouth)
//   R dusty rose (skin mid)       P pink quartz (skin hi)
//   B basalt (eyes shadow line)
//   S stone grey (hair primary, scroll shadow)
//   V aged paper (hair hi, sash)
//   Z bronze (scroll end-caps, book binding shadow)
//   N antique brass (scroll cap shadow, sash mid)
//   G old gold (book clasp)
//   L brass leaf (speech curl pixel)
//   U bog umber (book binding spine)
//   C parchment cream (book pages, scroll body)
//   . transparent

const REF22_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSSVVSSSX.............', //  3 hair w/ pale top
  '........XSSVVVVVVSSX............', //  4
  '........XSSVVVVVVVSX............', //  5
  '........XPPPPPPPPRRX............', //  6 forehead
  '........XBPPRRRRRRWX............', //  7 brow line
  '........XX.X.XX.X.XX............', //  8 eyes
  '........X.PPRRRRRRWX............', //  9
  '........XXPPRWWWWPXL............', // 10 mouth + speech curl pixel L
  '........XXPPPRRRRPXX............', // 11 chin
  '.........XYYYYYYYYX.............', // 12 collar
  '........XYYVVVVVYYYX............', // 13 robe upper + undertunic V
  '........XYYOOOOOOOYX............', // 14 robe lit row
  '.....XXXXXYOOOOOOYXXXXX.........', // 15 arms outstretched begin
  '....XUCCCCXYYYYYYYXCCCCXX.......', // 16 left book opens / right scroll begins
  '....XUCGCCXYYYYYYYXSCCCNX.......', // 17 book clasp G + page line, scroll cap N
  '....XUCCCCXYYYYYYYXSCCCNX.......', // 18
  '....XUCCCCXYYYYYYYXSCCCZX.......', // 19 scroll shadow Z at right cap
  '....XUCCCCXYYYYYYYXSCCCZX.......', // 20
  '....XXXXXXXYYYYYYYXXXXXXX.......', // 21 book/scroll close
  '........XYYYYYYYYYYX............', // 22 robe mid
  '........XYYYOOOOOYYX............', // 23
  '........XVVNNNNNNNVX............', // 24 sash V/N
  '........XYYYOOOOOYYX............', // 25 robe lower
  '........XYYYYYYYYYYX............', // 26
  '........XYYYYOOOYYYX............', // 27
  '........XYYYYYYYYYYX............', // 28 hem
  '........XZZZXXXXZZZX............', // 29 boot tops
  '........XZUUX..XZUUX............', // 30 boot mid
  '........XXXXX..XXXXX............', // 31 soles
];

window.REF22 = function REF22({ scale = 8 }) {
  return <PixelArt grid={REF22_GRID} scale={scale} title="REF-22 The Linguist" />;
};
window.REF22_GRID = REF22_GRID;
window.REF22_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','C'];
