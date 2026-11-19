// SPDX-License-Identifier: MIT
// REF-150 - Rowboat, Act 7 district prop, 32x32.
// DRAFT 2026-11-18. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// A bronze clinker rowboat seen from the side with a gold gunwale, two
// thwarts and an oar shipped across it, riding on a bright-teal
// waterline. Meant for the water tiles beside the dock. Bottom-centre
// anchor; solid over the hull.
//
// Legend: X coalblack  Z bronze  N antique brass  G old gold
//         U bog umber  I bright teal  M mist teal

const REF150_GRID = [
  '................................', //  0
  '...................XXXXX........', //  1 oar
  '...................XGNNX........', //  2
  '..................XXNNNX........', //  3
  '................XXNXNNNX........', //  4
  '..............XXNN.XXXXX........', //  5
  '............XXNN................', //  6
  '..........XXNN..................', //  7
  '........XXNN....................', //  8
  '......XXNN......................', //  9
  '......NN........................', // 10
  '.XGGGGGGGGGGGGGGGGGGGGGGGGGGGGX.', // 11 gunwale
  '.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.', // 12
  '....XXXXXNNNXXXXXXXXNNNXXXXX....', // 13
  '.XXNUUUUUNNNUUUUUUUUNNNUUUUUNXX.', // 14 hull
  '.XXZUUUUUNNNUUUUUUUUNNNUUUUUZXX.', // 15
  '.XXUUUUUUNNNUUUUUUUUNNNUUUUUUXX.', // 16
  '.XXZZZZZZZZZZZZZZZZZZZZZZZZZZXX.', // 17
  '..XZZZZZZZZZZZZZZZZZZZZZZZZZZX..', // 18
  '..XXZZZZZZZZZZZZZZZZZZZZZZZZXX..', // 19
  '...XXUUUUUUUUUUUUUUUUUUUUUUXX...', // 20
  '...XXZZZZZZZZZZZZZZZZZZZZZZXX...', // 21
  '....XXZZZZZZZZZZZZZZZZZZZZXX....', // 22
  '.....XXXZZZZZZZZZZZZZZZZXXX.....', // 23
  '......XXXUUUUUUUUUUUUUUXXX......', // 24
  '........XXXXZZZZZZZZXXXX........', // 25
  '..........XXXXXXXXXXXX..........', // 26
  '..MMM.....................MMMM..', // 27
  'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII', // 28 waterline
  '...IIIIIIIIII.....IIIIIIIIIII...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF150_GRID = REF150_GRID;
