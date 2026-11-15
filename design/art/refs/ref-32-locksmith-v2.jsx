// SPDX-License-Identifier: MIT
// REF-32 v2 - The Locksmith on the Guildhall kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/locksmith_idle_0.png.
//
// Kit: flat cap, aged-paper ruff, long oxblood coat (REF-69). A tradesman, so
// the cap is the darker crypt cloth, with a basalt moustache and a pine hem.
// Lesson prop (if_let): one old-gold key raised clear above the head in the
// frame-right hand, the highest and brightest point of the sprite, while two
// more keys hang ignored from a bronze ring at the frame-left hip.
//
// Legend: X coalblack  Y crypt  O oxblood  W wineflesh  K inkblood
//         P pink quartz  R dusty rose  S stone grey  B basalt  V aged paper
//         C cream  G old gold  L brass leaf  N antique brass  Z bronze  F pine

const REF32_V2_GRID = [
  '.........................XGX....', //  0 key bow
  '........................XGLGX...', //  1
  '.........XXXXXXXXXXXX...XG.GX...', //  2 flat cap top
  '........XYYOOOOYYYYYKX...XGX....', //  3
  '........XYOOOYYYYYYYKX...XNX....', //  4 key shaft
  '.........XXXXXXXXXXXXX...XNX....', //  5 brim
  '..........XSPPPPPPRSX....XNX....', //  6
  '..........XSPXPPPXRSX...XNGGX...', //  7 key bit
  '..........XPPPPRPPRWX...XPPPX...', //  8 raised hand
  '..........XXBBBBBBWXX...XRRPX...', //  9 moustache
  '.........XVVVVCCVVVVVX..XOOX....', // 10 ruff, raised sleeve
  '.......XXOOVVVVVVVVOOYXXOOX.....', // 11
  '......XOOWOONOOOOONOOOYYOX......', // 12
  '......XOWWOOOOOOOOOOOOYYX.......', // 13
  '......XOWWOOOOOOOOOOOOYYX.......', // 14
  '......XOWOOOOOOOOOOOOOYYX.......', // 15
  '......XPXOOOOOOOOOOOOOYX........', // 16 hip hand
  '.....XZRZXOWOOOOOOOOOOYX........', // 17 key ring
  '....XGXXGXNNNNGLNNNNNNXX........', // 18 hanging keys, belt
  '....XGX.XGXOWOOOOOOOOYYX........', // 19
  '....XNX.XNXOWOOOOOOOOYYX........', // 20
  '......XOOWWOOOOOOOOOOOYYX.......', // 21
  '......XOOWOOOOOOOOOOOOYYX.......', // 22
  '......XFFFFFFFFFFFFFFFFFX.......', // 23 pine hem
  '......XXXXXXXXXXXXXXXXXXX.......', // 24
  '.........XYYYX...XYYYX..........', // 25
  '.........XYYKX...XYYKX..........', // 26
  '.........XBBBX...XBBBX..........', // 27
  '........XBBBBX...XBBBBX.........', // 28
  '........XXXXXX...XXXXXX.........', // 29
  '................................', // 30
  '................................', // 31
];

window.REF32_V2_GRID = REF32_V2_GRID;
