// SPDX-License-Identifier: MIT
// REF-182 - The Librarian, intro frame, 64x64.
// DRAFT 2026-11-19. NOT approved, NOT wired. Awaiting Matt on the design canvas.
//
// The moment her trial begins. The approved REF-93 figure: her cream
// eyes widen to three pixels, the sigil on the scapular flares from
// royal arcane to mage glow, the top drawer of the catalogue stands
// open with its cards showing, six loose pages lift off either side
// of her and six violet motes hang between them. Same anchor and
// height as the idle sprite. Violet stays under the 5% cap.
//
// Legend: X coalblack  K inkblood  Y crypt  O oxblood  V aged paper
//         C parchment cream  * royal arcane  % mage glow
//         P pink quartz  S stone grey  B basalt  N antique brass
//         Z bronze  G old gold

const REF182_GRID = [
  '............................XXXXXXXX............................', //  0
  '.........................XXXYYOOOOYYXXX.........................', //  1
  '.......................XXYYYOOOOYYYYYYKXX.......................', //  2
  '......................XYYYOOOYYYYYYYYYYKKX......................', //  3
  '.....................XYYYOOOYYYYYYYYYYYYKKX.....................', //  4
  '....................XYYYOOYYYYYYYYYYYYYYYKKX....................', //  5
  '.....XXXX...........XYYOOY************YYYKKX....................', //  6
  '.....XCVX...........XYYOO**KKKKKKKKKK**YYKKX....................', //  7
  '.....XVVX...........XYYOO**KKKKKKKKKK**YYKKX....................', //  8
  '.....XXXX...........XYYOO**KCCCKKCCCK**YYKKX...........XXXX.....', //  9 wide eyes
  '....................XYYOO**KCCCKKCCCK**YYKKX...........XCVX.....', // 10
  '....................XYYOO**KKKKKKKKKK**YYKKX...........XVVX.....', // 11
  '....................XYYOO**KKKKKKKKKK**YYKKX...........XXXX.....', // 12
  '....................XYYOO***KKKKKKKK***YYKKX....................', // 13
  '..........*.........XYYYOO***KKKKKK***YYYKKX....................', // 14
  '....................XYYYYOO****KK****YYYYKKX....................', // 15
  '.................XYYYYYYOOO***VVVV***YYYYYYYKKX.................', // 16
  '...............XYYYYYYYOOOCVVVVVVVVVVSYYYYYYYYKKX....%..........', // 17
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 18
  '..............XYYYOOYYYYYCVVVV%%VVVVSYYYYYYKYYYKKX..............', // 19
  '..............XYYYOOYYYYYCVVV%%%%VVVSYYYYYYKYYYKKX..............', // 20 flaring sigil
  '..............XYYYOOYYYYYCVV%%%%%%VVSYYYYYYKYYYKKX..............', // 21
  '........XXXX..XYYYOOYYYYYCVVV%%%%VVVSYYYYYYKYYYKKX..............', // 22
  '........XCVX..XYYYOOYYYYYCVVVV%%VVVVSYYYYYYKYYYKKX..............', // 23
  '........XVVX..XYYOOOYYYYYCVVVVVVVVVVSYYYYYOOOYYKKX..............', // 24
  '........XXXX..XYYYOOOYYYYCVVVVVVVVVVSYYYYOOOYYYKKX..XXXX........', // 25
  '..............XYYYYOOOOYYCVVVVVVVVVVSYYOOOOYYYYKKX..XCVX........', // 26
  '..............XYYYYYYOOOOOOPPPPPPPPOOOOOOYYYYYYKKX..XVVX........', // 27
  '..............XYYYYYYYKKKKKPPPPPPPPKKKKKYYYYYYYKKX..XXXX........', // 28
  '..............XYYYOOYYYYYCVKKKKKKKKVSYYYYYYKYYYKKX..............', // 29
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 30
  '......%.......XYSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBYX..............', // 31
  '..............XYBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSBSYX..............', // 32
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX........*.....', // 33
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 34
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 35
  '..............XYYYOOYYYYXKCKCKCKCKCKCKKXYYYKYYYKKX..............', // 36 open drawer, cards showing
  '..............XYYYOOYYYYXKKKKKKKKKKKKKKXYYYKYYYKKX..............', // 37
  '...XXXX.......XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 38
  '...XCVX.......XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 39
  '...XVVX.......XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 40
  '...XXXX.......XYYYOOYYYYXNZZCCCCCCCCZZNXYYYKYYYKKX.......XXXX...', // 41
  '..............XYYYOOYYYYXNZZZZZGGZZZZZNXYYYKYYYKKX.......XCVX...', // 42
  '..............XYYYOOYYYYXZZZZZZZZZZZZZZXYYYKYYYKKX.......XVVX...', // 43
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX.......XXXX...', // 44
  '..............XYYYOOYYYYXNNNNNNNNNNNNNNXYYYKYYYKKX..............', // 45
  '...........*..XYYYOOYYYYXNZZCCCCCCCCZZNXYYYKYYYKKX..............', // 46
  '..............XYYYOOYYYYXNZZZZZGGZZZZZNXYYYKYYYKKX..............', // 47
  '..............XYYYOOYYYYXZZZZZZZZZZZZZZXYYYKYYYKKX..............', // 48
  '..............XYYYOOYYYYXXXXXXXXXXXXXXXXYYYKYYYKKX..............', // 49
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX....%.........', // 50
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 51
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 52
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 53
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 54
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 55
  '..............XYYYOOYYYYYCVVVVVVVVVVSYYYYYYKYYYKKX..............', // 56
  '..............XYYYOOYYYYYYCVVVVVVVVSYYYYYYYKYYYKKX..............', // 57
  '..............XOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOKKKX..............', // 58
  '..............XYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYKKKX..............', // 59
  '..............XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..............', // 60
  '................................................................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF182_GRID = REF182_GRID;
