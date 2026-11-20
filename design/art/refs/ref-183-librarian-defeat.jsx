// SPDX-License-Identifier: MIT
// REF-183 - The Librarian, defeat frame, 64x64.
// APPROVED by Matt 2026-11-19. Wired as game/assets/sprites/npc/librarian_defeat_0.png.
//
// The moment the player's code passes her trial. She kneels: the
// REF-93 figure with twelve rows taken out, so the hem stays on the
// ground line. Her head is bowed: the hood sits three rows lower,
// sunk into the shoulders, with the brow down and no eyes showing.
// The sigil has gone out. The two drawers of the catalogue that
// remain both stand open with their cards showing: the Library,
// indexed, and open at last. Same anchor as the idle sprite.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  P pink quartz
//         S stone grey  B basalt  N antique brass  Z bronze
//         G old gold

const REF183_GRID = [
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
  '............................XXXXXXXX............................', // 15 hood, bowed
  '.........................XXXYYOOOOYYXXX.........................', // 16
  '.......................XXYYYOOOOYYYYYYKXX.......................', // 17
  '......................XYYYOOOYYYYYYYYYYKKX......................', // 18
  '.....................XYYYOOOYYYYYYYYYYYYKKX.....................', // 19
  '....................XYYYOOYYYYYYYYYYYYYYYKKX....................', // 20
  '....................XYYOOYYYYYYYYYYYYYYYYKKX....................', // 21
  '....................XYYOOYYYYYYYYYYYYYYYYKKX....................', // 22
  '....................XYYOOYYYYYYYYYYYYYYYYKKX....................', // 23
  '....................XYYOO**************YYKKX....................', // 24 brow, no eyes
  '....................XYYOO**KKKKKKKKKK**YYKKX....................', // 25
  '....................XYYOO**KKKKKKKKKK**YYKKX....................', // 26
  '....................XYYOO**KKKKKKKKKK**YYKKX....................', // 27
  '.................XYYXYYOO***KKKKKKKK***YYKKXKKX.................', // 28
  '...............XYYYYXYYYOO***KKKKKK***YYYKKXYYKKX...............', // 29
  '..............XYYYOOXYYYYOO****KK****YYYYKKXYYYKKX..............', // 30
  '..............XYYYOOYYYYYCVVVV**VVVVSYYYYYYKYYYKKX..............', // 31
  '..............XYYYOOYYYYYCVVV****VVVSYYYYYYKYYYKKX..............', // 32
  '..............XYYYOOYYYYYCVV******VVSYYYYYYKYYYKKX..............', // 33
  '..............XYYYOOYYYYYCVVV****VVVSYYYYYYKYYYKKX..............', // 34
  '..............XYYYOOYYYYYCVVVV**VVVVSYYYYYYKYYYKKX..............', // 35
  '..............XYYOOOYYYYYCVVVVVVVVVVSYYYYYOOOYYKKX..............', // 36
  '..............XYYYOOOYYYYCVVVVVVVVVVSYYYYOOOYYYKKX..............', // 37
  '..............XYYYYOOOOYYCVVVVVVVVVVSYYOOOOYYYYKKX..............', // 38
  '..............XYYYYYYOOOOOOPPPPPPPPOOOOOOYYYYYYKKX..............', // 39
  '..............XYYYYYYYKKKKKPPPPPPPPKKKKKYYYYYYYKKX..............', // 40
  '..............XYYYOOYYYYYCVKKKKKKKKVSYYYYYYKYYYKKX..............', // 41
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 42
  '..............XYSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBYX..............', // 43
  '..............XYBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSYX..............', // 44
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 45
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 46
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 47
  '..............XYYYOOYYYYXKCKCKCKCKCKCKKXYYYKYYYKKX..............', // 48 open drawers
  '..............XYYYOOYYYYXKKKKKKKKKKKKKKXYYYKYYYKKX..............', // 49
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 50
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 51
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 52
  '..............XYYYOOYYYYXKCKCKCKCKCKCKKXYYYKYYYKKX..............', // 53
  '..............XYYYOOYYYYXKKKKKKKKKKKKKKXYYYKYYYKKX..............', // 54
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 55
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 56
  '..............XYYYOOYYYYYYCVVVVVVVVSYYYYYYYKYYYKKX..............', // 57
  '..............XOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOKKKX..............', // 58
  '..............XYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYKKKX..............', // 59
  '..............XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..............', // 60
  '................................................................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF183_GRID = REF183_GRID;
