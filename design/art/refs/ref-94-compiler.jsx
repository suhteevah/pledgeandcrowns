// SPDX-License-Identifier: MIT
// REF-94 - The Compiler, Act 10 final boss, 64x64, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Colossal, filling the frame edge to edge. A brass-leaf crown over a
// stone-grey helm with two cobalt eyes and a row of vents for a mouth - it
// has no face, only output. Squared stone pauldrons with brass studs, a
// brass gorget and belt, an aged-paper tabard with a brass device, and a
// cobalt-deep cape that falls from the shoulders to the floor on both sides.
// Specular white is 7 px of 4096 (cap 40): crown, eyes, one pauldron edge,
// gorget, device.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         N antique brass  Z bronze  # specular white

const REF94_GRID = [
  '........................X..X...XX...X..X........................', //  0 crown points
  '.......................XLXXLXXLLLLXXLXXLX.......................', //  1
  '.......................XLLLLLLL#LLLLLLLGX.......................', //  2
  '.......................XGLG>GLGLLGLG>GLGX.......................', //  3 gems
  '.......................XGGGGGGGGGGGGGGGNX.......................', //  4
  '......................XXXXXXXXXXXXXXXXXXXX......................', //  5
  '......................XSSVSSSSSSSSSSSSSBBX......................', //  6 helm
  '......................XSSVSSSSSSSSSSSSSBBX......................', //  7
  '......................XSVVSBBBBBBBBBBBSSBX......................', //  8 brow
  '......................XSVVSBB>>BBB>>BBSSBX......................', //  9 eyes
  '......................XSVVSBB>#BBB>#BBSSBX......................', // 10
  '......................XSVVSBBBBBBBBBBBSSBX......................', // 11
  '......................XSSVSSSSSXXSSSSSSBBX......................', // 12
  '......................XSSVSSXSXSXXSXSXSBBX......................', // 13 vents
  '......................XSSVSSSSSSSSSSSSSBBX......................', // 14
  '.......................XSSVSSSSSSSSSSSBBX.......................', // 15 jaw
  '....XXXXXXXXXXXXXXXX...XXXXXXXXXXXXXXXXXX...XXXXXXXXXXXXXXXX....', // 16 pauldron tops
  '...X#VVVSSSSSSSSSSSBXXXLLLLLLLL#LLLLLLLLGXXXSSSSSSSSSSSSSBBBX...', // 17 gorget
  '...XVVVSSSSSSSSSSSSB<<LLLLLLLLLLLLLLLLLLLG<<SSSSSSSSSSSSBBBBX...', // 18
  '...XVVSSSSSLLSSSSSSB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSLLSSSBBBBX...', // 19 brass studs
  '...XVVSSSSSLLSSSSSSB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSLLSSSBBBBX...', // 20
  '...XVVSSSSSSSSSSSSBB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSSSSSBBBBBX...', // 21
  '...XVSSSSSSSSSSSSBBB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSSSSBBBBBBX...', // 22
  '...XBBBBBBBBBBBBBBBB<<SVSSSSCVVVVVVSSSSSBB<<BBBBBBBBBBBBBBBBX...', // 23
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVLLVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 24 cape, device
  '...X<<><<<<<><<<<<<<<<SVSSSSCVL##LVSSSSSBB<<<<<<<<<><<<<<<><X...', // 25
  '...X<<><<<<<><<<<<<<<<SVSSSSCLLGGLLSSSSSBB<<<<<<<<<><<<<<<><X...', // 26
  '...X<<><<<<<><<<<<<<<<SVSSSSCVLGGLVSSSSSBB<<<<<<<<<><<<<<<><X...', // 27
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVLLVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 28
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 29
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 30
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 31
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 32
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 33
  '...X<<><<<<<><<<<<<<<<GGGGGGGGGLLGGGGGGGGN<<<<<<<<<><<<<<<><X...', // 34 belt
  '...X<<><<<<<><<<<<<<<<NNNNNNNNNGGNNNNNNNNZ<<<<<<<<<><<<<<<><X...', // 35
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 36 tassets
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 37
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 38
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 39
  '...X<<><<<<<><<<<<<<<<BBBBBBCVVVVVVSBBBBBB<<<<<<<<<><<<<<<><X...', // 40 plate line
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 41
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 42
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 43
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 44
  '...X<<><<<<<><<<<<<<<<BBBBBBCVVVVVVSBBBBBB<<<<<<<<<><<<<<<><X...', // 45 plate line
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 46
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 47
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 48
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 49
  '...X<<><<<<<><<<<<<<<<BBBBBBCVVVVVVSBBBBBB<<<<<<<<<><<<<<<><X...', // 50 plate line
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 51
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 52
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 53
  '...X<<><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<><X...', // 54
  '...X<<><<<<<><<<<<<<<<SVSSSBBCVVVVSBBSSSBB<<<<<<<<<><<<<<<><X...', // 55 tabard ends
  '...X<<><<<<<><<<<<<<<<<<<<><<<<<<<<<<><<<<<<<<<<<<<><<<<<<><X...', // 56 hem
  '...XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX...', // 57
  '..............XSSSSSSSSSSX............XSSSSSSSSBBX..............', // 58 boots
  '..............XSSSSSSSSSSX............XSSSSSSSSBBX..............', // 59
  '..............XXXXXXXXXXXX............XXXXXXXXXXXX..............', // 60
  '................................................................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF94_GRID = REF94_GRID;
