// SPDX-License-Identifier: MIT
// REF-103 - The Bell Tower, Act 1 Hearthstone Village landmark, 64x64.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// The village landmark and the Act 1 goal (the Bell Tower mission rings
// it three times). Same materials as the REF-68 cottage so it belongs to
// the village: a pointed wine roof, bronze timber over cream plaster, one
// cobalt window, a stone footing. The open belfry holds an old-gold bell
// with the only specular-white pixel. Bottom-centre anchor; solid from
// the base to the eave.
//
// Legend: X coalblack  O oxblood  W wineflesh  Y crypt
//         Z bronze  N antique brass  G old gold  L brass leaf  U bog umber
//         C parchment cream  V aged paper  S stone grey  B basalt
//         < cobalt deep  > cobalt  # specular white

const REF103_GRID = [
  '...............................LG...............................', //  0 finial
  '...............................GN...............................', //  1
  '..............................XXXX..............................', //  2 roof
  '.............................XWWYYX.............................', //  3
  '............................XWWYYYYX............................', //  4
  '...........................XWWOOOOYYX...........................', //  5
  '..........................XWWOOOOOOYYX..........................', //  6
  '.........................XWWYYYYYYYYYYX.........................', //  7
  '........................XWWOOOOOOOOOOYYX........................', //  8
  '.......................XWWOOOOOOOOOOOOYYX.......................', //  9
  '......................XWWYYYYYYYYYYYYYYYYX......................', // 10
  '.....................XWWOOOOOOOOOOOOOOOOYYX.....................', // 11
  '....................XWWOOOOOOOOOOOOOOOOOOYYX....................', // 12
  '...................XWWYYYYYYYYYYYYYYYYYYYYYYX...................', // 13
  '..................XWWOOOOOOOOOOOOOOOOOOOOOOYYX..................', // 14
  '.................XWWOOOOOOOOOOOOOOOOOOOOOOOOYYX.................', // 15
  '................XWWYYYYYYYYYYYYYYYYYYYYYYYYYYYYX................', // 16
  '...............XWWOOOOOOOOOOOOOOOOOOOOOOOOOOOOYYX...............', // 17
  '..............XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..............', // 18
  '....................XXXXXXXXXXXXXXXXXXXXXXXX....................', // 19 belfry
  '....................XNZXXXXXXXXNNXXXXXXXXZZX....................', // 20
  '....................XNZXXXXXXXXNNXXXXXXXXZZX....................', // 21
  '....................XNZXXXXXXXGGGGXXXXXXXZZX....................', // 22 bell
  '....................XNZXXXXXXG#GGGNXXXXXXZZX....................', // 23
  '....................XNZXXXXXXGLGGGNXXXXXXZZX....................', // 24
  '....................XNZXXXXXXGLGGGNXXXXXXZZX....................', // 25
  '....................XNZXXXXXXGLGGGNXXXXXXZZX....................', // 26
  '....................XNZXXXXXGLGGGGGNXXXXXZZX....................', // 27
  '....................XNZXXXXXNNNNNNNNXXXXXZZX....................', // 28
  '....................XNZXXXXXXXXZZXXXXXXXXZZX....................', // 29
  '....................XNZXXXXXXXXXXXXXXXXXXZZX....................', // 30
  '....................XXXXXXXXXXXXXXXXXXXXXXXX....................', // 31
  '..................XXXXXXXXXXXXXXXXXXXXXXXXXXXX..................', // 32 sill
  '..................XVVSSSSSSSSSSSSSSSSSSSSSSSSX..................', // 33
  '..................XXXXXXXXXXXXXXXXXXXXXXXXXXXX..................', // 34
  '....................XNZXXXXXXXXXXXXXXXXXVZZX....................', // 35 shaft
  '....................XNZCCCCCCCCCCCCCCCCCVZZX....................', // 36
  '....................XNZCCCCCXXXXXXXXCCCCVZZX....................', // 37 window
  '....................XNZCCCCCX<><X<<XCCCCVZZX....................', // 38
  '....................XNZCCCCCX<><X<<XCCCCVZZX....................', // 39
  '....................XNZCCCCCXXXXXXXXCCCCVZZX....................', // 40
  '....................XNZCCCCCX<<<X<<XCCCCVZZX....................', // 41
  '....................XNZCCCCCX<<<X<<XCCCCVZZX....................', // 42
  '....................XNZCCCCCXXXXXXXXCCCCVZZX....................', // 43
  '....................XNZCCCCCCCCCCCCCCCCCVZZX....................', // 44
  '....................XNZNNNNNNNNNNNNNNNNNNZZX....................', // 45
  '....................XNZZZZZZZZZZZZZZZZZZZZZX....................', // 46
  '....................XNZCCCCCCCCCCCCCCCCCVZZX....................', // 47
  '....................XNZCCCCCCCCCCCCCCCCCVZZX....................', // 48
  '....................XNZCCCCCCXXXXXXCCCCCVZZX....................', // 49 door
  '....................XNZCCCCCXZNZZZUXCCCCVZZX....................', // 50
  '....................XNZCCCCCXZNZZZUXCCCCVZZX....................', // 51
  '....................XNZCCCCCXZNZZZUXCCCCVZZX....................', // 52
  '....................XNZCCCCCXZNZZZUXCCCCVZZX....................', // 53
  '....................XNZCCCCCXZNZZGUXCCCCVZZX....................', // 54
  '....................XNZSSSBSXZNZZZUXSSBSSZZX....................', // 55
  '....................XNZSSSBSXZNZZZUXSSBSSZZX....................', // 56
  '....................XNZBBBBBXZNZZZUXBBBBBZZX....................', // 57
  '....................XNZSBSSSXZNZZZUXSSSBSZZX....................', // 58
  '..................XXXXXXXXXXXXXXXXXXXXXXXXXXXX..................', // 59 base
  '..................XVSSSSSSSSSSSSSSSSSSSSSSSSSX..................', // 60
  '..................XXXXXXXXXXXXXXXXXXXXXXXXXXXX..................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF103_GRID = REF103_GRID;
