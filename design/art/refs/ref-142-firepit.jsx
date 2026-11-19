// SPDX-License-Identifier: MIT
// REF-142 - Firepit with a spit, Act 5 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/firepit.png.
//
// A stone-ringed fire, gold-ramp flames over a parchment-cream bed
// with two alarm-scarlet tips, and a roast on an iron spit between
// two basalt posts. Bottom-centre anchor; solid over the stone ring.
//
// Legend: X coalblack  B basalt  S stone grey  V aged paper
//         N antique brass  G old gold  L brass leaf  Z bronze
//         C parchment cream  ! alarm scarlet

const REF142_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '...XXX....................XXX...', //  4
  '...XSX....................XSX...', //  5
  '...XSX......XXXXXXXX......XSX...', //  6 roast on the spit
  '...XSX.....XNGGNNGGNX.....XSX...', //  7
  '...XXXXXXXXXZNNZZNNZXXXXXXXXX...', //  8
  '...XSSSSSSSXZZNZZNZZXSSSSSSSX...', //  9
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', // 10
  '...XSX....................XSX...', // 11
  '...XSX....................XSX...', // 12
  '...XSX....................XSX...', // 13
  '...XSX....................XSX...', // 14 flames
  '...XSX........!...........XSX...', // 15
  '...XSX........G.!.........XSX...', // 16
  '...XSX......N.GNG.........XSX...', // 17
  '...XSX......GNLGG.N.......XSX...', // 18
  '...XSX....N.GGLGLNG.......XSX...', // 19
  '...XSX....GNLGLLLGG.N.....XSX...', // 20
  '...XSX...NGGLLLLLGLNGX....XSX...', // 21 stone ring
  '...XSX.XXGLGLLLLLLLGGNSXX.XSX...', // 22
  '...XSXXVNGLLLLLLLLLGLGNSSXXSX...', // 23
  '...XXXVXGLLLCCCCCCCCLGGXXSXXX...', // 24
  '...XXXVXXXXXXXXXXXXXXXXXXBXXX...', // 25
  '...XXXVSSSSSSSSSSSSSSBBBBBXXX...', // 26
  '.....XXSSSSSSSSSSSBBBBBBBXX.....', // 27
  '.......XXSSSSSSBBBBBBBBXX.......', // 28
  '..........XXXXXXXXXXXX..........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF142_GRID = REF142_GRID;
