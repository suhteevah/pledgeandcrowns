// SPDX-License-Identifier: MIT
// REF-34 v2 - The Surveyor on the Guildhall kit, 32x32, idle, transparent bg.
// DRAFT - pending Matt approval on the Design canvas. NOT wired into the game.
//
// Kit: beret, aged-paper ruff, long oxblood coat (REF-69). A field officer:
// plain beret, brass collar studs, gold belt buckle, pine hem, fair hair.
// Lesson prop (tuple_struct): a measuring rod as tall as the frame, a plain
// bronze length made meaningful by its evenly spaced old-gold ticks, with the
// one brass-leaf specular on the topmost tick - the unit that names the number.
//
// Legend: X coalblack  Y crypt  O oxblood  W wineflesh  K inkblood
//         P pink quartz  R dusty rose  V aged paper  C cream  B basalt
//         G old gold  L brass leaf  N antique brass  Z bronze  F pine

const REF34_V2_GRID = [
  '..........................XXX...', //  0 rod cap
  '..........................XLX...', //  1 top tick specular
  '.........XXXXXXXXXXXX.....XZX...', //  2 beret
  '........XOOWWWWOOOOOYX....XZX...', //  3
  '........XOWWWOOOOOOOYX....XGX...', //  4 tick
  '.........XXXXXXXXXXXXX....XZX...', //  5 brim
  '..........XNPPPPPPRNX.....XZX...', //  6 fair hair
  '..........XNPXPPPXRNX.....XZX...', //  7 eyes
  '..........XPPPPRPPRWX.....XGX...', //  8 tick
  '..........XXPRRRRRWXX.....XZX...', //  9
  '.........XVVVVCCVVVVVX....XZX...', // 10 ruff
  '.......XXOOVVVVVVVVOOYXX..XZX...', // 11
  '......XOOWOONOOOOONOOOYYX.XGX...', // 12 collar studs, tick
  '......XOWWOOOOOOOOOOOOYYX.XZX...', // 13
  '......XOWWOOOOOOOOOOOOYYXPXZX...', // 14 hand reaches the rod
  '......XOWOOOOOOOOOOOOOYYXRPPX...', // 15 grip
  '......XPXOOOOOOOOOOOOOYYXXXGX...', // 16 tick
  '......XRXOOWOOOOOOOOOOYYX.XZX...', // 17
  '.......XXNNNNNGLNNNNNNXX..XZX...', // 18 belt
  '.......XOOWOOOOOOOOOOYYX..XZX...', // 19
  '.......XOOWOOOOOOOOOOYYX..XGX...', // 20 tick
  '......XOOWWOOOOOOOOOOOYYX.XZX...', // 21
  '......XOOWOOOOOOOOOOOOYYX.XZX...', // 22
  '......XFFFFFFFFFFFFFFFFFX.XZX...', // 23 pine hem
  '......XXXXXXXXXXXXXXXXXXX.XGX...', // 24 tick
  '.........XYYYX...XYYYX....XZX...', // 25
  '.........XYYKX...XYYKX....XZX...', // 26
  '.........XBBBX...XBBBX....XZX...', // 27
  '........XBBBBX...XBBBBX...XNX...', // 28 rod foot
  '........XXXXXX...XXXXXX...XXX...', // 29
  '................................', // 30
  '................................', // 31
];

window.REF34_V2_GRID = REF34_V2_GRID;
