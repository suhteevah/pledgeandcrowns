// SPDX-License-Identifier: MIT
// REF-107 - Market stall, Market Row district landmark, 64x64.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/market_stall.png.
//
// The landmark of Market Row (the second Act 1 plaza): a wine-and-cream
// striped awning with a scalloped edge on two bronze posts, over a
// brass-topped counter. On the counter, left to right: two loaves, a
// tray of apples, two teal jars, a bolt of cobalt cloth. Stripes are 4
// px so they hold at game size. Bottom-centre anchor; solid from the
// counter's foot to the awning.
//
// Legend: X coalblack  W wineflesh  O oxblood  R dusty rose  Y crypt
//         C parchment cream  V aged paper  Z bronze  N antique brass
//         G old gold  L brass leaf  U bog umber
//         T main teal  I bright teal  < cobalt deep  > cobalt

const REF107_GRID = [
  '................................................................', //  0
  '................................................................', //  1
  '................................................................', //  2
  '................................................................', //  3
  '................................................................', //  4
  '................................................................', //  5
  '................................................................', //  6
  '................................................................', //  7
  '..........XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX..........', //  8 awning
  '..........XCRRRRCCCCRRRRCCCCRRRRCCCCRRRRCCCCRRRRCCCCRX..........', //  9
  '.........XCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWX.........', // 10
  '.........XCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWX.........', // 11
  '........XCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWX........', // 12
  '........XCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWX........', // 13
  '.......XCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWX.......', // 14
  '.......XCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWX.......', // 15
  '......XWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCX......', // 16
  '......XWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCX......', // 17
  '.....XWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCX.....', // 18
  '.....XWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCX.....', // 19
  '....XWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCCWWWWCCCX....', // 20
  '....XOOOVVVVOOOOVVVVOOOOVVVVOOOOVVVVOOOOVVVVOOOOVVVVOOOOVVVX....', // 21
  '....XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX....', // 22 scalloped edge
  '....XOOXXVVXXOOXXVVXXOOXXVVXXOOXXVVXXOOXXVVXXOOXXVVXXOOXXVVX....', // 23
  '.....XX..XXYYXXYYXXYYXXYYXXYYXXYYXXYYXXYYXXYYXXYYXXYYXX..XX.....', // 24
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 25
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 26
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 27
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 28
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 29
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 30
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 31
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 32
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXZX.......', // 33
  '.......XZXYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYXXXXXYYXZX.......', // 34 goods
  '.......XZXYYYYYYYYYYYYYYYYYYXXYXXYYYYXXYYYXXYYX>>>>>XYXZX.......', // 35
  '.......XZXYYYXXXXYYYXXXXYYYXRWXRWXYYXIIXYXIIXYX<><<>XYXZX.......', // 36
  '.......XZXYYXGGGNXYXGGGNXYYXWWXWWXYYXTIXYXTIXYX<<<<<XYXZX.......', // 37
  '.......XZXYYXNGNNXYXNGNNXYYXRWXWRXYYXTTXYXTTXYX<><<>XYXZX.......', // 38
  '.......XZXYYXXXXXXYXXXXXXYYXXXXXXXYYXXXXYXXXXYXXXXXXXYXZX.......', // 39
  '....XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX....', // 40 counter top
  '....XLLGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGX....', // 41
  '....XNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNX....', // 42
  '....XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX....', // 43 counter front
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 44
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 45
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 46
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 47
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 48
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 49
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 50
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 51
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 52
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 53
  '.....XZXNXZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUZZZZZUXNXZX.....', // 54
  '.....XUXNXUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUXNXUX.....', // 55
  '.....XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.....', // 56
  '................................................................', // 57
  '................................................................', // 58
  '................................................................', // 59
  '................................................................', // 60
  '................................................................', // 61
  '................................................................', // 62
  '................................................................', // 63
];

window.REF107_GRID = REF107_GRID;
