// SPDX-License-Identifier: MIT
// REF-131 - The Mage's Tower, Act 4 district landmark, 64x64.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/mages_tower.png.
//
// A round stone tower lit from the left, wound with a gold band, under a
// cobalt cone roof with four brass stars and a gold finial. One window
// glows mage violet: 24 px of royal arcane frame and 22 px of mage glow,
// the only violet in the district and well under the 5% cap. A cobalt
// door with gold handles. Bottom-centre anchor; solid from the base to
// the eave.
//
// Legend: X coalblack  S stone grey  V aged paper  C parchment cream
//         B basalt  G old gold  L brass leaf  N antique brass
//         < cobalt deep  > cobalt  * royal arcane  % mage glow
//         M mist teal

const REF131_GRID = [
  '...............................XX...............................', //  0 finial
  '..............................XLGX..............................', //  1
  '...............................XX...............................', //  2
  '...............................XX...............................', //  3 cone roof
  '...............................XX...............................', //  4
  '..............................X<<X..............................', //  5
  '.............................X<<<<X.............................', //  6
  '............................X>><<<<X............................', //  7
  '...........................X>><<<<<<X...........................', //  8
  '..........................X>><<<<><<<X..........................', //  9
  '.........................X>><L<<><<<<<X.........................', // 10
  '........................X>><<<<><<<<<<>X........................', // 11
  '.......................X>><<<<><<<<<<><<X.......................', // 12
  '.......................X>><<<><<<<<<><<<X.......................', // 13
  '......................X>><<<><<<<<<L<<<<<X......................', // 14
  '.....................X>><<<><<<<<<><<<<<<>X.....................', // 15
  '....................X>><<<><<<<<<><<<<<<><<X....................', // 16
  '...................X>><<<>L<<<<<><<<<<<><<<<X...................', // 17
  '..................X>><<<><<<<<<><<<<<<><<<<<<X..................', // 18
  '.................X>><<<><<<<<<><<<<<<L<<<<<<><X.................', // 19
  '................X>><<<><<<<<<><<<<<<><<<<<<><<<X................', // 20
  '...............X>><<<><<<<<<><<<<<<><<<<<<><<<<<X...............', // 21
  '..............XLLGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGX..............', // 22 gold eave
  '..............XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..............', // 23
  '..................XCCCVVVVVVSSSSSSSSSSSSSBBBBX..................', // 24
  '..................XCCCVVVVVVSSSSSSSSSSSSSBBBBX..................', // 25
  '..................XCCCVVVVVVSSSSSSSSSSSSSBBBBX..................', // 26
  '..................XCCCVVVVVVSSSSSSSSSSSSSBBBBX..................', // 27
  '..................XSSSSSSSSSBBBBBBBBBBBBBBBBGX..................', // 28
  '..................XCCCVVVVVVSSSSSSSSSSSGGGGGNX..................', // 29 gold band
  '..................XCCCVVVVVVSSSSSSGGGGGNNNNNBX..................', // 30
  '..................XCCCVVVVVVSGGGGGNNNNNSSBBBBX..................', // 31
  '..................XCCCVVGGGGGNNNNNSSSSSSSBBBBX..................', // 32
  '..................XGGGGGNNNNNSSSSSSSSSSSSBBBBX..................', // 33
  '..................XNNNNNVVVVSSSSSSSSSSSSSBBBBX..................', // 34
  '..................XSSSSSSSSSBBBBBBBBBBBBBBBBBX..................', // 35
  '..................XCCCVVVVVVSSSSSSSSSSSSSBBBBX..................', // 36
  '..................XCCCVVVVVVSXXXXXXSSSSSSBBBBX..................', // 37 glowing window
  '..................XCCCVVVVVVX******XSSSSSBBBBX..................', // 38
  '..................XCCCVVVVVVX*%%%%*XSSSSSBBBBX..................', // 39
  '..................XCCCVVVVVVX*%M%%*XSSSSSBBBBX..................', // 40
  '..................XCCCVVVVVVX*%%M%*XSSSSSBBBBX..................', // 41
  '..................XSSSSSSSSSX*%%%%*XBBBBBBBBBX..................', // 42
  '..................XCCCVVVVVVX*%%%%*XSSSSSBBBBX..................', // 43
  '..................XCCCVVVVVVX*%%%%*XSSSSSBBBBX..................', // 44
  '..................XCCCVVVVVVX******XSSSSSBBBBX..................', // 45
  '..................XCCCVVVVVVXXXXXXXXSSSSSBBBBX..................', // 46
  '..................XCCCVVVVVVSSSSSSSSSSSSSBBBBX..................', // 47
  '..................XCCCVVVVVVSSSSSSSSSSSSSBBBBX..................', // 48
  '..................XSSSSSSSVXXXXXXXXXXSBBBBBBBX..................', // 49 door
  '..................XCCCVVVVX<<<<XX<<<<XSSSBBBBX..................', // 50
  '..................XCCCVVVVX<<><XX<><<XSSSBBBBX..................', // 51
  '..................XCCCVVVVX<<><XX<><<XSSSBBBBX..................', // 52
  '..................XCCCVVVVX<<><XX<><<XSSSBBBBX..................', // 53
  '..................XCCCVVVVX<<><XX<><<XSSSBBBBX..................', // 54
  '..................XCCCVVVVX<<>GXXG><<XSSSBBBBX..................', // 55
  '..................XSSSSSSSX<<><XX<><<XBBBBBBBX..................', // 56
  '..................XCCCVVVVX<<><XX<><<XSSSBBBBX..................', // 57
  '...............XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX...............', // 58 base
  '...............XVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVX...............', // 59
  '...............XBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBX...............', // 60
  '...............XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX...............', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF131_GRID = REF131_GRID;
