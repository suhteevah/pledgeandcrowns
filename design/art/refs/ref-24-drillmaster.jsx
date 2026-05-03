// REF-24 — The Drillmaster, 32×32, idle, transparent bg.
// Square-shouldered captain in Forest tunic with gold chest-stripe.
// Right arm clamps a Crypt baton at the side; left hand raised palm-out
// with three fingers up — counting cadence. Slate at the right hip with
// three chalk tally marks. Mission: for_in_range.

// Legend:
//   X coalblack (outline, slate face, chalk frame)
//   J mossbed (tunic deep shadow)
//   F pine (tunic shadow)         E forest (tunic primary)
//   H spring meadow (tunic lit edge — left shoulder)
//   K inkblood (baton shadow)     Y crypt (baton primary)
//   R dusty rose (skin mid)       P pink quartz (skin hi)
//   W wineflesh (skin shadow, mouth)
//   B basalt (eyes, brow line)
//   S stone grey (hair, slate frame, chalk dust)
//   V aged paper (chalk tallies)
//   Z bronze (trousers, moustache)
//   N antique brass (epaulettes, slate strap, hair shadow)
//   G old gold (chest stripe, baton end-cap)
//   L brass leaf (baton pip specular)
//   U bog umber (boots)
//   . transparent

const REF24_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSSSSSSSX.............', //  3 hair (close-cropped)
  '........XSSNNNNNSSX.............', //  4
  '........XPPPPPPPPRRX............', //  5 forehead
  '........XBPPRRRRRWX.............', //  6 brow
  '........XX.X.XX.X.XX............', //  7 eyes
  '........X.PPRRRRRRWX............', //  8
  '........XXZZZZZZZZXX............', //  9 moustache row
  '........X.PPRWWWWPX.............', // 10 mouth
  '.........XPPRRRRRPX.............', // 11 chin
  '........XEEEEEEEEEEX............', // 12 collar
  '.....XXXXEEHEEEEEEEXXXX.........', // 13 shoulders + H lit edge left
  '....XPRRXNEEEEEEEEEXKYYX........', // 14 left arm raised palm-out + baton tucked right
  '....XPPPXEEGGGGGEEEXKYYX........', // 15 chest stripe G
  '....XPPRXEEEEEEEEEEXYYYX........', // 16
  '....XPPPXEEEEEEEEEEXYYLX........', // 17 baton specular pip L
  '....XPRRXEEEEEEEEEEXYYGX........', // 18 baton end-cap G
  '....XPPPXEEFFFFFFEEXKYYX........', // 19
  '....XXXXXEEFFFFFFEEXXXXX........', // 20 arms close
  '........XEEFFFFFFEEX............', // 21 lower tunic
  '........XEEFFFFFFEEX............', // 22
  '........XJJJJJJJJJJX...XXXX.....', // 23 tunic deep shadow + slate begins
  '........XZZZZZZZZZZX..XSXXSXX...', // 24 trousers + slate frame
  '........XZZZZZZZZZZX..XNXVXNX...', // 25 slate face X w/ V tally column 1
  '........XZZZZZZZZZZX..XNXVXNX...', // 26 tally column 2 V
  '........XZZZZZZZZZZX..XNXVXNX...', // 27 tally column 3 V
  '........XZZZZXXZZZZX..XSSSSSX...', // 28 leg split + slate base
  '........XUUUUX..XUUUUX..........', // 29 boot tops (wider)
  '........XUUUUX..XUUUUX..........', // 30 boots
  '........XXXXXX..XXXXXX..........', // 31 soles
];

window.REF24 = function REF24({ scale = 8 }) {
  return <PixelArt grid={REF24_GRID} scale={scale} title="REF-24 The Drillmaster" />;
};
window.REF24_GRID = REF24_GRID;
window.REF24_ROLES = ['X','J','F','E','H','K','Y','R','P','W','B','S','V','Z','N','G','L','U'];
