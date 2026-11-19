// SPDX-License-Identifier: MIT
// REF-134 - Lectern, Act 4 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Not wired yet.
//
// A brass-wood lectern on a single bronze post, with an open tome:
// parchment pages, stone-grey lines, two teal initials and a wine
// ribbon marker. Also the Librarian's lectern for Act 9. Bottom-centre
// anchor; solid at the post.
//
// Legend: X coalblack  N antique brass  G old gold  Z bronze
//         C parchment cream  S stone grey  I bright teal  W wineflesh

const REF134_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '......XXXXXXXXXXXXXXXXWWXX......', //  4 open tome
  '......XCCCCCCCCXXCCCCCWWCX......', //  5
  '......XCSISSSSCXXCSSSSWWCX......', //  6
  '......XCCCCCCCCXXCCCCCWWCX......', //  7
  '......XCSSSSSSCXXCSISSWWCX......', //  8
  '......XCCCCCCCCXXCCCCCWWCX......', //  9
  '.....XXCSSSSCCCXXCSSSSWWCXX.....', // 10 desk
  '.....XXXXXXXXXXXXXXXXXWWXXX.....', // 11
  '.....XNNNNNNNNNNNNNNNNWWNNX.....', // 12
  '.....XNNNNNNNNNNNNNNNNXWNNX.....', // 13
  '.....XZZZZZZZZZZZZZZZZZXZZX.....', // 14 post
  '.....XXXXXXXXXXXXXXXXXXXXXX.....', // 15
  '..............XNZX..............', // 16
  '..............XNZX..............', // 17
  '..............XNZX..............', // 18
  '..............XNZX..............', // 19
  '..............XNZX..............', // 20
  '..............XNZX..............', // 21
  '..............XNZX..............', // 22
  '..............XNZX..............', // 23
  '..............XNZX..............', // 24
  '..............XNZX..............', // 25
  '.........XXXXXXXXXXXXXX.........', // 26 foot
  '.........XNNNNNNNNNNNNX.........', // 27
  '.........XZZZZZZZZZZZZX.........', // 28
  '.........XZZZZZZZZZZZZX.........', // 29
  '.........XXXXXXXXXXXXXX.........', // 30
  '................................', // 31
];

window.REF134_GRID = REF134_GRID;
