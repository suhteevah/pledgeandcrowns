// SPDX-License-Identifier: MIT
// REF-97 - Forge yard slab, 16x16 tiles, 2-column sheet.
// APPROVED by Matt 2026-11-18 (Forge district style sample). Not wired yet.
//
// Ground for the Iterator Forge district (Act 6). Same slab layout as the
// REF-75 flagstone so the two can meet on a straight seam, but in stone
// grey with basalt joints and soot: a mid-tone, so the Forge cast (basalt
// aprons, coalblack outlines) still reads against it, and clearly darker
// and cooler than the cream plazas of the other districts.
// soot_b carries a scorch mark; use it for about one tile in six.
//
// Legend: S stone grey (slab)  B basalt (joints, soot)
//         V aged paper (lit corner)  U bog umber (scorch speck)

const REF97_COLS = 2;

const REF97_TILES = {
  soot_a: [
    'VSSSSSBVSSSSSSSB',
    'SSSSSSBSSSSSSSSB',
    'SSSSSSBSSSSSSSSB',
    'SSSSSSBSSSSSSSSB',
    'SSSSSSBSSSSSSVSB',
    'SSSSSSBSSSSSSSSB',
    'SSSSSSBSSSSSSSSB',
    'BBBBBBBBBBBBBBBB',
    'SSSBVSSSSSSSBVSS',
    'SSSBSSSSSSSSBSSS',
    'SSSBSSSSSSSSBSSS',
    'SSSBSSSSUSSSBSSS',
    'SSSBSSSSSSSSBSSS',
    'SSSBSSBSSSSSBSSS',
    'SSSBSSSSSSSSBSSS',
    'BBBBBBBBBBBBBBBB',
  ],
  soot_b: [
    'VSSSSSSSSBVSSSSB',
    'SSSSSSSSSBSSSSSB',
    'SSSSSSSSSBSSSSSB',
    'SSSBBSSSSBSSSSSB',
    'SSBBBBSSSBSSSSSB',
    'SSSBBSSSSBSSUSSB',
    'SSSSSSSSSBSSSSSB',
    'BBBBBBBBBBBBBBBB',
    'SSSSSBVSSSSSSSSS',
    'SSSSSBSSSSSSSSSS',
    'SSSSSBSSSSSBSSSS',
    'SSSSSBSSSSSSSSSS',
    'SSSSSBSSSSSSSSSS',
    'SSSSSBSSSSSSSSSS',
    'SSUSSBSSSSSSSSSS',
    'BBBBBBBBBBBBBBBB',
  ],
};

window.REF97_TILES = REF97_TILES;
window.REF97_COLS = REF97_COLS;
