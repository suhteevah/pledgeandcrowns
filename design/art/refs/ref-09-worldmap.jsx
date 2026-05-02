// REF-09 — World map cinematic background, 320×180. All 11 zones, painted
// pixel-art style with pin markers (small flame icons) per zone. No text.

function REF09({ scale = 2 }) {
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

    // Sky gradient — Cobalt deep top → Cobalt → mist teal at horizon
    for (let y = 0; y < 60; y++) {
      let key = '<';
      if (y > 30) key = '>';
      if (y > 50) key = 'M';
      px(0, y, W, 1, key);
    }
    // a few stars (specular white)
    [[14,8],[28,16],[55,5],[88,12],[132,7],[176,14],[212,9],[244,17],[278,6],[300,12],[40,22],[105,20]].forEach(([x,y]) => px(x,y,1,1,'#'));
    // distant moon
    px(258, 14, 8, 8, 'C'); px(258, 14, 1, 1, 'V'); px(265, 21, 1, 1, 'V');
    px(262, 16, 2, 2, 'V');

    // Sea (Briney Cove + map ocean) — left edge
    for (let y = 60; y < H; y++) {
      const t = (y-60) / (H-60);
      const k = t < 0.4 ? '>' : (t < 0.75 ? '<' : 'A');
      px(0, y, 60, 1, k);
    }
    // wave glints on sea
    for (let i = 0; i < 30; i++) {
      const x = 4 + (i * 7) % 52;
      const y = 70 + (i * 13) % 100;
      px(x, y, 2, 1, 'M');
    }

    // Coast: Briney Cove sand
    px(58, 60, 18, H-60, 'Q');
    px(60, 62, 14, H-62, 'L');

    // Continent base: forest greens (Hearthstone south + map body)
    for (let y = 60; y < H; y++) {
      px(76, y, W-76, 1, 'F');
    }
    // texture: scattered Forest mid + Pine shadow
    for (let i = 0; i < 220; i++) {
      const x = 80 + (i*47) % (W-90);
      const y = 64 + (i*29) % (H-70);
      px(x, y, 1, 1, 'E');
    }
    for (let i = 0; i < 140; i++) {
      const x = 80 + (i*53) % (W-90);
      const y = 70 + (i*37) % (H-80);
      px(x, y, 1, 1, 'J');
    }

    // Big gorge for Borrow Bridge (vertical-ish slash through the middle)
    for (let y = 80; y < 150; y++) {
      const cx = 152 + Math.round(Math.sin((y-80)*0.18) * 4);
      px(cx-3, y, 6, 1, 'A');
      px(cx-2, y, 4, 1, 'B');
      px(cx-1, y, 2, 1, 'X');
    }
    // Bridge spans gorge (stone)
    px(146, 110, 16, 4, 'V');
    px(146, 110, 16, 1, 'X');
    px(146, 113, 16, 1, 'X');
    px(148, 111, 12, 2, 'S');

    // Hearthstone Village (south) — Spring meadow patch + cottages
    for (let i = 0; i < 50; i++) {
      const x = 100 + (i*11) % 30;
      const y = 150 + (i*7) % 22;
      px(x, y, 2, 1, 'H');
    }
    // 3 cottages
    [[110, 145], [122, 152], [98, 154]].forEach(([cx,cy])=>{
      px(cx, cy, 6, 4, 'V');     // wall
      px(cx-1, cy-2, 8, 3, 'O'); // roof
      px(cx-1, cy-2, 1, 1, 'X'); px(cx+6, cy-2, 1, 1, 'X');
      px(cx+2, cy+1, 2, 3, 'Y'); // door
      px(cx, cy, 6, 1, 'X'); px(cx, cy+3, 6, 1, 'X');
    });

    // Guildhall City (mid-east) — oxblood brick rooftops cluster
    for (let i = 0; i < 14; i++) {
      const cx = 200 + (i%5)*9;
      const cy = 92 + Math.floor(i/5)*10;
      px(cx, cy, 7, 6, 'V');
      px(cx-1, cy-2, 9, 3, 'O');
      px(cx-1, cy-2, 1, 1, 'X'); px(cx+7, cy-2, 1, 1, 'X');
      px(cx+2, cy+2, 2, 4, 'Y');
    }

    // Trait Mage's Tower — tall Royal arcane spire (north-east)
    px(244, 60, 10, 50, '*');
    px(245, 58, 8, 4, '*');
    px(244, 60, 1, 50, 'X'); px(253, 60, 1, 50, 'X');
    px(244, 110, 10, 1, 'X');
    // Magic glow halo at top
    px(243, 56, 12, 1, '%');
    px(245, 54, 8, 2, '%');
    px(247, 52, 4, 2, 'L');
    // gold cap
    px(247, 56, 4, 2, 'G');

    // Tavern (small lit building near roads)
    px(166, 138, 10, 8, 'V');
    px(165, 136, 12, 3, 'Z');
    px(165, 136, 1, 1, 'X'); px(176, 136, 1, 1, 'X');
    px(170, 142, 2, 4, 'Y');
    px(168, 141, 1, 1, '!'); px(173, 141, 1, 1, '!'); // window lights

    // Iron Vale Forge — smoke (mid-west, north of village)
    px(95, 110, 12, 14, 'B');
    px(94, 108, 14, 3, 'X');
    px(98, 124, 6, 4, 'O'); // forge fire glow inside
    px(99, 125, 4, 2, '!');
    // smoke plume
    for (let i = 0; i < 12; i++) {
      const sy = 100 - i*2;
      const sx = 96 + (i%3) - 1;
      if (sy < 64) break;
      px(sx, sy, 4, 2, 'S');
      px(sx+1, sy, 3, 1, 'V');
    }

    // The Vault — mountain entrance, gold gleam
    px(280, 120, 30, 40, 'S');
    px(280, 120, 30, 1, 'X');
    // mountain triangle
    for (let i = 0; i < 30; i++) {
      const w = i*1;
      const cx = 295 - Math.floor(w/2);
      px(cx, 120-i, w, 1, i % 3 === 0 ? 'V' : 'S');
    }
    // gold gleam at entrance
    px(290, 138, 8, 6, 'X');
    px(291, 139, 6, 4, 'G');
    px(293, 140, 2, 2, 'L');

    // Forbidden Library — dark cathedral silhouette (far east)
    px(296, 95, 18, 25, 'A');
    px(296, 95, 18, 1, 'X');
    px(298, 90, 4, 6, 'A'); // spire 1
    px(308, 88, 4, 8, 'A'); // spire 2
    px(303, 85, 4, 11, 'A'); // tall middle spire
    // small alarm glints (corruption)
    px(304, 100, 1, 1, '!'); px(310, 105, 1, 1, '!');

    // Throne of the Compiler — silver citadel atop a peak (north center)
    // Peak
    for (let i = 0; i < 22; i++) {
      const w = i+2;
      const cx = 175 - Math.floor(w/2);
      px(cx, 90-i, w, 1, i % 4 === 0 ? 'V' : 'S');
    }
    // citadel
    px(170, 70, 10, 10, 'V');
    px(170, 70, 10, 1, 'X'); px(170, 79, 10, 1, 'X');
    px(170, 70, 1, 10, 'X'); px(179, 70, 1, 10, 'X');
    px(173, 65, 4, 6, 'V');
    px(174, 62, 2, 4, 'G'); // gold spire
    px(174, 60, 2, 2, '#'); // specular tip

    // Temple — stained glass cathedral floating in clouds (top, off-map)
    px(110, 30, 24, 20, 'O');
    px(110, 30, 24, 1, 'X'); px(110, 49, 24, 1, 'X');
    px(110, 30, 1, 20, 'X'); px(133, 30, 1, 20, 'X');
    // stained glass window panes (split-comp triad)
    px(114, 34, 4, 5, 'O');
    px(120, 34, 4, 5, 'T');
    px(126, 34, 4, 5, 'E');
    px(114, 41, 4, 5, 'G');
    px(120, 41, 4, 5, '%');
    px(126, 41, 4, 5, 'G');
    // cloud platform
    for (let i = 0; i < 30; i++) {
      const cx = 100 + (i*3) % 40;
      const cy = 50 + (i*5) % 4;
      px(cx, cy, 4, 2, 'M');
      px(cx, cy, 1, 1, 'V');
    }

    // Pin markers — small flame icons in zone-themed colors at zone centers
    const pin = (cx, cy, color) => {
      // flame
      px(cx, cy-2, 1, 1, color);
      px(cx-1, cy-1, 3, 1, color);
      px(cx-1, cy, 3, 1, color);
      px(cx, cy+1, 1, 1, 'X');
    };
    pin(112, 142, 'H');   // Hearthstone
    pin(154, 105, 'T');   // Borrow Bridge
    pin(212, 95, 'O');    // Guildhall
    pin(248, 75, '%');    // Mage Tower
    pin(170, 138, 'L');   // Tavern
    pin(101, 116, '!');   // Forge
    pin(67, 130, 'I');    // Briney Cove
    pin(295, 132, 'G');   // Vault
    pin(305, 100, '*');   // Library
    pin(174, 75, '#');    // Throne
    pin(122, 40, 'T');    // Temple

  }, []);

  return (
    <canvas ref={ref}
      style={{ width: W*scale, height: H*scale, imageRendering: 'pixelated', display: 'block' }} />
  );
}
window.REF09 = REF09;
window.REF09_ROLES = ['<','>','M','A','#','C','V','Q','L','F','E','J','S','B','X','H','O','Y','*','%','G','T','I','Z','!'];
