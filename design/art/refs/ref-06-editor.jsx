// REF-06 — Code Editor frame, 640×360. No code yet.
// Top bar w/ CAST/TEST/COMMIT pixel-glyph buttons. Main area split 60/40.
// Left: parchment book with leather binding. Right: GOAL (top, aged paper)
// + COMPILER OUTPUT (bottom, cobalt deep slate). Outer Stone grey double-line
// border with Oxblood corner rivets.

// We'll generate this procedurally — 640×360 is too large to hand-pixel.
function REF06({ scale = 1 }) {
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
    const rect = (x, y, w, h, key) => px(x, y, w, h, key);
    const stroke = (x, y, w, h, key) => {
      px(x, y, w, 1, key); px(x, y+h-1, w, 1, key);
      px(x, y, 1, h, key); px(x+w-1, y, 1, h, key);
    };

    // ── BG: dim parchment underlay (so the chamber doesn't show through cracks)
    rect(0, 0, W, H, 'V');

    // ── Outer frame: Stone grey double line w/ Oxblood corner rivets
    rect(0, 0, W, H, 'S');
    stroke(0, 0, W, H, 'X');                        // black outer edge
    rect(2, 2, W-4, H-4, 'S');                      // grey band
    stroke(4, 4, W-8, H-8, 'X');                    // inner edge
    rect(6, 6, W-12, H-12, 'B');                    // basalt inset
    stroke(7, 7, W-14, H-14, 'X');
    // Oxblood corner rivets (4 corners, 5×5)
    [[6,6],[W-12,6],[6,H-12],[W-12,H-12]].forEach(([rx,ry])=>{
      rect(rx, ry, 6, 6, 'O');
      stroke(rx, ry, 6, 6, 'X');
      rect(rx+2, ry+2, 2, 2, 'L');
    });

    // ── Top bar (24px tall, just inside outer frame)
    const topY = 14, topH = 28;
    rect(14, topY, W-28, topH, 'B');
    stroke(14, topY, W-28, topH, 'X');
    rect(15, topY+1, W-30, 1, 'S');                  // bevel highlight

    // Pixel-glyph buttons: CAST · TEST · COMMIT
    const drawBtn = (bx, by, bw, label, accent) => {
      rect(bx, by, bw, 18, 'V');
      stroke(bx, by, bw, 18, 'X');
      rect(bx+1, by+1, bw-2, 1, 'C');                // top hi
      rect(bx+1, by+16, bw-2, 1, 'N');               // bottom shadow
      // accent square
      rect(bx+3, by+5, 8, 8, accent);
      stroke(bx+3, by+5, 8, 8, 'X');
      // pixel-glyph "letters" — just blocks suggesting text
      let lx = bx + 14;
      for (let i = 0; i < label.length; i++) {
        rect(lx, by+6, 4, 6, 'X');
        rect(lx, by+8, 4, 1, 'V');
        lx += 6;
      }
    };
    drawBtn(20,  topY+5, 70,  'CAST',   'O');         // oxblood
    drawBtn(96,  topY+5, 70,  'TEST',   'T');         // teal
    drawBtn(172, topY+5, 92,  'COMMIT', 'G');         // gold

    // Window-style chrome dot on right
    [W-30, W-44, W-58].forEach((cx, i) => {
      const col = ['O','G','T'][i];
      rect(cx, topY+9, 8, 8, col);
      stroke(cx, topY+9, 8, 8, 'X');
    });

    // ── Main area split 60/40 below top bar
    const mainY = topY + topH + 4;
    const mainH = H - mainY - 14;
    const splitX = 14 + Math.floor((W-28) * 0.6);

    // LEFT panel — parchment book
    const leftX = 14, leftW = splitX - leftX - 2;
    rect(leftX, mainY, leftW, mainH, 'B');
    stroke(leftX, mainY, leftW, mainH, 'X');
    // iron rivets at corners of panel
    [[leftX+2,mainY+2],[leftX+leftW-8,mainY+2],[leftX+2,mainY+mainH-8],[leftX+leftW-8,mainY+mainH-8]].forEach(([rx,ry])=>{
      rect(rx, ry, 6, 6, 'O'); stroke(rx, ry, 6, 6, 'X');
    });
    // leather binding strip on left edge
    rect(leftX+10, mainY+10, 14, mainH-20, 'O');
    stroke(leftX+10, mainY+10, 14, mainH-20, 'X');
    // 3 leather bands across binding
    [0.25, 0.5, 0.75].forEach(t => {
      const by = mainY + 10 + Math.floor((mainH-20)*t);
      rect(leftX+8, by-2, 18, 5, 'Y');
      stroke(leftX+8, by-2, 18, 5, 'X');
      rect(leftX+10, by, 14, 1, 'L');
    });
    // Parchment page
    const pgX = leftX+26, pgY = mainY+10, pgW = leftW-36, pgH = mainH-20;
    rect(pgX, pgY, pgW, pgH, 'C');
    stroke(pgX, pgY, pgW, pgH, 'B');
    // Subtle parchment texture: scattered V dots
    for (let i = 0; i < 80; i++) {
      const dx = pgX + 4 + ((i*37) % (pgW-8));
      const dy = pgY + 4 + ((i*53) % (pgH-8));
      px(dx, dy, 1, 1, 'V');
    }
    // Faint horizontal "ruling lines" (very pale)
    for (let ly = pgY + 16; ly < pgY + pgH - 8; ly += 12) {
      for (let lx = pgX + 8; lx < pgX + pgW - 8; lx += 2) {
        px(lx, ly, 1, 1, 'V');
      }
    }
    // Line-number gutter (tiny gold dots, no numerals — empty UI)
    for (let ly = pgY + 16; ly < pgY + pgH - 8; ly += 12) {
      px(pgX + 6, ly-1, 2, 2, 'N');
    }

    // RIGHT panel — split into GOAL (top) + COMPILER OUTPUT (bottom)
    const rightX = splitX + 2, rightW = W - 14 - rightX;
    const goalH = Math.floor(mainH * 0.42);
    // GOAL frame
    rect(rightX, mainY, rightW, goalH, 'B');
    stroke(rightX, mainY, rightW, goalH, 'X');
    [[rightX+2,mainY+2],[rightX+rightW-8,mainY+2],[rightX+2,mainY+goalH-8],[rightX+rightW-8,mainY+goalH-8]].forEach(([rx,ry])=>{
      rect(rx, ry, 6, 6, 'O'); stroke(rx, ry, 6, 6, 'X');
    });
    rect(rightX+10, mainY+10, rightW-20, goalH-20, 'V');
    stroke(rightX+10, mainY+10, rightW-20, goalH-20, 'B');
    // GOAL header strip — tiny pixel-glyph "GOAL"
    rect(rightX+10, mainY+10, rightW-20, 12, 'O');
    let lx = rightX + 16;
    for (let i = 0; i < 4; i++) { rect(lx, mainY+13, 4, 6, 'C'); lx += 6; }

    // COMPILER OUTPUT frame (cobalt deep slate)
    const coY = mainY + goalH + 4;
    const coH = mainH - goalH - 4;
    rect(rightX, coY, rightW, coH, 'B');
    stroke(rightX, coY, rightW, coH, 'X');
    [[rightX+2,coY+2],[rightX+rightW-8,coY+2],[rightX+2,coY+coH-8],[rightX+rightW-8,coY+coH-8]].forEach(([rx,ry])=>{
      rect(rx, ry, 6, 6, 'O'); stroke(rx, ry, 6, 6, 'X');
    });
    rect(rightX+10, coY+10, rightW-20, coH-20, '<');
    stroke(rightX+10, coY+10, rightW-20, coH-20, 'B');
    // header strip — "COMPILER OUTPUT" placeholder
    rect(rightX+10, coY+10, rightW-20, 12, 'A');
    lx = rightX + 16;
    for (let i = 0; i < 7; i++) { rect(lx, coY+13, 4, 6, 'I'); lx += 6; }
    // Cursor-blink hint dot
    rect(rightX+18, coY+30, 4, 7, 'I');
  }, []);

  return (
    <canvas ref={ref}
      style={{ width: W*scale, height: H*scale, imageRendering: 'pixelated', display: 'block' }} />
  );
}
window.REF06 = REF06;
window.REF06_ROLES = ['X','S','B','V','C','O','L','Y','N','G','T','I','A','<'];
