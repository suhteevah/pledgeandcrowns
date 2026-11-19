// SPDX-License-Identifier: MIT
// REF-111 - The Checker's Gate, The Commons district landmark, 64x64.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/checkers_gate.png.
//
// The Borrow Checker's post (Act 2): a stone-grey gate, two pillars under
// an arched lintel, with a basalt keystone plaque that carries an
// ampersand, the borrow sign, in bright teal. A brass lantern with a teal
// flame hangs on each pillar. Teal is the Checker's colour; nothing here
// is red. Bottom-centre anchor. Solid at the two pillars only, so the
// player walks through the arch.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  N antique brass  G old gold
//         T main teal  I bright teal  M mist teal

const REF111_GRID = [
  '................................................................', //  0
  '................................................................', //  1
  '..........................XXXXXXXXXXXX..........................', //  2 keystone plaque
  '....XXXXXXXXXXXXXXXXXXXXXXXBBBBBBBBBBXXXXXXXXXXXXXXXXXXXXXXX....', //  3
  '....XCCVVVVVVVVVVVVVVVVVVVXBBBMIBBBBBXVVVVVVVVVVVVVVVVVVVVVX....', //  4
  '....XSSSSSSSSSSSSSSSSSSSSSXBBIBBIBBBBXSSSSSSSSSSSSSSSSSSSSSX....', //  5
  '....XXXXXXXXXXXXXXXXXXXXXXXBBBIIBBBBBXXXXXXXXXXXXXXXXXXXXXXX....', //  6 lintel
  '......XVVVVVVVVVVVVVVVVVVVXBBBIIBIBBBXVVVVVVVVVVVVVVVVVVVX......', //  7
  '......XSSSSSSSBSSSSSSSBSSSXBBIBBIBBBBXBSSSSSSSBSSSSSSSBSSX......', //  8
  '......XSSSSSSSBSSSSSSSBSSSXBBIBBIIBBBXBSSSSSSSBSSSSSSSBSSX......', //  9
  '......XSSSSSSSBSSSSSSSBSSSXBBBIIBBIBBXBSSSSSSSBSSSSSSSBSSX......', // 10
  '......XSSSSSSSBSSSSSSSBSSSXBBBBBBBBBBXBSSSSSSSBSSSSSSSBSSX......', // 11
  '......XBBBBBBBBBBBBBBBBBBBXXXXXXXXXXXXBBBBBBBBBBBBBBBBBBBX......', // 12
  '......XSSSBSSSSSSSBSSSSSSSXXXXXXXXXXXXSSSSBSSSSSSSBSSSSSSX......', // 13
  '......XSSSBSSSSSSSBSSSSXXX............XXXSBSSSSSSSBSSSSSSX......', // 14 arch
  '......XSSSBSSSSSSSBSSXX..................XXSSSSSSSBSSSSSSX......', // 15
  '......XBBBBBBBBBBBBBX......................XBBBBBBBBBBBBBX......', // 16
  '......XXXXXXXXXXXXXX........................XXXXXXXXXXXXXX......', // 17
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 18
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 19
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 20
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 21
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 22
  '........XVBBBBBBBBBX........................XVBBBBBBBBBX........', // 23
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 24
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 25
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 26
  '........XVSSXXXXSSBX........................XVSSXXXXSSBX........', // 27 lanterns
  '........XVSXNGGNXSBX........................XVSXNGGNXSBX........', // 28
  '........XVBXIMMIXBBX........................XVBXIMMIXBBX........', // 29
  '........XVSXTIITXSBX........................XVSXTIITXSBX........', // 30
  '........XVSXTIITXSBX........................XVSXTIITXSBX........', // 31
  '........XVSXNGGNXSBX........................XVSXNGGNXSBX........', // 32
  '........XVSSXXXXSSBX........................XVSSXXXXSSBX........', // 33
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 34
  '........XVBBBBBBBBBX........................XVBBBBBBBBBX........', // 35
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 36
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 37
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 38
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 39
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 40
  '........XVBBBBBBBBBX........................XVBBBBBBBBBX........', // 41
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 42
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 43
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 44
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 45
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 46
  '........XVBBBBBBBBBX........................XVBBBBBBBBBX........', // 47
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 48
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 49
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 50
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 51
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 52
  '........XVBBBBBBBBBX........................XVBBBBBBBBBX........', // 53
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 54
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 55
  '........XVSSSSSSSSBX........................XVSSSSSSSSBX........', // 56
  '......XXXXXXXXXXXXXXXX....................XXXXXXXXXXXXXXXX......', // 57 pillar bases
  '......XVVVVVVVVVVVVVVX....................XVVVVVVVVVVVVVVX......', // 58
  '......XSSSSSSSSSSSSSSX....................XSSSSSSSSSSSSSSX......', // 59
  '......XBBBBBBBBBBBBBBX....................XBBBBBBBBBBBBBBX......', // 60
  '......XXXXXXXXXXXXXXXX....................XXXXXXXXXXXXXXXX......', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF111_GRID = REF111_GRID;
