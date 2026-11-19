// SPDX-License-Identifier: MIT
// REF-104 - Village well, Act 1 Hearthstone Village prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/well.png.
//
// A stone-grey ring with a lit aged-paper lip and deep-teal water, two
// bronze posts, a small wine roof that matches the cottages, and a
// bucket on a brass rope. Bottom-centre anchor; solid over the ring.
//
// Legend: X coalblack  S stone grey  B basalt  V aged paper
//         C parchment cream  Z bronze  N antique brass
//         W wineflesh  O oxblood  Y crypt  R dusty rose
//         D deep teal  T main teal  I bright teal

const REF104_GRID = [
  '................................', //  0
  '................................', //  1
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', //  2 roof
  '...XRRWWWWWWWWWWWWWWWWWWWWWWX...', //  3
  '...XOOOOOOOOOOOOOOOOOOOOOOOOX...', //  4
  '...XYYYYYYYYYYYYYYYYYYYYYYYYX...', //  5
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', //  6
  '......XZX......N.......XZX......', //  7
  '......XZX......N.......XZX......', //  8
  '......XZX......N.......XZX......', //  9
  '......XZX......N.......XZX......', // 10
  '......XZX......N.......XZX......', // 11
  '......XZX......N.......XZX......', // 12
  '......XZX...XXXXXXXX...XZX......', // 13 bucket
  '......XZX...XNZZZZZX...XZX......', // 14
  '......XZX...XNNNNNNX...XZX......', // 15
  '......XZX...XZZZZZZX...XZX......', // 16
  '......XZX...XXXXXXXX...XZX......', // 17
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 18 stone ring
  '....XCCVXXXXXXXXXXXXXXXXVVVX....', // 19
  '....XSSSXDDDTIDDDDTDDDDXSSSX....', // 20
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 21
  '....XVSSSSSBSSSSSSSBSSSSSSBX....', // 22
  '....XVSSSSSBSSSSSSSBSSSSSSBX....', // 23
  '....XVSSSSSBSSSSSSSBSSSSSSBX....', // 24
  '....XVBBBBBBBBBBBBBBBBBBBBBX....', // 25
  '....XVSSBSSSSSSBSSSSSSBSSSBX....', // 26
  '....XVSSBSSSSSSBSSSSSSBSSSBX....', // 27
  '....XVSSBSSSSSSBSSSSSSBSSSBX....', // 28
  '....XXXXXXXXXXXXXXXXXXXXXXXX....', // 29
  '................................', // 30
  '................................', // 31
];

window.REF104_GRID = REF104_GRID;
