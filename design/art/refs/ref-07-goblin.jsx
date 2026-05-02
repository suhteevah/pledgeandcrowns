// REF-07 — Generic goblin enemy, 32×32, idle, looking right.
// Forest skin (Forest #487E40 / Pine #27502E shadows). Brown leather scraps,
// rusty short sword. Mischievous not evil.

// Legend:
//   X coalblack outline   F pine (skin shadow)  E forest (skin mid)
//   H spring meadow (skin hi)  Q hayfield (rim)
//   Z bronze (leather)  N antique brass (leather hi)  U bog umber (deep)
//   S stone grey (sword blade hi)  B basalt (blade core)
//   Y crypt (rust)  O oxblood (rust accent)
//   K inkblood (deep shadow)  P pink quartz (eye glint)

const REF07_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '..........XXXXXXXX..............', //  4 head top
  '.........XEEEEEEEFX.............', //  5
  '........XEEHHHEEEFFX............', //  6 hi on top-left of head
  '........XEHEEEEEEFFX............', //  7
  '.......XXEEXXEEXXEEFX...........', //  8 brow line w/ eye sockets
  '.......XEEFXXEFXXEFFX...........', //  9 ears form w/ eyes
  '......XEEEEEEEEEEEEFX...........', // 10
  '......XEEEEXX!!XXEEFX...........', // 11 nose w/ red glint
  '......XEEFXEEEEEXEFFX...........', // 12 mouth shadow
  '......XEEEXKKKKKEEFFX...........', // 13 teeth gap
  '......XEEEXC#C#CEEFFX...........', // 14 yellow teeth
  '......XEEFXXXXXXEEFFX...........', // 15
  '......XEEEEEEEEEEFFXX...........', // 16 chin
  '.......XXEEEEEEEEFXX............', // 17 neck
  '.......XZZZZZZZZZZX.............', // 18 leather collar
  '......XZNNZNNZNNZZX.............', // 19 leather scraps
  '.....XEEZZNNNNZZEEFX............', // 20 shoulders + skin
  '.....XEEZHHEEEHHZEFFX...........', // 21 chest/arms
  '.....XEFZZZZZZZZZEFFX...........', // 22
  '.....XEFZNZNZNZNZEFFXSSXX.......', // 23 left hand starts gripping sword
  '.....XEEEEEFEEEEFFFFFXSBSX......', // 24 sword blade upper
  '.....XEEXXXEEEEEFFFFXSBBSX......', // 25 hand grip + blade
  '......XXEEXEEEEEFFFXBBSSX.......', // 26
  '......XEEEXEEEFFFFFXSSSXX.......', // 27 belt line
  '......XZZZZZZZZZZZZZZX..........', // 28 belt
  '......XEEEFEEEFEEEFFFX..........', // 29 legs
  '......XEEFXXEEFXXEEFFX..........', // 30 leg split
  '......XKKXX.XKKXX.XKKX..........', // 31 feet
];

window.REF07 = function REF07({ scale = 8 }) {
  return <PixelArt grid={REF07_GRID} scale={scale} title="REF-07 Goblin enemy" />;
};
window.REF07_GRID = REF07_GRID;
window.REF07_ROLES = ['X','F','E','H','Q','Z','N','U','S','B','Y','O','K','P','!','C','#'];
