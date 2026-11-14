// SPDX-License-Identifier: MIT
// REF-67 - Village oak, 32x48 prop sprite, transparent bg.
// APPROVED by Matt 2026-11-13 (tranche 1); wired as sprites/props/oak.png.
//
// A PROP SPRITE, not a tile: spawn as a y-sorted Sprite so the player walks
// behind the canopy. Anchor bottom-centre at the trunk base. Replaces the
// 16x16 sapling tile used for the treeline.
//
// Legend: J mossbed (canopy outline)  F pine (shade)  E forest (mid)
//         H spring meadow (lit)  Q hayfield (specular leaf)
//         X coalblack (trunk outline)  N antique brass  Z bronze  U bog umber

const REF67_GRID = [
  '................................', //  0
  '............JJJJJJJJ............', //  1
  '.........JHHHHHHEEEEEEJ.........', //  2
  '.......JHHQHHHHHEEEEEEEFJ.......', //  3
  '......JHHHHHHHHHEEEEEEEEFJ......', //  4
  '.....JHHHHQHHHHEEEEEEEEEFFJ.....', //  5
  '....JHHHHHHHHEEEEEFEEEEEEFFJ....', //  6
  '...JEHHHHHHEEEEEFFEEEEEEEEFFJ...', //  7
  '...JEHHHHHEEEEEEEFFEEEEEEEFFJ...', //  8
  '..JEEHHHHEEEEHHEEEEEEEEEEEFFFJ..', //  9
  '..JEEHHHEEEEHHHHEEEEEEEEEEFFFJ..', // 10
  '..JFEEEEEEEEEHHEEEEEEEEFEEFFFJ..', // 11
  '..JFFEEEEEEEEEEEEEEEEEFFEEEFFJ..', // 12
  '..JEFFEEHHEEEEEEEEEEEEEEEEFFFJ..', // 13
  '..JEEEEHHHHEEEEEEEHHEEEEEEFFFJ..', // 14
  '..JEEEHHHEEEEEEEEHHHHEEEEEFFFJ..', // 15
  '..JEEEEEEEEEFEEEEEHHEEEEEFFFFJ..', // 16
  '..JFEEEEEEEFFEEEEEEEEEEEFFFFFJ..', // 17
  '..JFFEEEEEEEEEEEEEEEEEFFFFFFFJ..', // 18
  '..JEFFEEEHHEEEEEEEEEEFFEEFFFFJ..', // 19
  '..JEEFEEHHHEEEEEEFEEEEEEEFFFFJ..', // 20
  '..JEEEEEEEEEEEEFFEEEEEEFFFFFFJ..', // 21
  '...JEEEEEEEEEEEEEEEEEFFFFFFFJ...', // 22
  '...JFEEEEEEEFEEEEEEFFFFFFFFFJ...', // 23
  '....JFFEEEEEEEEEEFFFFFFFFFFJ....', // 24
  '.....JFFFEEEEEEFFFFFFFFFFFJ.....', // 25
  '......JFFFFEEFFFFFFFFFFFFJ......', // 26
  '......JJJFFFFFFFFFFFFFFJJJ......', // 27
  '.........JJJFFFFFFFFJJJ.........', // 28
  '............JJUZZUJJ............', // 29
  '............XNNZZZUX............', // 30
  '............XNZZZZUX............', // 31
  '............XNNZUZUX............', // 32
  '............XNNZZZUX............', // 33
  '............XNZZZZUX............', // 34
  '............XNNZZUUX............', // 35
  '............XNNZZZUX............', // 36
  '............XNZUZZUX............', // 37
  '............XNNZZZUX............', // 38
  '............XNNZZZUX............', // 39
  '............XNZZZUUX............', // 40
  '............XNNZZZUX............', // 41
  '...........XNNNZZZUUX...........', // 42
  '..........XNNZZZZZZUUX..........', // 43
  '..........XXXXXXXXXXXX..........', // 44
  '................................', // 45
  '................................', // 46
  '................................', // 47
];

window.REF67_GRID = REF67_GRID;
