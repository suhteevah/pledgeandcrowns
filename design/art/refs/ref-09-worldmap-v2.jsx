// REF-09 v2 — World map cinematic, 320×180, redone as a single cohesive
// painted scene with proper depth bands, atmospheric perspective, connecting
// roads, and a real coastline curve. Pin markers stay; zones flow as one map.

function REF09v2({ scale = 2 }) {
  const ref = React.useRef(null);
  const W = 320, H = 180;

  React.useEffect(() => {
    const cv = ref.current;
    cv.width = W; cv.height = H;
    const ctx = cv.getContext('2d');
    ctx.imageSmoothingEnabled = false;

    const C = window.PALETTE;
    const px = (x, y, w, h, key) => {
      const c = C[key]; if (!c) return;
      ctx.fillStyle = c.hex; ctx.fillRect(x|0, y|0, w|0, h|0);
    };

    // ── Sky band (top): Cobalt deep → Cobalt → Mist teal at horizon
    for (let y = 0; y < 56; y++) {
      const t = y / 56;
      const k = t < 0.35 ? '<' : (t < 0.7 ? '>' : 'M');
      px(0, y, W, 1, k);
    }
    // Stars + moon
    [[18,8],[34,15],[60,6],[92,12],[140,7],[182,14],[218,9],[252,17],[286,5],[300,11]].forEach(([x,y]) => px(x,y,1,1,'#'));
    px(264, 14, 7, 7, 'C'); px(264, 14, 1, 1, 'V'); px(270, 20, 1, 1, 'V');

    // ── Cloud strata (mid-sky, behind floating Temple)
    for (let i = 0; i < 40; i++) {
      const cx = (i * 17) % W;
      const cy = 30 + (i * 7) % 22;
      px(cx, cy, 8, 2, 'M');
      px(cx+1, cy-1, 6, 1, 'C');
    }
    // Lower cloud band (the floating Temple platform)
    for (let i = 0; i < 35; i++) {
      const cx = (i * 11) % W;
      const cy = 44 + (i * 3) % 6;
      px(cx, cy, 10, 3, 'V');
      px(cx, cy, 10, 1, 'C');
    }

    // ── TEMPLE — floating cathedral (top center), bigger and integrated
    const tcX = 132, tcY = 22;
    // base platform / cloud halo
    for (let i = 0; i < 60; i++) {
      const cx = tcX - 30 + i;
      const cy = 50 + Math.round(Math.sin(i * 0.4) * 1);
      px(cx, cy, 1, 4, 'C');
    }
    // cathedral body
    px(tcX-2, tcY+8, 28, 24, 'O');
    px(tcX-2, tcY+8, 28, 1, 'X');
    px(tcX-2, tcY+31, 28, 1, 'X');
    px(tcX-2, tcY+8, 1, 24, 'X');
    px(tcX+25, tcY+8, 1, 24, 'X');
    // central spire
    px(tcX+10, tcY-6, 4, 14, 'O');
    px(tcX+10, tcY-6, 4, 1, 'X');
    px(tcX+11, tcY-9, 2, 4, 'G');
    px(tcX+11, tcY-11, 2, 2, '#');
    // side spires
    px(tcX+1, tcY+2, 3, 8, 'O');
    px(tcX+20, tcY+2, 3, 8, 'O');
    px(tcX+1, tcY+2, 3, 1, 'X');
    px(tcX+20, tcY+2, 3, 1, 'X');
    // stained-glass rose window — split-comp triad
    const rwX = tcX+8, rwY = tcY+13;
    px(rwX+2, rwY, 4, 1, 'X');
    px(rwX, rwY+2, 1, 4, 'X');
    px(rwX+8, rwY+2, 1, 4, 'X');
    px(rwX+2, rwY+8, 4, 1, 'X');
    px(rwX+1, rwY+1, 1, 1, 'X'); px(rwX+7, rwY+1, 1, 1, 'X');
    px(rwX+1, rwY+7, 1, 1, 'X'); px(rwX+7, rwY+7, 1, 1, 'X');
    // pane fills
    px(rwX+3, rwY+1, 3, 1, 'O');
    px(rwX+1, rwY+3, 3, 3, 'T');
    px(rwX+5, rwY+3, 3, 3, 'E');
    px(rwX+3, rwY+6, 3, 1, 'G');
    px(rwX+4, rwY+4, 1, 1, '%');
    // lower windows along nave
    [4, 10, 16].forEach(dx => {
      px(tcX+dx, tcY+18, 2, 6, 'T');
      px(tcX+dx, tcY+18, 2, 1, 'X');
    });
    px(tcX+22, tcY+18, 2, 6, 'O');

    // ── HORIZON LINE — faint mist
    for (let y = 56; y < 64; y++) {
      px(0, y, W, 1, y < 60 ? 'M' : 'V');
    }

    // ── DISTANT MOUNTAINS (background depth band) — Throne in the middle
    for (let i = 0; i < 12; i++) {
      const baseX = i * 28 - 10;
      const peakH = 12 + (i*5)%14;
      for (let dy = 0; dy < peakH; dy++) {
        const w = (peakH - dy) * 2;
        px(baseX - w/2 + (i*11)%18, 64 - dy, w, 1, dy < peakH * 0.7 ? 'V' : '#');
      }
    }
    // Throne of the Compiler — silver citadel atop tallest peak (center)
    const throneX = 168;
    // big peak
    for (let dy = 0; dy < 28; dy++) {
      const w = (28 - dy) * 2 + 4;
      px(throneX - w/2, 70 - dy, w, 1, dy < 5 ? '#' : (dy < 12 ? 'V' : 'S'));
    }
    // citadel
    px(throneX-7, 56, 14, 12, 'V');
    px(throneX-7, 56, 14, 1, 'X');
    px(throneX-7, 67, 14, 1, 'X');
    px(throneX-7, 56, 1, 12, 'X');
    px(throneX+6, 56, 1, 12, 'X');
    px(throneX-3, 60, 6, 4, '<');
    px(throneX-4, 50, 8, 6, 'V');
    px(throneX-4, 50, 8, 1, 'X');
    px(throneX-1, 46, 2, 5, 'G');
    px(throneX-1, 44, 2, 2, '#');

    // Forbidden Library — far east, dark cathedral on the ridge
    const libX = 280;
    px(libX, 58, 22, 14, 'A');
    px(libX, 58, 22, 1, 'X');
    px(libX+2, 52, 4, 7, 'A'); px(libX+2, 52, 4, 1, 'X');
    px(libX+10, 48, 5, 11, 'A'); px(libX+10, 48, 5, 1, 'X');
    px(libX+18, 52, 4, 7, 'A'); px(libX+18, 52, 4, 1, 'X');
    px(libX+12, 54, 1, 1, '!');
    px(libX+5, 60, 1, 1, '!');

    // ── MIDGROUND LAND BAND
    // Continent base — forest greens, with a gradient that brings warmth at the south
    for (let y = 64; y < H; y++) {
      const t = (y - 64) / (H - 64);
      let key = 'F';
      if (t > 0.4) key = 'E';
      if (t > 0.75) key = 'H';
      px(0, y, W, 1, key);
    }
    // Texture: pine shadow + spring meadow specks
    for (let i = 0; i < 320; i++) {
      const x = (i * 47) % W;
      const y = 66 + (i * 29) % (H - 66);
      px(x, y, 1, 1, i % 3 === 0 ? 'J' : (i % 3 === 1 ? 'F' : 'E'));
    }
    // Subtle scattered trees (pine triangles)
    for (let i = 0; i < 35; i++) {
      const x = 70 + (i * 13) % 210;
      const y = 70 + (i * 7) % 60;
      // skip if on coast or gorge
      if (x < 78 || (x > 130 && x < 168 && y > 86 && y < 130)) continue;
      px(x, y, 1, 3, 'F');
      px(x-1, y+1, 3, 1, 'F');
      px(x-1, y+2, 3, 1, 'J');
    }

    // ── COASTLINE (west) — proper curve, sea + sand
    // Sea
    for (let y = 64; y < H; y++) {
      const t = (y - 64) / (H - 64);
      // coast curves: x_coast = function of y
      const xc = 38 + Math.round(Math.sin((y-64) * 0.08) * 8) + Math.round((y-64) * 0.05);
      // water gradient (deep cobalt deep → cobalt → mist near shore)
      for (let x = 0; x < xc; x++) {
        const wt = x / xc;
        const k = wt < 0.3 ? 'A' : (wt < 0.7 ? '<' : '>');
        px(x, y, 1, 1, k);
      }
      // sand
      px(xc, y, 6, 1, 'Q');
      px(xc, y, 3, 1, 'L');
    }
    // wave glints
    for (let i = 0; i < 40; i++) {
      const x = 4 + (i * 7) % 40;
      const y = 72 + (i * 11) % 100;
      px(x, y, 2, 1, 'M');
    }
    // Briney Cove ships — small distant sails
    [[14, 110], [22, 130]].forEach(([sx, sy]) => {
      px(sx, sy, 6, 2, 'V');
      px(sx, sy, 6, 1, 'X');
      px(sx+2, sy-6, 1, 6, 'B');
      px(sx, sy-5, 5, 4, 'C');
      px(sx, sy-5, 5, 1, 'X');
    });

    // ── BORROW BRIDGE GORGE — cuts across midway, cleanly
    const gorgeY0 = 86, gorgeY1 = 132;
    for (let y = gorgeY0; y < gorgeY1; y++) {
      const cx = 148 + Math.round(Math.sin((y-gorgeY0) * 0.18) * 3);
      const w = 8 - Math.abs(y - (gorgeY0+gorgeY1)/2) * 0.05;
      px(cx - w/2, y, w, 1, 'A');
      px(cx - w/2, y, 1, 1, 'X');
      px(cx + w/2 - 1, y, 1, 1, 'X');
    }
    // Bridge — clear stone span across gorge
    const brY = 108;
    px(140, brY, 22, 5, 'V');
    px(140, brY, 22, 1, 'X');
    px(140, brY+4, 22, 1, 'X');
    px(143, brY+1, 16, 2, 'S');
    // arches under bridge
    [144, 150, 156].forEach(ax => {
      px(ax, brY+5, 4, 3, 'A');
      px(ax, brY+5, 4, 1, 'X');
    });

    // ── ROADS — connecting major zones, 1px tan paths
    const drawRoad = (pts) => {
      for (let i = 1; i < pts.length; i++) {
        const [x0,y0] = pts[i-1], [x1,y1] = pts[i];
        const dx = x1-x0, dy = y1-y0;
        const steps = Math.max(Math.abs(dx), Math.abs(dy));
        for (let s = 0; s <= steps; s++) {
          const x = x0 + Math.round(dx * s/steps);
          const y = y0 + Math.round(dy * s/steps);
          px(x, y, 1, 1, 'N');
          px(x, y+1, 1, 1, 'Z');
        }
      }
    };
    // Hearthstone → Bridge → Guildhall → Throne
    drawRoad([[100, 158], [120, 140], [140, 120], [148, 110], [160, 102], [180, 94], [186, 80], [180, 70]]);
    // Hearthstone → Tavern → Forge
    drawRoad([[100, 158], [82, 148], [76, 134], [82, 120]]);
    // Guildhall → Library
    drawRoad([[200, 96], [240, 88], [275, 78]]);
    // Mage Tower offshoot
    drawRoad([[200, 96], [232, 84], [248, 76]]);

    // ── HEARTHSTONE VILLAGE (south) — cluster of cottages on meadow
    const hsX = 96, hsY = 156;
    // meadow patch
    for (let i = 0; i < 30; i++) {
      const x = hsX - 14 + (i * 5) % 28;
      const y = hsY - 6 + (i * 3) % 14;
      px(x, y, 1, 1, 'H');
    }
    // 4 cottages
    [[-10, 0], [4, 4], [-4, -8], [10, -2]].forEach(([dx, dy]) => {
      const cx = hsX + dx, cy = hsY + dy;
      px(cx, cy, 6, 4, 'V');
      px(cx, cy+3, 6, 1, 'X');
      px(cx-1, cy-2, 8, 3, 'O');
      px(cx-1, cy-2, 1, 1, 'X');
      px(cx+7, cy-2, 1, 1, 'X');
      px(cx+2, cy+1, 2, 2, 'Y');
      px(cx+5, cy+1, 1, 1, 'L');  // window light
    });

    // ── GUILDHALL CITY (mid-east of bridge)
    const ghX = 192;
    for (let i = 0; i < 9; i++) {
      const cx = ghX + (i % 3) * 7;
      const cy = 88 + Math.floor(i/3) * 8;
      px(cx, cy, 6, 6, 'V');
      px(cx, cy+5, 6, 1, 'X');
      px(cx-1, cy-2, 8, 3, 'O');
      px(cx-1, cy-2, 1, 1, 'X');
      px(cx+7, cy-2, 1, 1, 'X');
      px(cx+2, cy+2, 2, 3, 'Y');
    }
    // big guildhall keep
    px(ghX+12, 86, 12, 14, 'V');
    px(ghX+12, 86, 12, 1, 'X');
    px(ghX+12, 99, 12, 1, 'X');
    px(ghX+11, 82, 14, 5, 'O');
    px(ghX+11, 82, 14, 1, 'X');
    px(ghX+16, 78, 4, 5, 'O');
    px(ghX+17, 76, 2, 3, 'G');

    // ── TRAIT MAGE'S TOWER (NE)
    const mtX = 248;
    px(mtX, 70, 8, 22, '*');
    px(mtX, 70, 1, 22, 'X');
    px(mtX+7, 70, 1, 22, 'X');
    px(mtX, 92, 8, 1, 'X');
    px(mtX-1, 66, 10, 4, '*');
    px(mtX-1, 66, 10, 1, 'X');
    px(mtX+1, 62, 6, 4, 'G');
    px(mtX+1, 62, 6, 1, 'X');
    px(mtX+2, 58, 4, 4, '%');
    px(mtX+3, 56, 2, 2, '#');
    // magical glow
    for (let i = 0; i < 5; i++) px(mtX-2+i*3, 64, 1, 1, '%');

    // ── TAVERN (small lit)
    px(80, 134, 10, 8, 'V');
    px(80, 141, 10, 1, 'X');
    px(79, 132, 12, 3, 'Z');
    px(79, 132, 1, 1, 'X');
    px(90, 132, 1, 1, 'X');
    px(83, 138, 2, 4, 'Y');
    px(81, 137, 1, 1, '!');
    px(87, 137, 1, 1, '!');

    // ── IRON VALE FORGE (W)
    const fgX = 76;
    px(fgX, 116, 12, 12, 'B');
    px(fgX, 116, 12, 1, 'X');
    px(fgX, 127, 12, 1, 'X');
    px(fgX+4, 122, 4, 4, 'O');
    px(fgX+4, 123, 4, 2, '!');
    // smoke plume
    for (let i = 0; i < 14; i++) {
      const sy = 114 - i * 3;
      const sx = fgX+5 + (i%3) - 1;
      if (sy < 60) break;
      px(sx, sy, 5, 2, 'S');
      px(sx+1, sy, 4, 1, 'V');
    }

    // ── THE VAULT (mountain entrance, south of mage tower)
    const vtX = 270;
    for (let dy = 0; dy < 18; dy++) {
      const w = (18 - dy);
      px(vtX - w/2, 110 - dy, w + 8, 1, dy < 4 ? 'V' : 'S');
    }
    px(vtX-1, 116, 8, 4, 'X');
    px(vtX, 117, 6, 3, 'G');
    px(vtX+2, 118, 2, 2, 'L');

    // ── BRINEY COVE (west sand inlet) — emphasize cove curve
    for (let i = 0; i < 18; i++) {
      const x = 30 + i;
      const y = 138 + Math.round(Math.sin(i * 0.4) * 2);
      px(x, y, 1, 4, 'L');
      px(x, y-1, 1, 1, 'Q');
    }

    // ── PIN MARKERS — small flame icons in zone-themed colors
    const pin = (cx, cy, color) => {
      px(cx-1, cy-3, 3, 1, color);
      px(cx-1, cy-2, 3, 2, color);
      px(cx, cy-4, 1, 1, 'L');
      px(cx, cy, 1, 2, 'X');
    };
    pin(96, 152, 'H');    // Hearthstone
    pin(150, 110, 'T');   // Borrow Bridge
    pin(200, 92, 'O');    // Guildhall
    pin(252, 80, '%');    // Mage Tower
    pin(85, 130, 'L');    // Tavern
    pin(82, 118, '!');    // Forge
    pin(38, 138, 'I');    // Briney Cove
    pin(275, 112, 'G');   // Vault
    pin(290, 64, '*');    // Library
    pin(168, 50, '#');    // Throne
    pin(146, 30, 'T');    // Temple

  }, []);

  return (
    <canvas ref={ref}
      style={{ width: W*scale, height: H*scale, imageRendering: 'pixelated', display: 'block' }} />
  );
}
window.REF09 = REF09v2;  // override v1
window.REF09_ROLES = ['<','>','M','A','#','C','V','Q','L','F','E','J','S','B','X','H','O','Y','*','%','G','T','I','Z','N','!'];
