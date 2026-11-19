// SPDX-License-Identifier: MIT
// REF-147 - The harbour crane, Act 7 district landmark, 64x64.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A bronze timber crane on a stone quay: a braced mast with iron
// bands, a jib rising to the right with a stay rope, a winch, and a
// crate hanging from the hook. Many hands, one load: Act 7 is
// concurrency. Bottom-centre anchor (under the quay); solid over the
// quay and mast, not under the hanging crate.
//
// Legend: X coalblack  Z bronze  N antique brass  G old gold
//         U bog umber  S stone grey  V aged paper  B basalt

const REF147_GRID = [
  '................................................................', //  0
  '................................................................', //  1
  '.....................XXXXXX.....................................', //  2 mast head
  '.....................XBBBBX...........................V.........', //  3
  '.....................XBSBBX..............VVVVVVVVVVVVVN.........', //  4
  '.....................XBBBBX.VVVVVVVVVVVVV.......XXXNNNZ.........', //  5 jib
  '....................XXBBBBXX................XXXXNNNZZZZ.........', //  6
  '....................XXXXXXXX.............XXXNNNNZZZZZZX.........', //  7
  '....................XNNZZZUX..........XXXNNNZZZZZZZXXV..........', //  8
  '....................XNNZZZUX......XXXXNNNZZZZZZZXXX..V..........', //  9
  '....................XNNZZZUX...XXXNNNNZZZZZZXXXX.....V..........', // 10
  '....................XNNZZZUXXXXNNNZZZZZZZXXX.........V..........', // 11
  '....................XNNZXXXXNNNZZZZZZZXXX............V..........', // 12
  '....................XNNZNNNNZZZZZZXXXX...............V..........', // 13
  '....................XNNZZZZZZZZXXX...................V..........', // 14
  '....................XNNZZZZZXXX......................V..........', // 15
  '....................XBBBXXXX.........................V..........', // 16
  '....................XNNZZZUX.........................V..........', // 17
  '....................XNNZZZUX.........................V..........', // 18
  '....................XNNZZZUX.........................V..........', // 19
  '....................XNNZZZUX.........................V..........', // 20
  '....................XNNZZZUX.........................V..........', // 21
  '....................XNNZZZUX.........................V..........', // 22
  '....................XNNZZZUX.........................V..........', // 23
  '....................XNNZZZUX.........................V..........', // 24
  '....................XNNZZZUX.........................V..........', // 25
  '....................XNNZZZUX.........................V..........', // 26
  '....................XNNZZZUX.........................V..........', // 27
  '....................XNNZZZUX.........................V..........', // 28
  '....................XNNZZZUX.........................V..........', // 29
  '....................XBBBBBBX........................XXX.........', // 30 hook
  '....................XNNZZZUX.......................VXSXV........', // 31
  '....................XNNZZZUX......................V.XSX.V.......', // 32
  '....................XNNZZZUX.....................V.XX.XX.V......', // 33
  '....................XNNZZZUX.....................V.XS..X.V......', // 34
  '....................XNNZZZUX.....................V.XSSSX.V......', // 35
  '....................XNNZZZUX.....................V..XXX..V......', // 36
  '....................XNNZZZUX..................XXXXXXXXXXXXXXX...', // 37 hanging crate
  '....................XNNZZZUX..................XGGGGGGGGGGGGGX...', // 38
  '....................XNNZZZUX..................XZZNNNNNNNNNNZX...', // 39
  '..................XZZNNZZZUZZX................XZNZNNNNNNNNNZX...', // 40
  '.................XZZXNNZZZUXZZX...............XZNNZNNNNNNNNZX...', // 41
  '................XZZ.XNNZZZUX.ZZX..............XZNNNZNNNNNNNZX...', // 42
  '............XXXXXZ..XNNZZZUX..ZZX.............XZNNNNZNNNNNNZX...', // 43 winch
  '...........XSSSBBX..XBBBBBBX...ZZX............XZNNNNNZNNNNNZX...', // 44
  '..........XSSSBBBBX.XNNZZZUX....ZZX...........XZNNNNNNZNNNNZX...', // 45
  '..........XSSBBZZZZZXNNZZZUX.....ZZX..........XZNNNNNNNZNNNZX...', // 46
  '..........XSBBGBBXX.XNNZZZUX......ZZX.........XZNNNNNNNNZNNZX...', // 47
  '..........XBBBBBXXX.XNNZZZUX.......ZZX........XZNNNNNNNNNZNZX...', // 48
  '.........XXBBBBXXXX.XNNZZZUX........ZZX.......XZZZZZZZZZZZZZX...', // 49
  '........XZZXBBXXXX..XNNZZZUX.........ZZX......XXXXXXXXXXXXXXX...', // 50
  '.......XZZ..XXXXX...XNNZZZUX..........ZZX.......................', // 51
  '........XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX....................', // 52 stone quay
  '........XVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVX....................', // 53
  '........XSSSSSSSBSSSSSSSSSBSSSSSSSSSBSSSSSSX....................', // 54
  '........XSSSSSSSBSSSSSSSSSBSSSSSSSSSBSSSSSSX....................', // 55
  '........XSSSSSSSBSSSSSSSSSBSSSSSSSSSBSSSSSSX....................', // 56
  '........XBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBX....................', // 57
  '........XSSSBSSSSSSSSBSSSSSSSSSBSSSSSSSSBSSX....................', // 58
  '........XSSSBSSSSSSSSBSSSSSSSSSBSSSSSSSSBSSX....................', // 59
  '........XSSSBSSSSSSSSBSSSSSSSSSBSSSSSSSSBSSX....................', // 60
  '........XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX....................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF147_GRID = REF147_GRID;
