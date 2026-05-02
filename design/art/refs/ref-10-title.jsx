// REF-10 — Title screen / wordmark, 640×360.
// "PLEDGE & CROWN" in Oxblood + Bog umber outline + Old gold crown accents.
// Tagline "AN RPG WHERE THE COMPILER IS A BOSS" in Parchment cream below.
// Bottom third: hero shot — player + Ferris in front of Borrow Bridge at sunset,
// Borrow Checker silhouetted at far end. Sky gradient Oxblood → Wineflesh →
// Cobalt deep.

function REF10({ scale = 1 }) {
  const ref = React.useRef(null);
  const W = 640, H = 360;

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

    // ── Sky gradient: Oxblood (top) → Wineflesh → Crypt → Cobalt deep (bottom)
    const skyH = 230;
    for (let y = 0; y < skyH; y++) {
      const t = y / skyH;
      let key;
      if (t < 0.18) key = 'O';
      else if (t < 0.34) key = 'W';
      else if (t < 0.5) key = 'R';
      else if (t < 0.68) key = 'Y';
      else if (t < 0.85) key = 'K';
      else key = '<';
      px(0, y, W, 1, key);
    }
    // Soft horizon glow band
    for (let y = 196; y < 214; y++) {
      const t = (y - 196) / 18;
      px(0, y, W, 1, t < 0.5 ? 'W' : 'Y');
    }
    // Sun setting at horizon center-right
    const sunX = 410, sunY = 198;
    px(sunX-12, sunY-2, 24, 1, 'L');
    px(sunX-14, sunY-1, 28, 4, 'G');
    px(sunX-12, sunY+3, 24, 4, 'N');
    px(sunX-8, sunY+7, 16, 2, 'Z');

    // Distant stars (specular only top-left)
    [[40,18],[88,30],[140,12],[180,22],[220,14],[60,40]].forEach(([x,y]) => px(x, y, 1, 1, '#'));

    // Distant mountains (Cobalt deep silhouettes layered)
    for (let i = 0; i < 6; i++) {
      const baseX = i * 120 - 30;
      const peakH = 28 + (i*7)%18;
      for (let dy = 0; dy < peakH; dy++) {
        const w = (peakH - dy) * 2;
        px(baseX - w/2 + (i*23)%40, 200 - dy, w, 1, 'A');
      }
    }
    // Closer mountain ridge
    for (let i = 0; i < 9; i++) {
      const baseX = i * 80 - 20;
      const peakH = 18 + (i*5)%10;
      for (let dy = 0; dy < peakH; dy++) {
        const w = (peakH - dy) * 2;
        px(baseX - w/2 + (i*17)%30, 215 - dy, w, 1, 'K');
      }
    }

    // ── BORROW BRIDGE — central perspective, going from foreground to horizon
    // Bridge is a long stone span heading away from camera, vanishing at sunset.
    // Stone parapets on both sides converging.
    const bridgeY0 = 232;   // near edge
    const bridgeY1 = 218;   // far edge (where it meets horizon)
    for (let y = bridgeY0; y >= bridgeY1; y--) {
      const t = (bridgeY0 - y) / (bridgeY0 - bridgeY1);
      const halfW = Math.round(220 * (1 - t * 0.78));
      const cx = W / 2;
      // bridge deck
      const deckKey = y % 3 === 0 ? 'V' : (y % 3 === 1 ? 'S' : 'B');
      px(cx - halfW, y, halfW * 2, 1, deckKey);
      // parapet caps
      px(cx - halfW - 2, y, 2, 1, 'X');
      px(cx + halfW, y, 2, 1, 'X');
    }
    // Foreground bridge fill (below near edge — chasm + cobble)
    for (let y = bridgeY0; y < skyH + 30; y++) {
      const halfW = 220 + (y - bridgeY0) * 6;
      const cx = W / 2;
      // dark chasm sides outside bridge
      px(0, y, cx - halfW, 1, 'A');
      px(cx + halfW, y, W - (cx + halfW), 1, 'A');
      // bridge surface (cobble pattern)
      const k = ((y + Math.floor((cx-halfW)/4)) % 3 === 0) ? 'V' : 'S';
      px(cx - halfW, y, halfW * 2, 1, k);
    }
    // Cobble texture spots on near bridge
    for (let i = 0; i < 80; i++) {
      const x = 60 + (i * 7) % 520;
      const y = 234 + (i * 5) % 50;
      if (x < 50 || x > 590) continue;
      px(x, y, 2, 1, i % 2 === 0 ? 'B' : 'V');
    }
    // Bridge stone joints
    for (let y = 240; y < 290; y += 6) {
      const halfW = 220 + (y - bridgeY0) * 6;
      const cx = W / 2;
      px(cx - halfW, y, halfW * 2, 1, 'B');
    }
    // Parapet inner rail (foreground)
    for (let y = bridgeY0; y < 240; y++) {
      const halfW = 220 + (y - bridgeY0) * 6;
      const cx = W / 2;
      px(cx - halfW, y, 4, 1, 'X');
      px(cx + halfW - 4, y, 4, 1, 'X');
    }

    // Distant Borrow Checker silhouette at far end of bridge
    const bcX = W/2 - 4, bcY = 196;
    px(bcX, bcY, 8, 18, 'A');     // robe
    px(bcX+1, bcY-3, 6, 4, 'A');  // hood
    px(bcX+3, bcY+1, 1, 1, 'I');  // teal eye
    px(bcX+9, bcY-2, 1, 22, 'X'); // staff
    px(bcX+8, bcY-4, 3, 2, 'G');  // scale top

    // Two tiny torch braziers either side of bridge, mid distance
    [[200, 220], [432, 220]].forEach(([tx, ty]) => {
      px(tx, ty, 2, 4, 'Z');
      px(tx-1, ty-2, 4, 2, '!');
      px(tx, ty-3, 2, 1, 'L');
    });

    // ── HERO CHARACTERS — player + Ferris on near end of bridge
    // Player: simplified 28-tall sprite, centered slightly left
    const pX = 280, pY = 254;
    // Hair
    px(pX+2, pY, 6, 1, 'X');
    px(pX+1, pY+1, 8, 2, 'B');
    px(pX+2, pY+1, 1, 1, 'S');
    // Head
    px(pX+2, pY+3, 6, 1, 'X');
    px(pX+2, pY+4, 6, 4, 'P');
    px(pX+3, pY+4, 1, 1, 'R');
    px(pX+3, pY+6, 1, 1, 'X');
    px(pX+6, pY+6, 1, 1, 'X');
    // Tunic (cobalt deep)
    px(pX+1, pY+8, 8, 1, 'X');
    px(pX+1, pY+9, 8, 6, '<');
    px(pX+2, pY+10, 1, 4, '>');
    px(pX+5, pY+11, 1, 2, 'I');
    // Belt
    px(pX+1, pY+15, 8, 1, 'N');
    px(pX+4, pY+15, 2, 1, 'G');
    // Legs
    px(pX+2, pY+16, 2, 6, '<');
    px(pX+6, pY+16, 2, 6, '<');
    px(pX+2, pY+22, 2, 2, 'Y');
    px(pX+6, pY+22, 2, 2, 'Y');
    // Staff
    px(pX-2, pY+1, 1, 22, 'Z');
    px(pX-2, pY-1, 2, 2, 'L');
    px(pX-3, pY-2, 4, 2, 'G');

    // Ferris next to player (right side)
    const fX = 296, fY = 270;
    px(fX, fY, 14, 1, 'X');
    px(fX-1, fY+1, 16, 4, 'W');
    px(fX+2, fY+2, 2, 1, 'G');
    px(fX+9, fY+2, 2, 1, 'G');
    px(fX-1, fY+5, 16, 1, 'X');
    px(fX, fY+5, 14, 2, 'R');
    // claws
    px(fX-3, fY+1, 2, 3, 'N');
    px(fX-3, fY+1, 2, 1, 'X');
    px(fX+15, fY+1, 2, 3, 'N');
    px(fX+15, fY+1, 2, 1, 'X');
    // eye stalks
    px(fX+3, fY-2, 1, 2, 'X');
    px(fX+10, fY-2, 1, 2, 'X');
    px(fX+3, fY-3, 1, 1, 'C');
    px(fX+10, fY-3, 1, 1, 'C');

    // ── WORDMARK BANNER (top half)
    // Heraldic burgundy banner with gold trim
    const bnY = 28, bnH = 96;
    // Banner body
    px(60, bnY, W-120, bnH, 'O');
    // Banner gold borders
    px(60, bnY, W-120, 3, 'G');
    px(60, bnY+bnH-3, W-120, 3, 'G');
    px(60, bnY, 3, bnH, 'G');
    px(W-63, bnY, 3, bnH, 'G');
    // Gold inner pinstripe
    px(64, bnY+4, W-128, 1, 'L');
    px(64, bnY+bnH-5, W-128, 1, 'L');
    // Outer dark heraldic outline
    px(58, bnY-2, W-116, 2, 'K');
    px(58, bnY+bnH, W-116, 2, 'K');
    px(58, bnY-2, 2, bnH+4, 'K');
    px(W-60, bnY-2, 2, bnH+4, 'K');
    // Banner side flags / streamers (bottom V cuts at sides)
    for (let i = 0; i < 18; i++) {
      px(56, bnY+bnH+i, 2 + i, 1, 'O');
      px(60+i, bnY+bnH, 2, i, 'O');
      px(W-58, bnY+bnH+i, -(2 + i), 1, 'O');
      px(W-62-i, bnY+bnH, 2, i, 'O');
    }
    // Crown ornament centered above banner
    const crX = W/2;
    // crown band
    px(crX-22, bnY-12, 44, 6, 'G');
    px(crX-22, bnY-12, 44, 1, 'L');
    px(crX-22, bnY-7, 44, 1, 'N');
    px(crX-24, bnY-13, 48, 1, 'X');
    px(crX-24, bnY-6, 48, 1, 'X');
    // crown points
    [-18, -9, 0, 9, 18].forEach((dx, i) => {
      const h = i === 2 ? 12 : (i === 0 || i === 4 ? 7 : 9);
      px(crX+dx-2, bnY-12-h, 4, h, 'G');
      px(crX+dx-2, bnY-12-h, 4, 1, 'X');
      px(crX+dx-2, bnY-12-h, 1, h, 'L');
      // jewel
      const jewel = i === 2 ? 'O' : (i % 2 === 0 ? 'T' : 'O');
      px(crX+dx-1, bnY-12-h+2, 2, 2, jewel);
      px(crX+dx, bnY-12-h+2, 1, 1, '#');
    });

    // ── TITLE LETTERS — pixel-font "PLEDGE & CROWN"
    // Custom 5×7 pixel letters. We'll write a tiny font.
    const FONT = {
      P: ['11110','10001','10001','11110','10000','10000','10000'],
      L: ['10000','10000','10000','10000','10000','10000','11111'],
      E: ['11111','10000','10000','11110','10000','10000','11111'],
      D: ['11110','10001','10001','10001','10001','10001','11110'],
      G: ['01111','10000','10000','10011','10001','10001','01111'],
      '&':['01100','10010','10010','01100','10101','10010','01101'],
      C: ['01111','10000','10000','10000','10000','10000','01111'],
      R: ['11110','10001','10001','11110','10100','10010','10001'],
      O: ['01110','10001','10001','10001','10001','10001','01110'],
      W: ['10001','10001','10001','10001','10101','11011','10001'],
      N: ['10001','11001','10101','10101','10101','10011','10001'],
      ' ': ['00000','00000','00000','00000','00000','00000','00000'],
    };
    // letter scale
    const drawLetter = (ch, ox, oy, sc, fillKey, shadowKey, hiKey) => {
      const g = FONT[ch];
      if (!g) return;
      // shadow
      for (let y = 0; y < 7; y++) for (let x = 0; x < 5; x++) {
        if (g[y][x] === '1') px(ox + x*sc + 2, oy + y*sc + 2, sc, sc, shadowKey);
      }
      // fill
      for (let y = 0; y < 7; y++) for (let x = 0; x < 5; x++) {
        if (g[y][x] === '1') px(ox + x*sc, oy + y*sc, sc, sc, fillKey);
      }
      // top-left highlight (1px corner)
      if (hiKey) {
        for (let y = 0; y < 7; y++) for (let x = 0; x < 5; x++) {
          if (g[y][x] === '1' && (y === 0 || g[y-1][x] === '0') && (x === 0 || g[y][x-1] === '0')) {
            px(ox + x*sc, oy + y*sc, 1, 1, hiKey);
          }
        }
      }
    };

    // PLEDGE & CROWN — single line. 12 chars + gap. scale=4 → letter is 20w × 28h.
    const titleSc = 4;
    const lw = 5*titleSc, lh = 7*titleSc, lg = titleSc;
    const word1 = 'PLEDGE';
    const word2 = '&';
    const word3 = 'CROWN';
    const totalLetters = word1.length + 1 + word2.length + 1 + word3.length;
    const totalW = totalLetters * (lw + lg) - lg;
    let cursor = (W - totalW) / 2;
    const titleY = bnY + (bnH - lh) / 2 + 2;
    const drawWord = (word) => {
      for (const ch of word) {
        drawLetter(ch, cursor, titleY, titleSc, 'C', 'K', 'L');
        cursor += lw + lg;
      }
    };
    drawWord(word1); cursor += lw + lg;
    drawLetter('&', cursor, titleY, titleSc, 'G', 'K', 'L'); cursor += 2*(lw + lg);
    drawWord(word3);

    // Tagline below banner
    const TAG_FONT = {
      A:['010','101','111','101','101'],
      N:['101','111','111','111','101'],
      R:['110','101','110','110','101'],
      P:['110','101','110','100','100'],
      G:['011','100','101','101','011'],
      W:['101','101','101','111','010'],
      H:['101','101','111','101','101'],
      E:['111','100','110','100','111'],
      ' ':['000','000','000','000','000'],
      T:['111','010','010','010','010'],
      C:['011','100','100','100','011'],
      O:['010','101','101','101','010'],
      M:['101','111','111','101','101'],
      I:['111','010','010','010','111'],
      L:['100','100','100','100','111'],
      S:['011','100','010','001','110'],
      B:['110','101','110','101','110'],
    };
    const tag = 'AN RPG WHERE THE COMPILER IS A BOSS';
    const tagSc = 2;
    const tlw = 3*tagSc, tlh = 5*tagSc, tlg = 1*tagSc;
    const tagY = bnY + bnH + 24;
    const tagW = tag.length * (tlw + tlg) - tlg;
    let tcur = (W - tagW) / 2;
    for (const ch of tag) {
      const g = TAG_FONT[ch];
      if (g) {
        // small shadow
        for (let y = 0; y < 5; y++) for (let x = 0; x < 3; x++) {
          if (g[y][x] === '1') px(tcur + x*tagSc + 1, tagY + y*tagSc + 1, tagSc, tagSc, 'K');
        }
        for (let y = 0; y < 5; y++) for (let x = 0; x < 3; x++) {
          if (g[y][x] === '1') px(tcur + x*tagSc, tagY + y*tagSc, tagSc, tagSc, 'C');
        }
      }
      tcur += tlw + tlg;
    }

    // Press start prompt at very bottom
    const PROMPT_Y = H - 18;
    const prompt = 'PRESS START';
    let pcur = W/2 - (prompt.length * 4 * tagSc) / 2;
    for (const ch of prompt) {
      const g = TAG_FONT[ch];
      if (g) {
        for (let y = 0; y < 5; y++) for (let x = 0; x < 3; x++) {
          if (g[y][x] === '1') px(pcur + x*tagSc, PROMPT_Y + y*tagSc, tagSc, tagSc, 'L');
        }
      }
      pcur += tlw + tlg;
    }

  }, []);

  return (
    <canvas ref={ref}
      style={{ width: W*scale, height: H*scale, imageRendering: 'pixelated', display: 'block' }} />
  );
}
window.REF10 = REF10;
window.REF10_ROLES = ['O','W','R','Y','K','<','A','S','V','B','X','I','G','L','N','Z','C','#','T','!','>','P'];
