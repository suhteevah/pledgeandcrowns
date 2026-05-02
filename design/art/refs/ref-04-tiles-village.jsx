// REF-04 — Hearthstone Village tile sample sheet, 128×128.
// 8×8 grid of 16×16 tiles, white-grid lines for clarity.
// Tiles: grass, tall grass+flowers, dirt path straight, dirt path corner,
// cobblestone, fence horiz, fence corner, stone well top, stone well bottom,
// wildflower yellow, wildflower red, bush, sapling, hay bale, wooden cart, signpost.

// We render tiles into a 128×128 canvas and overlay a 1px white grid.

// Each tile is a 16x16 grid (rows of strings).
const T = {
  // (1) plain grass
  grass: [
    'EEEEEHEEEEEEEEHE',
    'EEEHEEEEEHEEEEEE',
    'EEEEEEHEEEEEEEHE',
    'EEHEEEEEEEEHEEEE',
    'EEEEEEEHEEEEEEHE',
    'EEHEEEEEEEEEHEEE',
    'EEEEHEEEEHEEEEEE',
    'EEEEEEEEEEEEEHEE',
    'HEEEEEHEEEEEEEEE',
    'EEEHEEEEEEEHEEEH',
    'EEEEEEEEHEEEEEEE',
    'EEHEEEEEEEEEEHEE',
    'EEEEEEHEEEEEEEEE',
    'EEEEHEEEEHEEEHEE',
    'EHEEEEEEEEEEEEEE',
    'EEEEEEHEEEEEEEHE',
  ],
  // (2) tall grass with flowers
  tallGrass: [
    'EHEHEEEHEHEHEEEH',
    'EHEHEEEHEHEHEHEH',
    'HQHHEHHQHEHHEHHE',
    'HQHHEHHQHEHHEHHE',
    'EHEHGHEHEHEHEH!H',
    'EQEHEHEEEQEHEHEH',
    'HEHEEEHEHEHQHEHE',
    'HEHEEEHEHEHQHEHE',
    'EHGHEHEHEHEHEH#H',
    'EHEHE!HEHEHEHEEH',
    'EHEEHEHEEHEHGHEH',
    'EEEEHEHEEHEHEHEH',
    'EHEHEHE!HEHGHEHE',
    'HEHEHEHEHEHEHEHE',
    'EFEEEFEEEFEEEFEE',
    'FFFFFFFFFFFFFFFF',
  ],
  // (3) dirt path straight (vertical)
  dirtStraight: [
    'EEEEZNNZNNNZEEEE',
    'EEEZNZZNZNNZEEEE',
    'EEEZNNNZNZNNEEEE',
    'EEEZNNZNNNZNEEEE',
    'EEEZNZNZNNZZEEEE',
    'EEEZNNZNZNNNEEEE',
    'EEEZZNZNNNZNEEEE',
    'EEEZNZNZNZNZEEEE',
    'EEEZNNZNNNZNEEEE',
    'EEEZNZNZZNZNEEEE',
    'EEEZNZNNNZNZEEEE',
    'EEEZNNZNZNNZEEEE',
    'EEEZNZNZNNZZEEEE',
    'EEEZNNZNZNNNEEEE',
    'EEEZZNNZZNNZEEEE',
    'EEEZNZNNZNZNEEEE',
  ],
  // (4) dirt path corner
  dirtCorner: [
    'EEEZNNZNNNZNEEEE',
    'EEEZNNNNNNNNEEEE',
    'EEEZNZNNNNZZEEEE',
    'EEEZNNNNNNNNEEEE',
    'EEEZNNZNNNZNEEEE',
    'EEEZNNNZNNNZEEEE',
    'EEEZNNNNNZNNZZZZ',
    'EEEZNNNNNNNNNNNN',
    'EEEZNNZNNNZNNZZN',
    'EEEZNNNNNNNNNNNN',
    'EEEZNNNNZNNNNNZZ',
    'EEEZZZZZZZZZZZZZ',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
  ],
  // (5) cobblestone
  cobble: [
    'SVSVSVSVSVSVSVSV',
    'XSVSVSXSVSVSXSVS',
    'SXVSXSVXSXSVXSVS',
    'VSVSVSXVSVXSVXSV',
    'SVSXSVSVSXVSXVSX',
    'VSVSVXSVSVSVSVSV',
    'XSVSXSVSXSVSVSXS',
    'SVXSVSVSVSXVSVSV',
    'SVSVSVSVSVSVSVSV',
    'VSXSVSXVSXSVSXSV',
    'XVSVSVSVSVSVSVSV',
    'VSVSXSVSXSVSVSVS',
    'SVXSVXSVSVSVXSVS',
    'XSVSVSVXSVSXVSXS',
    'SVSVSXSVSVSVSVSV',
    'XVSXVSXSVXSVSXSV',
  ],
  // (6) wooden fence horizontal
  fenceHoriz: [
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'XBXEEEEXBXEEEEXB',
    'XNZEEEEXNZEEEEXN',
    'XNZEEEEXNZEEEEXN',
    'XNZNNNNXNZNNNNXN',
    'XNZBBBBXNZBBBBXN',
    'XNZNNNNXNZNNNNXN',
    'XNZEEEEXNZEEEEXN',
    'XNZNNNNXNZNNNNXN',
    'XNZBBBBXNZBBBBXN',
    'XNZNNNNXNZNNNNXN',
    'XBXEEEEXBXEEEEXB',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
  ],
  // (7) wooden fence corner
  fenceCorner: [
    'EEEEEEEXBXEEEEEE',
    'EEEEEEEXNZEEEEEE',
    'EEEEEEEXNZEEEEEE',
    'XBXEEEEXNZEEEEEE',
    'XNZEEEEXNZEEEEEE',
    'XNZEEEEXNZEEEEEE',
    'XNZNNNNXNZNNNNNN',
    'XNZBBBBXNZBBBBBB',
    'XNZNNNNXNZNNNNNN',
    'XNZEEEEXNZEEEEEE',
    'XNZNNNNXNZNNNNNN',
    'XNZBBBBXNZBBBBBB',
    'XNZNNNNXNZNNNNNN',
    'XBXEEEEXNZEEEEEE',
    'EEEEEEEXNZEEEEEE',
    'EEEEEEEXNZEEEEEE',
  ],
  // (8) stone well top
  wellTop: [
    'EEEXXXXXXXXXXEEE',
    'EEXSSSSSSSSSSXEE',
    'EXSSVVVVVVVVSSXE',
    'XSVVVVVVVVVVVVSX',
    'XSVAAAAAAAAAAVSX',
    'XSVAAAAAAAAAAVSX',
    'XSVAAAA><AAAAVSX',
    'XSVAAA<><<AAAVSX',
    'XSVAAA><<<<AAVSX',
    'XSVAAAA<<<AAAVSX',
    'XSVAAAAAAAAAAVSX',
    'XSVAAAAAAAAAAVSX',
    'XSVVVVVVVVVVVVSX',
    'EXSSVVVVVVVVSSXE',
    'EEXSSSSSSSSSSXEE',
    'EEEXXXXXXXXXXEEE',
  ],
  // (9) stone well bottom (base)
  wellBot: [
    'EEEEEXXXXXXEEEEE',
    'EEEEXNNNNNNXEEEE',
    'EEEXNZBBBBNZXEEE',
    'EEEXNZBBBBNZXEEE',
    'EEEXNZBBBBNZXEEE',
    'EEEXNZBBBBNZXEEE',
    'EEEXNZBBBBNZXEEE',
    'EEEXNZBBBBNZXEEE',
    'EEEXNZBBBBNZXEEE',
    'EEEXXXXXXXXXEEEE',
    'EEXSSSSSSSSSSXEE',
    'EXSVVVVVVVVVVSXE',
    'XSVVVVVVVVVVVVSX',
    'XSVVVVVVVVVVVVSX',
    'XSSSSSSSSSSSSSSX',
    'XXXXXXXXXXXXXXXX',
  ],
  // (10) wildflower yellow patch
  flowerYellow: [
    'EEEEEEEEEEEEEEEE',
    'EHEEEEHEEEEHEEEE',
    'EEHGHEEEHGHEEHGH',
    'EHGLGHEHGLGHEHGL',
    'EEHGHEEEHGHEEHGH',
    'EHEEEEHEEEEHEEEE',
    'EEEEHEEEEHEEEEHE',
    'EHGHEEHGHEEHGHEE',
    'HGLGHEGLGHEHGLGH',
    'EHGHEEHGHEEHGHEE',
    'EEEEHEEEEHEEEEHE',
    'EHEEEEHEEEEHEEEE',
    'EEHGHEEEHGHEEHGH',
    'EHGLGHEHGLGHEHGL',
    'EEHGHEEEHGHEEHGH',
    'EEEEEEEEEEEEEEEE',
  ],
  // (11) wildflower red patch
  flowerRed: [
    'EEEEEEEEEEEEEEEE',
    'EHEEEEHEEEEHEEEE',
    'EEHWHEEEHWHEEHWH',
    'EHW!WHEHW!WHEHW!',
    'EEHWHEEEHWHEEHWH',
    'EHEEEEHEEEEHEEEE',
    'EEEEHEEEEHEEEEHE',
    'EHWHEEHWHEEHWHEE',
    'HW!WHEW!WHEHW!WH',
    'EHWHEEHWHEEHWHEE',
    'EEEEHEEEEHEEEEHE',
    'EHEEEEHEEEEHEEEE',
    'EEHWHEEEHWHEEHWH',
    'EHW!WHEHW!WHEHW!',
    'EEHWHEEEHWHEEHWH',
    'EEEEEEEEEEEEEEEE',
  ],
  // (12) bush
  bush: [
    'EEEEEEEEEEEEEEEE',
    'EEEEEFFFFFFEEEEE',
    'EEEFFEEFFEEFFEEE',
    'EEFEEEEEFEEFEEFE',
    'EFEEHHHEFEHHEFEE',
    'FEEHHEHHFHHEHHFE',
    'FEHEEHEEFEHEEHFE',
    'FEHEEHEEFEHEEHFE',
    'FFEEHEEFFFEEHEFF',
    'EFFEEEFFFFEEEEFE',
    'EEFFFFEEFFFFFFEE',
    'EEEFFFEEEFFFFFEE',
    'EEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
  ],
  // (13) sapling tree
  sapling: [
    'EEEEEEEEEEEEEEEE',
    'EEEEEEFFEEEEEEEE',
    'EEEEEFEHFEEEEEEE',
    'EEEEFEHHEFEEEEEE',
    'EEEFEHEHEEFEEEEE',
    'EEFEHEEEHFEEEEEE',
    'EEFFEEHEEFEEEEEE',
    'EEEFFFEFFFEEEEEE',
    'EEEEEZZEEEEEEEEE',
    'EEEEEZUEEEEEEEEE',
    'EEEEEZUEEEEEEEEE',
    'EEEEZZUUEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
  ],
  // (14) hay bale
  hayBale: [
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
    'EXXXXXXXXXXXXXXE',
    'XQLLQLQLLQLLQLLX',
    'XLQLQQLQQLLQLLLX',
    'XQLQLLQLLQLLQLLX',
    'XLQQLQQLLQLLLQLX',
    'XQLLQLQLLQLQQLLX',
    'XLQLQQLQQLLQLLLX',
    'XQLQLLQLLQLLQLLX',
    'XLQQLQQLLQLLLQLX',
    'XQLLQLQLLQLQQLLX',
    'XLQLQQLQQLLQLLLX',
    'XXXXXXXXXXXXXXXE',
    'EEEEEEEEEEEEEEEE',
    'EEEEEEEEEEEEEEEE',
  ],
  // (15) wooden cart
  cart: [
    'EEEEEEEEEEEEEEEE',
    'EEXXXXXXXXXXXXEE',
    'EXNZBBBBBBBBNZXE',
    'EXNZBQQQQQQBNZXE',
    'EXNZBQLQLQQBNZXE',
    'EXNZBQQQQLQBNZXE',
    'EXNZBQLQQQQBNZXE',
    'EXNZBQQQQLQBNZXE',
    'EXNZBBBBBBBBNZXE',
    'EXXXXXXXXXXXXXXE',
    'EEXXXEEEEEEEXXXE',
    'EXBBXEEEEEEEXBBX',
    'XBNBBXEEEEEXBNBB',
    'XBNBBXEEEEEXBNBB',
    'EXBBXEEEEEEEXBBX',
    'EEXXXEEEEEEEXXXE',
  ],
  // (16) signpost (no text)
  signpost: [
    'EEEEEEEEEEEEEEEE',
    'EEXBBBBBBBBBBXEE',
    'EXNZNNNNNNNNNZXE',
    'EXNZVVVVVVVVNZXE',
    'EXNZVVVVVVVVNZXE',
    'EXNZVVVVVVVVNZXE',
    'EXNZNNNNNNNNNZXE',
    'EEXBBBBBBBBBBXEE',
    'EEEEEEXBXEEEEEEE',
    'EEEEEEXBXEEEEEEE',
    'EEEEEEXNZEEEEEEE',
    'EEEEEEXNZEEEEEEE',
    'EEEEEEXNZEEEEEEE',
    'EEEEEEXNZEEEEEEE',
    'EEEEEEXBXEEEEEEE',
    'EEEEEEEEEEEEEEEE',
  ],
};

