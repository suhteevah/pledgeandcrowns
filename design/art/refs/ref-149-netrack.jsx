// SPDX-License-Identifier: MIT
// REF-149 - Net rack, Act 7 district prop, 32x32.
// APPROVED by Matt 2026-11-18. Wired as game/assets/sprites/props/net_rack.png.
//
// A hayfield fishing net hung to dry on a bronze frame, three wine
// floats along its lower edge. Open mesh, so the dock shows through.
// Bottom-centre anchor; solid only at the two posts.
//
// Legend: X coalblack  Z bronze  N antique brass  Q hayfield
//         W wineflesh  R dusty rose  O oxblood

const REF149_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '................................', //  3
  '...XXX....................XXX...', //  4
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', //  5 top rail
  '..XNNNNNNNNNNNNNNNNNNNNNNNNNNX..', //  6
  '..XXXXXXXXXXXXXXXXXXXXXXXXXXXX..', //  7
  '...XZX..Q...Q...Q...Q...Q.XZX...', //  8 net
  '...XZX.Q.Q.Q.Q.Q.Q.Q.Q.Q.QXZX...', //  9
  '...XZXQ...Q...Q...Q...Q...XZX...', // 10
  '...XZX.Q.Q.Q.Q.Q.Q.Q.Q.Q.QXZX...', // 11
  '...XZX..Q...Q...Q...Q...Q.XZX...', // 12
  '...XZX.Q.Q.Q.Q.Q.Q.Q.Q.Q.QXZX...', // 13
  '...XZXQ...Q...Q...Q...Q...XZX...', // 14
  '...XZX.Q.Q.Q.Q.Q.Q.Q.Q.Q.QXZX...', // 15
  '...XZX..Q...Q...Q...Q...Q.XZX...', // 16
  '...XZX.Q.Q.Q.Q.Q.Q.Q.Q.Q.QXZX...', // 17
  '...XZXQ...Q...Q...Q...Q...XZX...', // 18
  '...XZX.Q.Q.Q.Q.Q.Q.Q.Q.Q.QXZX...', // 19
  '...XZX..QX..Q...Q...QX..Q.XZX...', // 20 floats
  '...XZX.QRWWQ.Q.Q.Q.QRWWQ.QXZX...', // 21
  '...XZXQXWWWX..QX..QXWWWX..XZX...', // 22
  '...XZX..WWOQ.QRWWQ.QWWO...XZX...', // 23
  '...XZX...X..QXWWWX..QX....XZX...', // 24
  '...XZX........WWO.........XZX...', // 25
  '...XZX.........X..........XZX...', // 26
  '...XZX....................XZX...', // 27
  '...XZX....................XZX...', // 28
  '...XXX....................XXX...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF149_GRID = REF149_GRID;
