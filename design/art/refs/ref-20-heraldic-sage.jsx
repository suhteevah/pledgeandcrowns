// REF-20 — The Heraldic Sage, 32×32, idle, transparent bg.
// First Act-2 NPC. Long burgundy robe, gold sash with FOUR sigil panels
// (each panel = an enum variant). Mission: enum_match. Foreshadowing
// allowance: one cobalt sigil (≤1% canvas) flags Act 6's cool zone.

// Legend:
//   X coalblack (outline, sigil borders)
//   K inkblood (robe deep shadow)
//   Y crypt (robe primary)        O oxblood (robe lit row)
//   W wineflesh (robe lit edge, mouth)
//   R dusty rose (skin mid)       P pink quartz (skin hi)
//   B basalt (eyes shadow line)
//   S stone grey (hair primary, beard hi)
//   V aged paper (hair hi)
//   Z bronze (beard primary)      N antique brass (beard shadow)
//   G old gold (sash primary, sun sigil)
//   L brass leaf (sash hi, hem stitch)
//   E forest (oak sigil)
//   T main teal (river sigil)     I bright teal (river sigil hi)
//   < cobalt deep (azure sigil shadow)
//   > cobalt (azure sigil primary)  -- foreshadowing allowance, ~6 px
//   . transparent

const REF20_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSVVVVSSX.............', //  3 silver hair
  '........XSVVVVVVVVSX............', //  4
  '........XSVVVVVVVVSX............', //  5
  '........XPPPPPPPPRRX............', //  6 forehead
  '........XBPPRRRRRRWX............', //  7 brow
  '........XX.X.XX.X.XX............', //  8 eyes
  '........X.PPRRRRRRWX............', //  9
  '........XZZNRRRRRWZX............', // 10 beard top sides + cheek
  '........XZNNNRRRRZZX............', // 11 beard mid
  '........XXZNNNNNZZXX............', // 12 beard tip
  '.......XYYYYYYYYYYYYX...........', // 13 robe collar
  '......XYYYOOOOOOOYYYX...........', // 14 robe upper lit row band
  '......XYYOXEEXTTXGGXLYX.........', // 15 sash lit row + sigil borders begin
  '......XLGGXEEXTIXGGXLLX.........', // 16 sash + sigils row 1: oak, river, sun
  '......XLGGXEEXTTXGGXLLX.........', // 17 sash + sigils row 2
  '......XYYYXXXXXXXXXXLYX.........', // 18 sash bottom edge + sigil borders close
  '......XYYYYOOOOOYYYYYX..........', // 19 robe under sash
  '......XYYYYYYYYYYYYYYX..........', // 20
  '......XYYYYYYYYYYYYYYX..........', // 21 plus 4th sigil (azure) appended below sash on right
  '......XYYYKKYYX<>>XYYYX.........', // 22 azure (cobalt) sigil — foreshadowing patch
  '......XYYKKKKYX<>>XYYYX.........', // 23
  '......XYYKKKKYXXXXYYYYX.........', // 24
  '......XYKKKKKKYYYYYYYYX.........', // 25 robe widens
  '......XYKKKKKKKKKKYYYYX.........', // 26
  '......XYKKKKKKKKKKKYYYX.........', // 27
  '......XYKKKKKKKKKKKKYYX.........', // 28
  '......XLLLLLLLLLLLLLLLX.........', // 29 brass-leaf hem stitch
  '......XYYYKKKKKKKKKYYYX.........', // 30 hem trail
  '......XXXXXXXXXXXXXXXXX.........', // 31
];

window.REF20 = function REF20({ scale = 8 }) {
  return <PixelArt grid={REF20_GRID} scale={scale} title="REF-20 The Heraldic Sage" />;
};
window.REF20_GRID = REF20_GRID;
window.REF20_ROLES = ['X','K','Y','O','W','R','P','B','S','V','Z','N','G','L','E','T','I','<','>'];
