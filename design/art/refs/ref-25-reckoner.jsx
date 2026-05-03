// REF-25 — The Reckoner, 32×32, idle, transparent bg.
// Bowed-head clerk in a Crypt robe over a desk that fills the lower
// third of frame. Right hand grips a quill mid-stroke (specular L on
// the nib). Left hand rests on the open ledger pages.
// Mission: closure_basic — inks the ledger, summed in a single stroke.

// Legend:
//   X coalblack (outline, quill, tally lines, sum line)
//   K inkblood (robe deep shadow)
//   Y crypt (robe primary)        O oxblood (robe lit row)
//   W wineflesh (robe lit edge)
//   R dusty rose (skin mid)       P pink quartz (skin hi, hand hi)
//   B basalt (brow line, eyes)
//   S stone grey (hair primary)
//   V aged paper (hair hi, undercollar hi, left page shadow row)
//   Z bronze (desk trim)          N antique brass (earring)
//   G old gold (ledger ribbon bookmark)
//   L brass leaf (quill nib specular)
//   U bog umber (desk top)
//   C parchment cream (ledger pages)
//   . transparent

const REF25_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '..........XXXXXXXX..............', //  3 head top (bowed forward, lower in frame)
  '.........XSSSVVSSSX.............', //  4 hair w/ V hi
  '........XSSVVVVVVSSX............', //  5
  '........XSSVVVVVVVSX............', //  6
  '........XPPPPPPPPRRX............', //  7 forehead (only forehead/brow visible)
  '........XBPPRRRRRRWXN...........', //  8 brow line + earring N right
  '........XX.X.XX.X.XX............', //  9 eyes (downcast, masked by bow)
  '........XXPPRRRRRRPXX...........', // 10 nose bridge fragment
  '........XYYYYYYYYYYYX...........', // 11 collar drops to shoulders
  '......XXYYYYYYYYYYYYYXX.........', // 12 robe shoulders
  '.....XYYYYOOOOOOOOOYYYYX........', // 13 robe upper lit row
  '.....XYYYOOOWWWOOOOOYYYX........', // 14
  '.....XYYYYYYYYYYYYYYYYYX........', // 15 robe mid
  '.....XYYYYYYYYYYYYYYYYYX........', // 16
  '....XPPPYYYYYYYYYYYYYYYX........', // 17 left hand emerges onto ledger (left side)
  '....XPRPYYYYYYYYYYYYPPPX........', // 18 right hand emerges with quill
  '....XPPPYYYYYYYYYYYYPRPXX.......', // 19 quill begins right
  '....XPPPYYYYYYYYYYYYPPPXLX......', // 20 quill nib specular L
  '....XCCCCCCCXCCCCCCCCXX.X.......', // 21 ledger top edge — left page CCCCC | right page CCCC
  '....XCXXVCCCXCCCCCCCCCX.........', // 22 left page tally row 1 (XX) + V shadow
  '....XCXXVCCCXCCCCCCCCCX.........', // 23 left page tally row 2
  '....XCXXVCCCXCCCXXXXXCX.........', // 24 left tally 3 + right page sum line XXXXX
  '....XCCCCGCCXCCCCCCCCCX.........', // 25 ribbon bookmark G left page
  '....XUUUUUUUUUUUUUUUUUUX........', // 26 desk top U
  '....XZZZZZZZZZZZZZZZZZZX........', // 27 desk trim Z
  '....XUUUUUUUUUUUUUUUUUUX........', // 28 desk underside
  '....XUUUUUUUUUUUUUUUUUUX........', // 29
  '....XBBBBXX........XXBBBBX......', // 30 desk legs (basalt)
  '....XBBBBXX........XXBBBBX......', // 31
];

window.REF25 = function REF25({ scale = 8 }) {
  return <PixelArt grid={REF25_GRID} scale={scale} title="REF-25 The Reckoner" />;
};
window.REF25_GRID = REF25_GRID;
window.REF25_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','U','C'];
