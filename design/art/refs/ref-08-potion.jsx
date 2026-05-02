// REF-08 — Healing Potion icon, 16×16.
// Glass bottle, cork stopper, alarm scarlet liquid, parchment cream label.
// Highlight upper-left for roundness.

const REF08_GRID = [
  '................',
  '......XXXX......',  //  cork top
  '.....XZNZNX.....',
  '.....XZNZNX.....',  //  cork
  '....XX....XX....',  //  neck
  '....XCCXX#C....',  // glass neck (C cream label hint, X outline, # specular)
  '...X........X...',
  '..X##........X..',  // glass body — top hi
  '..X#!!!!!!!!X...',  // liquid begins
  '..X##!!!!!!!X...',
  '..X!!!!!!!!!X...',
  '..X!CCCCCCC!X...',  // parchment label band
  '..X!CKKKKKC!X...',  // label inner
  '..X!!!!!!!!!X...',
  '..XX!!!!!!!XX...',
  '...XXXXXXXXXX...',
];

// Pad to exact 16-char rows
const REF08_NORM = REF08_GRID.map(r => (r.length < 16 ? r + '.'.repeat(16-r.length) : r.slice(0,16)));

window.REF08 = function REF08({ scale = 16 }) {
  return <PixelArt grid={REF08_NORM} scale={scale} title="REF-08 Healing potion" />;
};
window.REF08_GRID = REF08_NORM;
window.REF08_ROLES = ['X','Z','N','C','#','!','K'];
