// SPDX-License-Identifier: MIT
// REF-98 - The Great Furnace, Act 6 district landmark, 64x64.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/great_furnace.png.
//
// The Iterator Forge's landmark: a stone furnace under an iron hood and a
// brick chimney, a brass band with rivets across its brow, a brass hammer
// crest on the hood, and an arched fire mouth with a bronze rim. The fire
// is the only warm light in the district: gold ramp flames over a
// parchment-cream bed, three alarm-scarlet tips (3 px, far under the 1%
// cap), and a brass glow on the plinth edge below the mouth.
// Bottom-centre anchor. Solid from the plinth to the hood.
//
// Legend: X coalblack  B basalt  S stone grey  V aged paper
//         Z bronze  N antique brass  G old gold  L brass leaf
//         C parchment cream  ! alarm scarlet

const REF98_GRID = [
  '........................XXXXXXXXXXXXXXXX........................', //  0 chimney cap
  '........................XVSSSSSSSSSSSSSX........................', //  1
  '........................XXXXXXXXXXXXXXXX........................', //  2
  '..........................XXXXXXXXXXXX..........................', //  3 chimney shaft
  '..........................XVSSSSBSSSBX..........................', //  4
  '..........................XVSSSSBSSSBX..........................', //  5
  '..........................XVSSSSBSSSBX..........................', //  6
  '..........................XVBBBBBBBBBX..........................', //  7
  '..........................XVSSBSSSBSBX..........................', //  8
  '..........................XVSSBSSSBSBX..........................', //  9
  '..........................XVSSBSSSBSBX..........................', // 10
  '..........................XVBBBBBBBBBX..........................', // 11
  '..........................XVSSSSBSSSBX..........................', // 12
  '..........................XVSSSSBSSSBX..........................', // 13
  '..........................XVSSSSBSSSBX..........................', // 14
  '..........................XVBBBBBBBBBX..........................', // 15
  '..........................XVSSBSSSBSBX..........................', // 16
  '..........................XVSSBSSSBSBX..........................', // 17
  '..........................XVSSBSSSBSBX..........................', // 18
  '..........................XXXXXXXXXXXX..........................', // 19
  '......................XXXXSSBBBBBBBBBBXXXX......................', // 20 iron hood
  '....................XXSSBBBBBBBBBBBBBBBBBBXX....................', // 21
  '..................XXSSBBBBBBBLLGGGGBBBBBBBBBXX..................', // 22 brass hammer crest
  '................XXSSBBBBBBBBBGGGGGNBBBBBBBBBBBXX................', // 23
  '..............XXSSBBBBBBBBBBBBBNNBBBBBBBBBBBBBBBXX..............', // 24
  '............XXSSBBBBBBBBBBBBBBBNNBBBBBBBBBBBBBBBBBXX............', // 25
  '..........XXSSBBBBBBBBBBBBBBBBBNNBBBBBBBBBBBBBBBBBBBXX..........', // 26
  '..........XSBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBX..........', // 27
  '..........XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..........', // 28 brass band, rivets
  '..........XLLGGZGGGGGZGGGGGZGGGGGZGGGGGZGGGGGZGGGGNNNX..........', // 29
  '..........XNNNNLNNNNNLNNNNNLNNNNNLNNNNNLNNNNNLNNNNZZZX..........', // 30
  '..........XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..........', // 31
  '..........XVSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSBX..........', // 32 stone body
  '..........XVSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSBX..........', // 33
  '..........XVSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSBX..........', // 34
  '..........XVSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSSSSSBSSSBX..........', // 35
  '..........XVBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBX..........', // 36
  '..........XVSSSSSSSSBSSSSSSXXXXXXXXXXSSSSSSSBSSSSSSSBX..........', // 37 fire mouth
  '..........XVSSSSSSSSBSSSSXXNNZZZZZZZZXXSSSSSBSSSSSSSBX..........', // 38
  '..........XVSSSSSSSSBSSSXXNNXXXXXXXXZZXXSSSSBSSSSSSSBX..........', // 39
  '..........XVSSSSSSSSBSSXXNNXXXXXXXXXXZZXXSSSBSSSSSSSBX..........', // 40
  '..........XVBBBBBBBBBBXNZZXXXXXXXXXXXXZZXXBBBBBBBBBBBX..........', // 41
  '..........XVSSSSBSSSSSXNZXXXXXXXXXXXXXXZZXSSSSSSBSSSBX..........', // 42
  '..........XVSSSSBSSSSSXNXXXXXXXXXXXXXXXXZXSSSSSSBSSSBX..........', // 43
  '..........XVSSSSBSSSSSXNXXXXXXXXXXXXXXXXZXSSSSSSBSSSBX..........', // 44
  '..........XVSSSSBSSSSSXNXXXXXXXXXXXXXXXXZXSSSSSSBSSSBX..........', // 45
  '..........XVBBBBBBBBBBXNXXXXXXXXX!XXXXXXZXBBBBBBBBBBBX..........', // 46 flame tips
  '..........XVSSSSSSSSBSXNXXXXXXX!XGXXXXXXZXSSBSSSSSSSBX..........', // 47
  '..........XVSSSSSSSSBSXNXXXXX!XGXGX!XXXXZXSSBSSSSSSSBX..........', // 48
  '..........XVSSSSSSSSBSXNXXXXXGXGNLXGXXXXZXSSBSSSSSSSBX..........', // 49
  '..........XVSSSSSSSSBSXNXXXXNGNLGLNGXXXXZXSSBSSSSSSSBX..........', // 50
  '..........XVBBBBBBBBBBXZXXNXGLGLGLGLNXXXZXBBBBBBBBBBBX..........', // 51
  '..........XVSSSSBSSSSSXZXXGNGLGLLLGLGNXXZXSSSSSSBSSSBX..........', // 52
  '..........XVSSSSBSSSSSXZXNGGLLLLLLLLGGNXZXSSSSSSBSSSBX..........', // 53
  '..........XVSSSSBSSSSSXZNGLGLLCCCCLLLGGNZXSSSSSSBSSSBX..........', // 54
  '..........XVSSSSBSSSSSXZGGLLLCCCCCCLLLGGZXSSSSSSBSSSBX..........', // 55 white-hot bed
  '..........XVBBBBBBBBBBXZBBBBBBBBBBBBBBBBZXBBBBBBBBBBBX..........', // 56
  '..........XBBBBBBBBBBBXXXXXXXXXXXXXXXXXXXXBBBBBBBBBBBX..........', // 57
  '.......XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.......', // 58 plinth, glow on its edge
  '.......XVSSSSSSSSSSSSSSSSNNNGGGGGGGGNNNSSSSSSSSSSSSSSSSSX.......', // 59
  '.......XBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBX.......', // 60
  '.......XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.......', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF98_GRID = REF98_GRID;
