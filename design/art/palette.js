// Heraldic Code palette v2.0 — 32 colors, role-named.
// Reference colors by role name (single char in pixel grids), never by hex.
window.PALETTE = {
  // Burgundy ramp — anchor (350°)
  K: { hex: '#1B0810', name: 'Inkblood',       group: 'burgundy' },
  Y: { hex: '#3E1220', name: 'Crypt',          group: 'burgundy' },
  O: { hex: '#6B1F35', name: 'Oxblood',        group: 'burgundy' },
  W: { hex: '#982D52', name: 'Wineflesh',      group: 'burgundy' },
  R: { hex: '#C56883', name: 'Dusty rose',     group: 'burgundy' },
  P: { hex: '#EBC2CC', name: 'Pink quartz',    group: 'burgundy' },
  // Gold ramp (42°)
  U: { hex: '#2D1F0A', name: 'Bog umber',      group: 'gold' },
  Z: { hex: '#5E4116', name: 'Bronze',         group: 'gold' },
  N: { hex: '#9C7026', name: 'Antique brass',  group: 'gold' },
  G: { hex: '#D2A53F', name: 'Old gold',       group: 'gold' },
  L: { hex: '#F0D27D', name: 'Brass leaf',     group: 'gold' },
  C: { hex: '#FCEFC8', name: 'Parchment cream',group: 'gold' },
  // Teal ramp (178°)
  A: { hex: '#062123', name: 'Abyssal',        group: 'teal' },
  D: { hex: '#154548', name: 'Deep teal',      group: 'teal' },
  T: { hex: '#2A8482', name: 'Main teal',      group: 'teal' },
  I: { hex: '#5BB8AF', name: 'Bright teal',    group: 'teal' },
  M: { hex: '#A4DED4', name: 'Mist teal',      group: 'teal' },
  // Forest ramp (140°)
  J: { hex: '#142A19', name: 'Mossbed',        group: 'forest' },
  F: { hex: '#27502E', name: 'Pine',           group: 'forest' },
  E: { hex: '#487E40', name: 'Forest',         group: 'forest' },
  H: { hex: '#82B450', name: 'Spring meadow',  group: 'forest' },
  Q: { hex: '#C9DC97', name: 'Hayfield',       group: 'forest' },
  // Neutrals
  X: { hex: '#161313', name: 'Coalblack',      group: 'neutral' },
  B: { hex: '#3E3833', name: 'Basalt',         group: 'neutral' },
  S: { hex: '#7A7064', name: 'Stone grey',     group: 'neutral' },
  V: { hex: '#BFB2A0', name: 'Aged paper',     group: 'neutral' },
  // Magic violet (270°)
  '*': { hex: '#3A1559', name: 'Royal arcane', group: 'magic' },
  '%': { hex: '#9D6FE0', name: 'Mage glow',    group: 'magic' },
  // Cool counterweight (215°)
  '<': { hex: '#0E2E54', name: 'Cobalt deep',  group: 'cool' },
  '>': { hex: '#377AB8', name: 'Cobalt',       group: 'cool' },
  // Alarms / specular
  '!': { hex: '#E63946', name: 'Alarm scarlet',group: 'alarm' },
  '#': { hex: '#FFFFFF', name: 'Specular wht', group: 'alarm' },
  // Transparent
  '.': null,
  ' ': null,
};

window.PALETTE_GROUPS = {
  burgundy: { label: 'Burgundy ramp · anchor 350°',         keys: ['K','Y','O','W','R','P'] },
  gold:     { label: 'Gold ramp · analogous 42°',           keys: ['U','Z','N','G','L','C'] },
  teal:     { label: "Teal ramp · split-comp 178°",         keys: ['A','D','T','I','M']     },
  forest:   { label: 'Forest ramp · split-comp 140°',       keys: ['J','F','E','H','Q']     },
  neutral:  { label: 'Warm-biased neutrals',                keys: ['X','B','S','V']         },
  magic:    { label: 'Tetradic violet · 270° (≤5%)',        keys: ['*','%']                 },
  cool:     { label: 'Cool counterweight · 215°',           keys: ['<','>']                 },
  alarm:    { label: 'Alarm + specular',                    keys: ['!','#']                 },
};
