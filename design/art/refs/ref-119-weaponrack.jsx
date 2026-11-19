// SPDX-License-Identifier: MIT
// REF-119 - Weapon rack, Act 6 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/weapon_rack.png.
//
// Three finished swords, points down, in a brass-wood rack: aged-paper
// and stone-grey blades, old-gold guards, bronze grips. What the Forge
// makes. Bottom-centre anchor; solid over the rack.
//
// Legend: X coalblack  V aged paper  S stone grey  G old gold
//         L brass leaf  N antique brass  Z bronze

const REF119_GRID = [
  '................................', //  0
  '........XXXX..XXXX..XXXX........', //  1 pommels
  '........XGNX..XGNX..XGNX........', //  2
  '........XZZX..XZZX..XZZX........', //  3
  '........XZZX..XZZX..XZZX........', //  4
  '........XZZX..XZZX..XZZX........', //  5
  '......XXXXXXXXXXXXXXXXXXXX......', //  6 guards
  '......XLGGGGXLGGGGXLGGGGGX......', //  7
  '...XXXXXXXXXXXXXXXXXXXXXXXXXX...', //  8
  '...XXXXXXVSXXXXVSXXXXVSXXXXXX...', //  9 top rail
  '...XGGGGXVSXGGXVSXGGXVSXGGGGX...', // 10
  '...XNNNNXVSXNNXVSXNNXVSXNNNNX...', // 11
  '...XXXXXXVSXXXXVSXXXXVSXXXXXX...', // 12
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 13
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 14
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 15
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 16
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 17
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 18
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 19
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 20
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 21
  '...XZX..XVSX..XVSX..XVSX..XZX...', // 22
  '...XXXXXXVSXXXXVSXXXXVSXXXXXX...', // 23 bottom rail
  '...XGGGGXVSXGGXVSXGGXVSXGGGGX...', // 24
  '...XNNNNXVSXNNXVSXNNXVSXNNNNX...', // 25
  '...XXXXXXVSXXXXVSXXXXVSXXXXXX...', // 26
  '...XZX..XXXX..XXXX..XXXX..XZX...', // 27
  '...XZX...XX....XX....XX...XZX...', // 28
  '...XXX....................XXX...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF119_GRID = REF119_GRID;