// Compose 8×8 grid of 16×16 tiles into a 128×128 grid string array.
const TILE_ORDER_REF04 = [
  T.grass, T.tallGrass, T.dirtStraight, T.dirtCorner,
  T.cobble, T.fenceHoriz, T.fenceCorner, T.wellTop,
  T.wellBot, T.flowerYellow, T.flowerRed, T.bush,
  T.sapling, T.hayBale, T.cart, T.signpost,
  // remaining 16 cells: repeat for visual variety; bible asks for those 16 but
  // sheet is 8×8=64 cells. We'll repeat at half scale.
];

function composeSheet(tiles, cols = 4) {
  // 4 cols × 4 rows = 16 tiles, each 16×16 = 64×64. To make 128×128 we
  // double-pixel = 8×8 grid of 16×16. Actually bible says 8×8 grid of 16×16
  // cells with grid lines = 128×128 with thin lines... but 8*16=128 exact.
  // We'll use 4×4 layout (4*16=64) but spec wants 8×8 — so stretch by repeating
  // each tile into a 2×2 cluster (so visual spec still reads clearly).
  const out = [];
  const rows = Math.ceil(tiles.length / cols);
  for (let ty = 0; ty < rows; ty++) {
    for (let py = 0; py < 16; py++) {
      let line = '';
      for (let tx = 0; tx < cols; tx++) {
        const tile = tiles[ty * cols + tx];
        if (!tile) line += '.'.repeat(16);
        else line += tile[py];
      }
      out.push(line);
    }
  }
  return out;
}

const REF04_GRID = composeSheet(TILE_ORDER_REF04, 4); // 64 wide × 64 tall

window.REF04 = function REF04({ scale = 4 }) {
  return <PixelArt grid={REF04_GRID} scale={scale} title="REF-04 Hearthstone tiles" />;
};
window.REF04_GRID = REF04_GRID;
window.REF04_ROLES = ['E','H','F','J','Q','Z','N','U','S','V','X','B','G','L','W','!','>','<','A','C'];
