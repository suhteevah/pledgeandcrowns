// SPDX-License-Identifier: MIT
// REF-31 v2 - The Recruiter on the Guildhall kit, 32x32, idle, transparent bg.
// APPROVED by Matt 2026-11-15. Wired as game/assets/sprites/npc/recruiter_idle_0.png.
//
// Kit: oxblood cap, aged-paper ruff, long oxblood coat (REF-69). A clerk, so a
// plain round cap with no badge, brass collar studs and a pine hem instead of
// the Guildmaster's gold. Lesson prop (assoc_new): the open recruit-roll held
// across the chest with three written lines and a fresh old-gold seal, and a
// quill raised in the frame-right hand. A name goes in, a member comes out.
//
// Legend: X coalblack  Y crypt  O oxblood  W wineflesh  K inkblood
//         P pink quartz  R dusty rose  B basalt  V aged paper  C cream
//         G old gold  L brass leaf  N antique brass  F pine

const REF31_V2_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXXXX............', //  2 round cap
  '.........XOWWWOOOOOYX...........', //  3
  '.........XOWWOOOOOOYX...........', //  4
  '.........XXXXXXXXXXXXX..........', //  5 brim
  '..........XBPPPPPPRBX...........', //  6 dark hair
  '..........XBPXPPPXRBX...........', //  7 eyes
  '..........XPPPPRPPRWX...........', //  8
  '..........XXPRRRRRWXX...........', //  9
  '.........XVVVVCCVVVVVX..........', // 10 ruff
  '.......XXOOVVVVVVVVOOYXX.....X..', // 11 quill tip
  '......XOOWOONOOOOONOOOYYX...XL..', // 12 collar studs, nib specular
  '......XOWXXXXXXXXXXXXXXYX..XX...', // 13 roll top edge
  '......XOXCCCCCCCCCCCCCXYXPX.....', // 14 roll
  '......XPXCXXXXXCCXXXXCXPXRX.....', // 15 written lines, hands
  '......XRXCCCCCCCCCCCCCXRXXX.....', // 16
  '......XXXCXXXXXXXCCCCCXXX.......', // 17
  '.......XXCCCCCCCCGGCCCXX........', // 18 fresh seal
  '.......XOXVVVVVVVGGVVXYX........', // 19 roll curl
  '.......XOOXXXXXXXXXXXOYX........', // 20
  '......XOOWWOOOOOOOOOOOYYX.......', // 21 coat skirt
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

window.REF31_V2_GRID = REF31_V2_GRID;
