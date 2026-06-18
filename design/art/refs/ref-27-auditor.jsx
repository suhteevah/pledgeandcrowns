// REF-27 - The Auditor, 32x32, idle, transparent bg.
// Severe, upright. Crypt robe, hair scraped into a tight bun, a brass
// loupe to one eye, an open ledger held chest-high. One alarm-scarlet
// wax seal marks the first failing row - the propagated Err. Rows below
// it are blank (unread; the function already returned).
// Mission: result_question_mark - propagate the error with ?.
// Bible exception: ~0.6% Alarm scarlet (!), under the 1% cap, justified
// like the Heraldic Sage's cobalt sigil - the Auditor IS the error.

// Legend:
//   X coalblack (outline, tally rows, loupe arm)
//   Y crypt (robe deep)   O oxblood (robe primary)   W wineflesh (lit)
//   R dusty rose (skin)   P pink quartz (skin hi)     B basalt (eyes)
//   S stone grey (hair/bun)   V aged paper (collar hi)
//   N antique brass (loupe chain)   G old gold (loupe ring)   L brass leaf (lens glint)
//   C parchment cream (ledger pages)   ! alarm scarlet (the flagged Err seal)
//   U bog umber (boots)
//   . transparent

const REF27_GRID = [
  '................................', //  0
  '................................', //  1
  '...........XXXXXX...............', //  2 bun top
  '..........XSSSSSSX..............', //  3 hair bun
  '.........XXSSSSSSXX.............', //  4
  '.........XSSSSSSSSX.............', //  5 hairline
  '.........XPPPPPPPPX.............', //  6 forehead
  '.........XPBPPPBPPXGL...........', //  7 eyes + loupe ring (G) / glint (L)
  '.........XPPRRRRPPXN............', //  8 loupe chain (N)
  '.........XXPRRRRPXX.............', //  9 thin mouth
  '.........XXPRRRPXX..............', // 10 jaw
  '.........XYYYYYYYYX.............', // 11 high collar
  '........XYYYYVVYYYYX............', // 12 collar hi
  '.......XYYOOOOOOOOYYX...........', // 13 shoulders
  '......XYYOOOOOOOOOOYYX..........', // 14 robe upper
  '......XYOOOWWWWOOOOYOX..........', // 15
  '.....XPPPXCCCCCCCCXPPPX.........', // 16 hands hold ledger; pages (C)
  '.....XPRPXCXXXXXXCXPRPX.........', // 17 ledger top edge + tally header
  '.....XPPPXCXRRRRXCXPPPX.........', // 18 row 1 (read OK)
  '......XYOXCXRRRRXCXOYX..........', // 19 row 2 (read OK)
  '......XYOXC!XXXXXCXOYX..........', // 20 FLAGGED row - Err seal (!)
  '......XYOXCXBBBBXCXOYX..........', // 21 rows below blank/unread (B shadow)
  '......XYOXCXBBBBXCXOYX..........', // 22
  '......XYOXCCCCCCCCXOYX..........', // 23 ledger bottom edge
  '......XYYOOOOOOOOOOYYX..........', // 24 robe lower
  '......XYYOOWWWWWOOOYYX..........', // 25
  '......XYYOOOOOOOOOOYYX..........', // 26
  '......XYYOOOOOOOOOOYYX..........', // 27
  '......XYYYOOOOOOOOYYYX..........', // 28 hem
  '......XXZZZX....XZZZXX..........', // 29 boots
  '.......XUUZX....XZUUX...........', // 30
  '.......XXXXX....XXXXX...........', // 31 soles
];

window.REF27 = function REF27({ scale = 8 }) {
  return <PixelArt grid={REF27_GRID} scale={scale} title="REF-27 The Auditor" />;
};
window.REF27_GRID = REF27_GRID;
window.REF27_ROLES = ['X','Y','O','W','R','P','B','S','V','N','G','L','C','!','U'];
