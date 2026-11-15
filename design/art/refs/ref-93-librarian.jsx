// SPDX-License-Identifier: MIT
// REF-93 - The Librarian, Act 9 boss, 64x64, idle.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// The Library kit (REF-79) at boss scale: a hooded column so tall the hood
// touches the top of the frame. Two cream eyes in the hood, a large violet
// sigil on the scapular, long sleeves meeting at clasped hands, a doubled
// chain at the belt. Lesson prop: a catalogue cabinet of three drawers set
// into the robe below the chain, each with a blank label and a brass pull -
// the whole Library, indexed, and none of it open.
// Violet is about 80 px of 4096, under the 5% cap (204). No alarm scarlet.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow  P pink quartz
//         S stone grey  B basalt  N antique brass  Z bronze  G old gold

const REF93_GRID = [
  '............................XXXXXXXX............................', //  0 hood peak
  '.........................XXXYYOOOOYYXXX.........................', //  1
  '.......................XXYYYOOOOYYYYYYKXX.......................', //  2
  '......................XYYYOOOYYYYYYYYYYKKX......................', //  3
  '.....................XYYYOOOYYYYYYYYYYYYKKX.....................', //  4
  '....................XYYYOOYYYYYYYYYYYYYYYKKX....................', //  5
  '....................XYYOOY************YYYKKX....................', //  6 hood mouth
  '....................XYYOO**KKKKKKKKKK**YYKKX....................', //  7
  '....................XYYOO**KKKKKKKKKK**YYKKX....................', //  8
  '....................XYYOO**KKCCKKCCKK**YYKKX....................', //  9 eyes
  '....................XYYOO**KKCCKKCCKK**YYKKX....................', // 10
  '....................XYYOO**KKKKKKKKKK**YYKKX....................', // 11
  '....................XYYOO**KKKKKKKKKK**YYKKX....................', // 12
  '....................XYYOO***KKKKKKKK***YYKKX....................', // 13
  '....................XYYYOO***KKKKKK***YYYKKX....................', // 14
  '....................XYYYYOO****KK****YYYYKKX....................', // 15
  '.................XYYYYYYOOO***VVVV***YYYYYYYKKX.................', // 16 shoulders
  '...............XYYYYYYYOOOCVVVVVVVVVVSYYYYYYYYKKX...............', // 17
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 18
  '..............XYYYOOYYYYYCVVVV**VVVVSYYYYYYKYYYKKX..............', // 19 sigil
  '..............XYYYOOYYYYYCVVV*%%*VVVSYYYYYYKYYYKKX..............', // 20
  '..............XYYYOOYYYYYCVV**%%**VVSYYYYYYKYYYKKX..............', // 21
  '..............XYYYOOYYYYYCVVV*%%*VVVSYYYYYYKYYYKKX..............', // 22
  '..............XYYYOOYYYYYCVVVV**VVVVSYYYYYYKYYYKKX..............', // 23
  '..............XYYOOOYYYYYCVVVVVVVVVVSYYYYYOOOYYKKX..............', // 24 sleeves
  '..............XYYYOOOYYYYCVVVVVVVVVVSYYYYOOOYYYKKX..............', // 25
  '..............XYYYYOOOOYYCVVVVVVVVVVSYYOOOOYYYYKKX..............', // 26
  '..............XYYYYYYOOOOOOPPPPPPPPOOOOOOYYYYYYKKX..............', // 27 clasped hands
  '..............XYYYYYYYKKKKKPPPPPPPPKKKKKYYYYYYYKKX..............', // 28
  '..............XYYYOOYYYYYCVKKKKKKKKVSYYYYYYKYYYKKX..............', // 29
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 30
  '..............XYSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBYX..............', // 31 chain
  '..............XYBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSYX..............', // 32
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 33
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 34 catalogue cabinet
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 35 drawer one
  '..............XYYYOOYYYYXNZZCCCCCCCCZZNXYYYKYYYKKX..............', // 36 blank label
  '..............XYYYOOYYYYXNZZZZZGGZZZZZNXYYYKYYYKKX..............', // 37 brass pull
  '..............XYYYOOYYYYXZZZZZZZZZZZZZZXYYYKYYYKKX..............', // 38
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 39
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 40 drawer two
  '..............XYYYOOYYYYXNZZCCCCCCCCZZNXYYYKYYYKKX..............', // 41
  '..............XYYYOOYYYYXNZZZZZGGZZZZZNXYYYKYYYKKX..............', // 42
  '..............XYYYOOYYYYXZZZZZZZZZZZZZZXYYYKYYYKKX..............', // 43
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 44
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 45 drawer three
  '..............XYYYOOYYYYXNZZCCCCCCCCZZNXYYYKYYYKKX..............', // 46
  '..............XYYYOOYYYYXNZZZZZGGZZZZZNXYYYKYYYKKX..............', // 47
  '..............XYYYOOYYYYXZZZZZZZZZZZZZZXYYYKYYYKKX..............', // 48
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 49
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 50
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 51
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 52
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 53
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 54
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 55
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 56
  '..............XYYYOOYYYYYYCVVVVVVVVSYYYYYYYKYYYKKX..............', // 57 scapular ends
  '..............XOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOKKKX..............', // 58 hem
  '..............XYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYKKKX..............', // 59
  '..............XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..............', // 60
  '................................................................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF93_GRID = REF93_GRID;
