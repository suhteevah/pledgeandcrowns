// SPDX-License-Identifier: MIT
// REF-185 - The Compiler, defeat frame, 64x64.
// DRAFT 2026-11-19. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// The moment the player's last program passes. The colossus kneels:
// the REF-94 figure with twelve rows taken out below the belt, so
// the boots stay on the ground line. The crowned helm is bowed three
// rows onto the gorget, both eyes are dark, and the device on the
// tabard has gone dull. The crown stays on. Same anchor as the idle
// sprite.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf
//         G old gold  N antique brass  Z bronze

const REF185_GRID = [
  '................................................................', //  0
  '................................................................', //  1
  '................................................................', //  2
  '................................................................', //  3
  '................................................................', //  4
  '................................................................', //  5
  '................................................................', //  6
  '................................................................', //  7
  '................................................................', //  8
  '................................................................', //  9
  '................................................................', // 10
  '................................................................', // 11
  '................................................................', // 12
  '................................................................', // 13
  '................................................................', // 14
  '........................X..X...XX...X..X........................', // 15 crown and helm, bowed
  '.......................XLXXLXXLLLLXXLXXLX.......................', // 16
  '.......................XLLLLLLLLLLLLLLLGX.......................', // 17
  '.......................XGLGBGLGLLGLGBGLGX.......................', // 18
  '.......................XGGGGGGGGGGGGGGGNX.......................', // 19
  '......................XXXXXXXXXXXXXXXXXXXX......................', // 20
  '......................XSSVSSSSSSSSSSSSSBBX......................', // 21
  '......................XSSVSSSSSSSSSSSSSBBX......................', // 22
  '......................XSVVSBBBBBBBBBBBSSBX......................', // 23
  '......................XSVVSBBBBBBBBBBBSSBX......................', // 24 dark eyes
  '......................XSVVSBBBLBBBBLBBSSBX......................', // 25
  '......................XSVVSBBBBBBBBBBBSSBX......................', // 26
  '......................XSSVSSSSSXXSSSSSSBBX......................', // 27
  '....XXXXXXXXXXXXXXXX..XSSVSSXSXSXXSXSXSBBX..XXXXXXXXXXXXXXXX....', // 28
  '...X#VVVSSSSSSSSSSSBXXXSSVSSSSSSSSSSSSSBBXXXSSSSSSSSSSSSSBBBX...', // 29
  '...XVVVSSSSSSSSSSSSB<<LXSSVSSSSSSSSSSSBBXG<<SSSSSSSSSSSSBBBBX...', // 30
  '...XVVSSSSSLLSSSSSSB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSLLSSSBBBBX...', // 31
  '...XVVSSSSSLLSSSSSSB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSLLSSSBBBBX...', // 32
  '...XVVSSSSSSSSSSSSBB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSSSSSBBBBBX...', // 33
  '...XVSSSSSSSSSSSSBBB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSSSSBBBBBBX...', // 34
  '...XBBBBBBBBBBBBBBBB<<SVSSSSCVVVVVVSSSSSBB<<BBBBBBBBBBBBBBBBX...', // 35
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVLLVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 36
  '...X<<><<<<<><<<<<<<<<SVSSSSCVLGGLVSSSSSBB<<<<<<<<<><<<<<<><X...', // 37
  '...X<<><<<<<><<<<<<<<<SVSSSSCLLGGLLSSSSSBB<<<<<<<<<><<<<<<><X...', // 38
  '...X<<><<<<<><<<<<<<<<SVSSSSCVLGGLVSSSSSBB<<<<<<<<<><<<<<<><X...', // 39
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVLLVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 40
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 41
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 42
  '...XX<><<<<<><<<<<<<<<XXXXXSCVVVVVVSSXXXXX<<<<<<<<<><<<<<<>XX...', // 43
  '.....X><<<<<><<<<<<<<<VVSSXSCVVVVVVSSXSSBB<<<<<<<<<><<<<<<X.....', // 44
  '.....X><<<<<><<<<<<<<<VSSSXSCVVVVVVSSXSSBB<<<<<<<<<><<<<<<X.....', // 45
  '.....X><<<<<><<<<<<<<<VSSSXGGGGLLGGGGXSSBB<<<<<<<<<><<<<<<X.....', // 46
  '.....X><<<<<><<<<<<<<<XXXXXNNNNGGNNNNXXXXX<<<<<<<<<><<<<<<X.....', // 47
  '.....X><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<X.....', // 48
  '.....X><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<X.....', // 49
  '.....X><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<X.....', // 50
  '.....XX<<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<XX.....', // 51
  '.......XX<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<XX.......', // 52
  '.........X<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<X.........', // 53
  '.........X<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<X.........', // 54
  '.........X<<><<<<<<<<<SVSSSBBCVVVVSBBSSSBB<<<<<<<<<><<X.........', // 55
  '.........X<<><<<<<<<<<<<<<><<<<<<<<><<<<<<<<<<<<<<<><<X.........', // 56
  '.........XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.........', // 57
  '..............XSSSSSSSSSSX............XSSSSSSSSBBX..............', // 58
  '..............XSSSSSSSSSSX............XSSSSSSSSBBX..............', // 59
  '..............XXXXXXXXXXXX............XXXXXXXXXXXX..............', // 60
  '................................................................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF185_GRID = REF185_GRID;
