// SPDX-License-Identifier: MIT
// REF-65 - The Ghostkeeper, 32x32, idle, transparent bg.
// A faded, half-there figure rendered in low-contrast aged-paper and stone
// grey with a Cobalt wash - he reads as ghostly, NON-owning. From one open
// hand a thin dashed Cobalt tether runs out to a hoard chest at frame-right
// that has gone faint, its outline broken, half-vanished. He can point at
// the hoard, can ask whether it is still there, but he does not own it and
// cannot keep it alive; when the last real owner drops it, his tether goes
// slack and the hoard is simply gone. That is `Weak`: a non-owning weak
// reference you must `upgrade()` to use, and which may already be None.
// Faded Crypt tunic (mostly V/S low-contrast), Cobalt tether, no magic.
// Mission: weak_ref - hold a non-owning Weak reference.
//
// Legend:
//   X coalblack (sparse outline)   B basalt (faint brow/eyes)
//   S stone grey (faded form mid)   V aged paper (faded form hi)
//   Y crypt (ghostly tunic deep, used sparingly)
//   R dusty rose (faint skin)      P pink quartz (faint skin hi)
//   < cobalt deep (tether shadow, vanished-hoard outline)
//   > cobalt (dashed tether, faint hoard glow)
//   Z bronze (faint boots)         G old gold (one ghost of a coin, faded)
//   . transparent
//
// The thin dashed Cobalt tether (> with gaps, < shadow) from his open hand
// to the half-vanished hoard (broken < outline, one faded G coin) is the
// Weak reference: it points but does not own, and the hoard it points to
// may already be gone - upgrade() may return None.

const REF65_GRID = [
  '................................', //  0
  '..........VVSSSSVV..............', //  1 head top (faded, no hard outline)
  '.........VSSVVVVSSV.............', //  2 hair faint
  '.........VSVVVVVVSV.............', //  3
  '.........VSPPRRPPSV.............', //  4 forehead faint skin
  '.........VPPRRRRPPV.............', //  5
  '.........VPBPPPBPRV.............', //  6 faint brow/eyes (B)
  '.........VPPRRRRRPV.............', //  7
  '..........VPRRRRPV..............', //  8 jaw faint
  '..........VSPRRPSV..............', //  9 collar faint
  '.........VSSVVVVSSV.............', // 10 shoulders faded
  '........VSYYVVVVYYSV............', // 11 tunic upper faint (Y sparse)
  '.......VSYVVVVVVVVYSV.>.........', // 12 tunic | tether starts (>)
  'PPRP..VSYVVVVVVVVVYSV..>........', // 13 open hand (P) | dashed tether
  'PPRPVSYVVVVVVVVVVVYSV...>.<.....', // 14 hand holds tether | gap, hoard top
  '.PPVSYVVVVVVVVVVVVYSV....<.<....', // 15 tether dashes | faded hoard rim
  '....VSYVVVVVVVVVVYSV....<GG.<...', // 16 | faded coin (G) inside hoard
  '....VSYVVVVVVVVVYSV.....<G.>....', // 17 | hoard half-gone (broken <)
  '....VSSYVVVVVVVYSSV......>.<....', // 18 | one wall of hoard vanished
  '....VSSVVVVVVVVVSSV......<.<....', // 19 widest faded
  '....VSVVVVVVVVVVVSV.........<...', // 20 | hoard base broken
  '....VSVFFVVVVVFFVSV.............', // 21 hem faint (no pine - keep ghostly)
  '....VSVVVXX..XXVVVSV............', // 22 leg split faint
  '....VSZZVX....XVZZSV............', // 23 boots faint (Z)
  '.....VZZV......VZZV.............', // 24
  '.....VVVV......VVVV.............', // 25 faint soles
  '................................', // 26
  '................................', // 27
  '................................', // 28
  '................................', // 29
  '................................', // 30
  '................................', // 31
];

window.REF65 = function REF65({ scale = 8 }) {
  return <PixelArt grid={REF65_GRID} scale={scale} title="REF-65 The Ghostkeeper" />;
};
window.REF65_GRID = REF65_GRID;
window.REF65_ROLES = ['X','B','S','V','Y','R','P','<','>','Z','G'];
