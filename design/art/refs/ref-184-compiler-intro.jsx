// SPDX-License-Identifier: MIT
// REF-184 - The Compiler, intro frame, 64x64.
// DRAFT 2026-11-19. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// The moment the last trial begins. The approved REF-94 figure: both
// eyes burn specular white, the row of vents lights up cobalt, the
// device on the tabard lights, and rays of cobalt light break from
// behind the helm to the top corners of the frame. Same anchor and
// height as the idle sprite. Specular white is 17 px (cap 40).
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf
//         G old gold  N antique brass  Z bronze  # specular white

const REF184_GRID = [
  '........................X..X...XX...X..X........................', //  0
  '.......<>..............XLXXLXXLLLLXXLXXLX..............><.......', //  1
  '........<>.............XLLLLLLL#LLLLLLLGX.............><........', //  2
  '.........<>............XGLG>GLGLLGLG>GLGX............><.........', //  3
  '..........<>...........XGGGGGGGGGGGGGGGNX...........><..........', //  4
  '...........<>.........XXXXXXXXXXXXXXXXXXXX.........><...........', //  5
  '..>>........<>........XSSVSSSSSSSSSSSSSBBX........><........>>..', //  6
  '....>>>>.....<>.......XSSVSSSSSSSSSSSSSBBX.......><.....>>>>....', //  7
  '........>>>>..<>......XSVVSBBBBBBBBBBBSSBX......><..>>>>........', //  8
  '............>>><>.....XSVVSBB##BBB##BBSSBX.....><>>>............', //  9 burning eyes
  '................<>>>..XSVVSBB##BBB##BBSSBX..>>><................', // 10
  '.................<>...XSVVSBBBBBBBBBBBSSBX...><.................', // 11
  '..................<>..XSSVSSSSS>>SSSSSSBBX..><..................', // 12 lit vents
  '...................<>.XSSVSS>S>S>>S>S>SBBX.><...................', // 13
  '....<<<<<<<<<<<<<<<<..XSSVSSSSSSSSSSSSSBBX..<<<<<<<<<<<<<<<<....', // 14
  '.......................XSSVSSSSSSSSSSSBBX.......................', // 15
  '....XXXXXXXXXXXXXXXX...XXXXXXXXXXXXXXXXXX...XXXXXXXXXXXXXXXX....', // 16
  '...X#VVVSSSSSSSSSSSBXXXLLLLLLLL#LLLLLLLLGXXXSSSSSSSSSSSSSBBBX...', // 17
  '...XVVVSSSSSSSSSSSSB<<LLLLLLLLLLLLLLLLLLLG<<SSSSSSSSSSSSBBBBX...', // 18
  '...XVVSSSSSLLSSSSSSB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSLLSSSBBBBX...', // 19
  '...XVVSSSSSLLSSSSSSB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSLLSSSBBBBX...', // 20
  '...XVVSSSSSSSSSSSSBB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSSSSSBBBBBX...', // 21
  '...XVSSSSSSSSSSSSBBB<<SVSSSSCVVVVVVSSSSSBB<<SSSSSSSSSSBBBBBBX...', // 22
  '...XBBBBBBBBBBBBBBBB<<SVSSSSCVVVVVVSSSSSBB<<BBBBBBBBBBBBBBBBX...', // 23
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVLLVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 24
  '...X<<><<<<<><<<<<<<<<SVSSSSCVL##LVSSSSSBB<<<<<<<<<><<<<<<><X...', // 25 lit device
  '...X<<><<<<<><<<<<<<<<SVSSSSCLL##LLSSSSSBB<<<<<<<<<><<<<<<><X...', // 26
  '...X<<><<<<<><<<<<<<<<SVSSSSCVL##LVSSSSSBB<<<<<<<<<><<<<<<><X...', // 27
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVLLVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 28
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 29
  '...X<<><<<<<><<<<<<<<<SVSSSSCVVVVVVSSSSSBB<<<<<<<<<><<<<<<><X...', // 30
  '...XX<><<<<<><<<<<<<<<XXXXXSCVVVVVVSSXXXXX<<<<<<<<<><<<<<<>XX...', // 31
  '.....X><<<<<><<<<<<<<<VVSSXSCVVVVVVSSXSSBB<<<<<<<<<><<<<<<X.....', // 32
  '.....X><<<<<><<<<<<<<<VSSSXSCVVVVVVSSXSSBB<<<<<<<<<><<<<<<X.....', // 33
  '.....X><<<<<><<<<<<<<<VSSSXGGGGLLGGGGXSSBB<<<<<<<<<><<<<<<X.....', // 34
  '.....X><<<<<><<<<<<<<<XXXXXNNNNGGNNNNXXXXX<<<<<<<<<><<<<<<X.....', // 35
  '.....X><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<X.....', // 36
  '.....X><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<X.....', // 37
  '.....X><<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<<X.....', // 38
  '.....XX<<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<<XX.....', // 39
  '.......X<<<<><<<<<<<<<BBBBBBCVVVVVVSBBBBBB<<<<<<<<<><<<<X.......', // 40
  '.......X<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<X.......', // 41
  '.......X<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<X.......', // 42
  '.......X<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<X.......', // 43
  '.......X<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<X.......', // 44
  '.......X<<<<><<<<<<<<<BBBBBBCVVVVVVSBBBBBB<<<<<<<<<><<<<X.......', // 45
  '.......X<<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<<X.......', // 46
  '.......XX<<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<<XX.......', // 47
  '.........X<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<X.........', // 48
  '.........X<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<X.........', // 49
  '.........X<<><<<<<<<<<BBBBBBCVVVVVVSBBBBBB<<<<<<<<<><<X.........', // 50
  '.........X<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<X.........', // 51
  '.........X<<><<<<<<<<<SVSSSBCVVVVVVSBSSSBB<<<<<<<<<><<X.........', // 52
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

window.REF184_GRID = REF184_GRID;
