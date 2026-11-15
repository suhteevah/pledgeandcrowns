// SPDX-License-Identifier: MIT
// REF-65 v2 - The Ghostkeeper on the Vault kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: the Vault guard (REF-74) as a shade: the same crested helm, pauldrons
// and plate, drained to aged paper and stone grey with mist-teal where the
// gold used to be. Lesson prop (weak_ref): it has no legs - below the tassets
// it thins to a wisp. It can point at the hoard but does not hold it up.
//
// Why v2: the first sprite had no dark outline at all and vanished against
// the pale plaza flagstone. This one keeps a Basalt outline - softer than the
// Coalblack every other villager has, so it still reads as faded, but dark
// enough to hold an edge on grass, cobble and flagstone alike.
//
// Legend: B basalt (outline, visor)  V aged paper  C parchment cream
//         S stone grey  M mist teal  P pink quartz  R dusty rose

const REF65_V2_GRID = [
  '................................', //  0
  '..............BMMB..............', //  1 crest
  '..............BVVB..............', //  2
  '...........BBBBVVBBBB...........', //  3
  '..........BVCCVVVVVVSB..........', //  4 helm
  '..........BVCVVVVVVVSB..........', //  5
  '..........BMMMMMMMMMMB..........', //  6 brow band
  '..........BVBBBBBBBBSB..........', //  7 visor slit
  '..........BVVVVVVVVVSB..........', //  8
  '..........BVVVVVVVVSSB..........', //  9
  '...........BVVVVVVSSB...........', // 10
  '........BBBBBBBBBBBBBBBB........', // 11
  '.......BMMMBVCVVVVVSBMMMB.......', // 12 pauldrons
  '......BMCMMBVCVVMVVSBMMMSB......', // 13
  '......BMMMSBVVVMCMVSBMMSSB......', // 14
  '......BBBBBBVVVVMVVSBBBBBB......', // 15
  '......BVVSBBVVVVVVSSBBVSSB......', // 16
  '......BVVSBBMMMMMMMMBBVSSB......', // 17 belt
  '......BCVSBBVVVVVVSSBBVSSB......', // 18
  '......BPPRBBVVVVVVSSBBPPRB......', // 19 faint hands
  '......BBBBBBVVVVVVSSBBBBBB......', // 20
  '...........BVVVVVVSSB...........', // 21 thinning
  '...........BVVVVVSSSB...........', // 22
  '............BVVVVSSB............', // 23
  '............BVVVSSSB............', // 24
  '.............BVVSSB.............', // 25
  '.............BVSSB..............', // 26
  '..............BVSB..............', // 27
  '..............BSB...............', // 28
  '...............B................', // 29 wisp
  '................................', // 30
  '................................', // 31
];

window.REF65_V2_GRID = REF65_V2_GRID;
