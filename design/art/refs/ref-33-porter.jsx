// SPDX-License-Identifier: MIT
// REF-33 - The Porter, 32x32, idle, transparent bg.
// A Crypt-tunic labourer drains a stack of bog-umber crates: one crate is
// lifted in both hands off the top of the pile to his right. Pop until the
// stack is empty - `while let Some(crate) = stack.pop()`.
// Mission: while_let - loop popping while a variant remains.
//
// Legend:
//   X coalblack (outline, crate edges)
//   Y crypt (tunic deep)   O oxblood (tunic primary)   W wineflesh (lit)
//   R dusty rose (skin mid)   P pink quartz (skin hi)   B basalt (brow/eyes)
//   S stone grey (hair)   V aged paper (hair hi)
//   U bog umber (crates body)   Z bronze (crate banding, boots)
//   N antique brass (crate corner brackets)   G old gold (lift crate strap)
//   F pine (tunic hem)
//   . transparent

const REF33_GRID = [
  '................................', //  0
  '................................', //  1
  '..........XXXXXXXX..............', //  2 head top
  '.........XSSSVVSSSX.............', //  3 hair
  '........XSSVVVVVVSSX............', //  4
  '........XSPPPPPPPPSX.XUUUUUX....', //  5 forehead | lifted crate top
  '........XPPPPPPPPRRX.XUNUUNUX...', //  6 lifted crate (corner brackets N)
  '........XPBPPPPBPRWX.XUUGGUUX...', //  7 brow / eyes | crate strap (G)
  '........XPPRRRRRRRPX.XUNUUNUX...', //  8
  '........XXPRRRRRRPXX.XUUUUUX....', //  9 jaw | lifted crate base
  '........XYPPPRRRRPYX............', // 10 collar
  '.......XYYYYYYYYYYYYX...........', // 11 shoulders
  '......XYYYOOOOOOOOYYYPPPP.......', // 12 robe upper | right arms reach
  '......XYOOOWWWWOOOOYPRRPX.......', // 13 hands grip crate
  '......XYOOWWWWWWOOOYPPPPXX......', // 14 | stack top crate begins
  '....PPPYOOOOOOOOOOYOX.XUUUUUX...', // 15 left hand on hip | stack crate 1
  '....PRPYOOOOOOOOOYYOX.XUNUUNUX..', // 16
  '....PPPYOOOOOOOOOYYOX.XUUUUUUX..', // 17
  '......XXYYOOOOOOOOYYXX.XUNUUNUX.', // 18 | stack crate 1 base
  '......XYYOOOWWWWOOOYYX.XUUUUUUX.', // 19 stack crate 2 top
  '......XYYOOOOOOOOOOYYX.XUNUUNUX.', // 20
  '......XXXNZZZZZZZZNXXX.XUUUUUUX.', // 21 belt | stack crate 2 mid
  '......XYYOOOOOOOOOOYYX.XUNUUNUX.', // 22 robe lower | crate 2 base
  '......XYYOOOWWWWOOOYYX.XUUUUUUX.', // 23 | stack crate 3 top
  '......XYYOOOOOOOOOOYYX.XUNUUNUX.', // 24
  '......XYYOOOOOOOOOOYYX.XUUUUUUX.', // 25
  '......XYFFFFFFFFFFFFYX.XUNUUNUX.', // 26 hem | crate 3 base
  '......XFFFFXX..XXFFFFX.XXXXXXXX.', // 27 leg split | stack floor
  '......XZZZZX....XZZZZX..........', // 28 boots
  '......XZUUZX....XZUUZX..........', // 29
  '......XXXXXX....XXXXXX..........', // 30 soles
  '................................', // 31
];

window.REF33 = function REF33({ scale = 8 }) {
  return <PixelArt grid={REF33_GRID} scale={scale} title="REF-33 The Porter" />;
};
window.REF33_GRID = REF33_GRID;
window.REF33_ROLES = ['X','Y','O','W','R','P','B','S','V','U','Z','N','G','F'];
