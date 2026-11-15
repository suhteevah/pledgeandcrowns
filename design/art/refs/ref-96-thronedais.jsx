// SPDX-License-Identifier: MIT
// REF-96 - The Throne Dais, Act 10 district landmark and arena prop, 64x64.
// APPROVED by Matt 2026-11-15. Not wired yet (lands with Act 10).
//
// A three-step stone-grey dais with a wine runner down the middle and a
// high-backed stone throne on top: brass inlay around a wine cushion panel,
// brass along the arms and across the seat front, a brass finial with the
// one specular-white glint. A cobalt-deep banner hangs on a stand at either
// side, each with a small brass crown. Two aged-paper scrolls lie on the
// lowest step. Bottom-centre anchor. Specular white is 1 px.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  < cobalt deep  > cobalt  L brass leaf  G old gold
//         N antique brass  Z bronze  # specular white
//         O oxblood  W wineflesh  R dusty rose

const REF96_GRID = [
  '..............................XXXX..............................', //  0 finial
  '.............................XLL#GX.............................', //  1
  '......................XXXXXXXXXXXXXXXXXXXX......................', //  2 throne back
  '......................XSVGLLLLLLLLLLLLGSBX......................', //  3 brass inlay
  '.......XGGX...........XSVGOWWWWWWWWWWOGSBX...........XGGX.......', //  4 banner finials
  '.......XLLX...........XSVGOWWWWWWWWWWOGSBX...........XLLX.......', //  5
  '..XXXXXXXXXXXXXX......XSVGOWWWWWWWWWWOGSBX......XXXXXXXXXXXXXX..', //  6 crossbars
  '..XNNNNNNNNNNNNX......XSVGOWWWWWWWWWWOGSBX......XNNNNNNNNNNNNX..', //  7
  '..XXXXXXXXXXXXXX......XSVGOWWWWLLWWWWOGSBX......XXXXXXXXXXXXXX..', //  8 stud
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', //  9 banners
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 10
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 11
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 12
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 13
  '...X<<><<<<><<X.......XSVGOWWWWLLWWWWOGSBX.......X<<><<<<><<X...', // 14 stud
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 15
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 16
  '...X<<L<LL<L<<X.......XSVGOWWWWWWWWWWOGSBX.......X<<L<LL<L<<X...', // 17 brass crowns
  '...X<<LLLLLL<<X.......XSVGOWWWWWWWWWWOGSBX.......X<<LLLLLL<<X...', // 18
  '...X<<GGGGGG<<X.......XSVGOWWWWWWWWWWOGSBX.......X<<GGGGGG<<X...', // 19
  '...X<<><<<<><<X.......XSVGOWWWWLLWWWWOGSBX.......X<<><<<<><<X...', // 20 stud
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 21
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 22
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 23
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 24
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 25
  '...X<<><<<<><<X.......XSVGOWWWWWWWWWWOGSBX.......X<<><<<<><<X...', // 26
  '...X<<><<<<><<X.......XSVGLLLLLLLLLLLLGSBX.......X<<><<<<><<X...', // 27 inlay closes
  '...X<<><<<<><<X...XXXXXSVSSSSSSSSSSSSSSSBXXXXX...X<<><<<<><<X...', // 28 arm tops
  '...X<<><<<<><<X...XLLGXSVSSSSSSSSSSSSSSSBXLLGX...X<<><<<<><<X...', // 29 brass on the arms
  '...X<<><<<<><<X...XSVSXSVSSSSSSSSSSSSSSSBXSSBX...X<<><<<<><<X...', // 30
  '...X<<><<<<><<X...XSVSXSVSSSSSSSSSSSSSSSBXSSBX...X<<><<<<><<X...', // 31
  '...X<<><<<<><<X...XSVSXSVSSSSSSSSSSSSSSSBXSSBX...X<<><<<<><<X...', // 32
  '...X<<><<<<><<X...XSVSXSVSSSSSSSSSSSSSSSBXSSBX...X<<><<<<><<X...', // 33
  '...X<<><<<<><<X...XSVSXRRRRRRRRRRRRRRRRWWXSSBX...X<<><<<<><<X...', // 34 seat cushion
  '...X<<><<<<><<X...XSVSXWWWWWWWWWWWWWWWWWWXSSBX...X<<><<<<><<X...', // 35
  '...X<<><<<<><<X...XSVSXOOOOOOOOOOOOOOOOOOXSSBX...X<<><<<<><<X...', // 36
  '...X<<><XX<><<X...XSVSXXXXXXXXXXXXXXXXXXXXSSBX...X<<><XX<><<X...', // 37 swallowtails
  '...X<<>XNZX><<X...XSVSXSVSSSSSSSSSSSSSSSBXSSBX...X<<>XNZX><<X...', // 38
  '...X<<X.NZ.X<<X...XSVSXSVSSSSSSSSSSSSSSSBXSSBX...X<<X.NZ.X<<X...', // 39
  '...XXX..NZ..XXX...XSVSXGLLLLLLLLLLLLLLLLGXSSBX...XXX..NZ..XXX...', // 40 brass across the seat
  '........NZ........XSVSXSVSSSSSSSSSSSSSSSBXSSBX........NZ........', // 41 banner stands
  '........NZ........XSVSXSVSSSSSSSSSSSSSSSBXSSBX........NZ........', // 42
  '........NZ........XSVSXSVSSSSSSSSSSSSSSSBXSSBX........NZ........', // 43
  '........NZ....XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX....NZ........', // 44 top step
  '........NZ....XVVVVVVVVVVVOWWWWWWWWWWOVVVVVVVVVVVX....NZ........', // 45 wine runner
  '........NZ....XSSSSSSSSSSSOWWWWWWWWWWOSSSSSSSSSSBX....NZ........', // 46
  '........NZ....XSSSSSSSSSSSOWWWWWWWWWWOSSSSSSSSSSBX....NZ........', // 47
  '........NZ....XBBBBBBBBBBBOOOOOOOOOOOOBBBBBBBBBBBX....NZ........', // 48
  '........XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX........', // 49 middle step
  '........XVVVVVVVVVVVVVVVVVOWWWWWWWWWWOVVVVVVVVVVVVVVVVVX........', // 50
  '........XSSSSSSSSSSSSSSSSSOWWWWWWWWWWOSSSSSSSSSSSSSSSSBX........', // 51
  '........XSSSSSSSSSSSSSSSSSOWWWWWWWWWWOSSSSSSSSSSSSSSSSBX........', // 52
  '........XBBBBBBBBBBBBBBBBBOOOOOOOOOOOOBBBBBBBBBBBBBBBBBX........', // 53
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 54 bottom step
  '..XVVVVVVVVVVVVVVVVVVVVVVVOWWWWWWWWWWOVVVVVVVVVVVVVVVVVVVVVVVX..', // 55
  '..XSSSSSXXXXXXSSSSSSSSSSSSOWWWWWWWWWWOSSSSSSSSSSSSXXXXXXSSSSBX..', // 56 scrolls
  '..XSSSSXCVCCVCXSSSSSSSSSSSOWWWWWWWWWWOSSSSSSSSSSSXCVCCVCXSSSBX..', // 57
  '..XSSSSSXXXXXXSSSSSSSSSSSSOWWWWWWWWWWOSSSSSSSSSSSSXXXXXXSSSSBX..', // 58
  '..XBBBBBBBBBBBBBBBBBBBBBBBOOOOOOOOOOOOBBBBBBBBBBBBBBBBBBBBBBBX..', // 59
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..', // 60
  '................................................................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF96_GRID = REF96_GRID;
