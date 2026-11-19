// SPDX-License-Identifier: MIT
// REF-160 - Key rack, Act 8 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/key_rack.png.
//
// Four long keys, two gold, one iron and one brass, hanging from a
// brass-wood rail on two bronze posts. Every Vault warden carries a
// ring-key; this is where they come from. Bottom-centre anchor; solid
// only at the two posts.
//
// Legend: X coalblack  Z bronze  N antique brass  G old gold
//         L brass leaf  S stone grey  V aged paper

const REF160_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '....XXX..................XXX....', //  4
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', //  5 rail
  '...XGGGGGGGGGGGGGGGGGGGGGGGGX...', //  6
  '...XNNNNNNNNNNNNNNNNNNNNNNNNX...', //  7
  '...XZZZZZZZZZZZZZZZZZZZZZZZZX...', //  8
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', //  9
  '....XZXXXX..XXX..XXX..XXXXZX....', // 10 key bows
  '....XZXLXGXXVXSXXLXGXXGXNXZX....', // 11
  '....XZXG.GXXS.SXXG.GXXN.NXZX....', // 12
  '....XZXXXX..XXX..XXX..XXXXZX....', // 13
  '....XZXXGX..XSX..XGX..XNXXZX....', // 14 key shanks
  '....XZXXGX..XSX..XGX..XNXXZX....', // 15
  '....XZXXGX..XSX..XGX..XNXXZX....', // 16
  '....XZXXGX..XSX..XGX..XNXXZX....', // 17
  '....XZXXGX..XSX..XGX..XNXXZX....', // 18
  '....XZXXGGX.XSSX.XGGX.XNNXZX....', // 19
  '....XZXXGXX.XSXX.XGXX.XNXXZX....', // 20
  '....XZXXGX..XSX..XGX..XNXXZX....', // 21
  '....XZXXGGX.XSSX.XGGX.XNNXZX....', // 22
  '....XZXXXXX.XXXX.XXXX.XXXXZX....', // 23
  '....XZX..................XZX....', // 24
  '....XZX..................XZX....', // 25
  '....XZX..................XZX....', // 26
  '....XZX..................XZX....', // 27
  '....XZX..................XZX....', // 28
  '....XXX..................XXX....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF160_GRID = REF160_GRID;
