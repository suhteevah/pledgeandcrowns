// REF-15 — The Cooper, 32×32, idle, transparent bg.
// Burly barrel-maker beside a small bound barrel. Wood + iron, no magic.
// Mission: vec_iter — Vec is a barrel of items.

// Legend:
//   X coalblack (outline)        Y crypt (hair shadow)
//   B basalt (hair mid, adze handle)
//   S stone grey (barrel hoops, adze blade)
//   L brass leaf (hoop highlight pixel)
//   R dusty rose (skin mid)      W wineflesh (skin shadow)   P pink quartz (skin hi)
//   Q hayfield (undershirt cream)
//   U bog umber (vest deep, glove, trouser shadow)
//   Z bronze (vest mid, trousers, barrel staves shadow, belt)
//   N antique brass (vest hi, barrel staves mid)
//   G old gold (buckle, barrel lid)
//   . transparent

const REF15_GRID = [
  '................................', //  0
  '................................', //  1
  '................................', //  2
  '......XXXXXXXX..................', //  3 head top
  '.....XYYYBBBBYX.................', //  4
  '....XYBBBBBBBBYX................', //  5 hair
  '....XYBBBBBBBBBX................', //  6
  '....XPPPPPPRRWWX................', //  7 forehead
  '....X.PPRRRRRRWX................', //  8 face top
  '....XX.X.XX.X.XX................', //  9 eyes
  '....X.PPRRRRRRWX................', // 10
  '....XXPPPRRRRRWX................', // 11 chin
  '...XQQQQQQQQQQQQX...............', // 12 undershirt collar
  '..XZZZQQQQQQQQZZZX...XSSSSSSSX..', // 13 vest shoulders + barrel top
  '.XZNNZQQQQQQQQZNNZX..XGGGGGGGX..', // 14 vest highlights + barrel lid
  '.XZNNZQQQQQQQQZNNZX..XSSSSSSSX..', // 15 hoop top
  '.XZZZZQQQQQQQQZZZZX..XZNNZNNZX..', // 16 vest mid + barrel staves
  '.XZZZZQQQQQQQQZZZZX..XZNNZNNZX..', // 17
  '.XUUZZZZZZZZZZZZUUX..XSSSSSSSX..', // 18 vest lower + hoop mid
  '.XUUUUUUUUUUUUUUUUX..XZNNZNNZX..', // 19 vest deep
  '.XGGGGGGGGGGGGGGGGX..XZNNZNNZX..', // 20 belt
  '..XZZUZZGGGGZZUZZX...XSSSSSSSX..', // 21 belt buckle
  '..XZZZZZZZZZZZZZZX...XZNNZNNZX..', // 22 trouser tops
  '..XZUZZUUUUUUZZUZX...XZNNZNNZX..', // 23 trousers
  '..XUUZUUUUUUUUZUUX...XZNNZNNZX..', // 24                     left hand on lid
  '..XUUZUUUUUUUUZUUX..XPRRWX......', // 25                     skin near lid
  '..XUUUXXXXXXXXUUUX..XPPRWX......', // 26 leg split + glove pad
  '..XZZZX......XZZZX...XXXX.......', // 27 boot tops
  '..XZUZX......XZUZX..............', // 28 boot mid
  '..XXXXX......XXXXX..............', // 29 soles
  '................................', // 30
  '................................', // 31
];

window.REF15 = function REF15({ scale = 8 }) {
  return <PixelArt grid={REF15_GRID} scale={scale} title="REF-15 The Cooper" />;
};
window.REF15_GRID = REF15_GRID;
window.REF15_ROLES = ['X','Y','B','S','L','R','W','P','Q','U','Z','N','G'];
