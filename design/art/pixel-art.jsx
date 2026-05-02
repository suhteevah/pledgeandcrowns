// PixelArt — render a 2D grid of palette-role characters to a crisp <canvas>.
// Grid is an array of equal-length strings; each char keys window.PALETTE.
// '.' or ' ' = transparent. Lighting + outline rules are the artist's job;
// this just renders pixels honestly.

function PixelArt({ grid, scale = 1, style, title }) {
  const ref = React.useRef(null);
  const w = grid[0].length;
  const h = grid.length;

  React.useEffect(() => {
    const cv = ref.current;
    if (!cv) return;
    cv.width = w;
    cv.height = h;
    const ctx = cv.getContext('2d');
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, w, h);
    for (let y = 0; y < h; y++) {
      const row = grid[y];
      for (let x = 0; x < w; x++) {
        const ch = row[x];
        const col = window.PALETTE[ch];
        if (!col) continue;
        ctx.fillStyle = col.hex;
        ctx.fillRect(x, y, 1, 1);
      }
    }
  }, [grid, w, h]);

  return (
    <canvas
      ref={ref}
      title={title}
      style={{
        width: w * scale,
        height: h * scale,
        imageRendering: 'pixelated',
        display: 'block',
        ...style,
      }}
    />
  );
}

// PixelLayered — multi-layer compositor. Each layer is { grid, dx, dy }.
// Useful for backgrounds + foreground sprites in the world map / cinematic.
function PixelLayered({ width, height, layers, scale = 1, background, style }) {
  const ref = React.useRef(null);

  React.useEffect(() => {
    const cv = ref.current;
    if (!cv) return;
    cv.width = width;
    cv.height = height;
    const ctx = cv.getContext('2d');
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, width, height);
    if (background) {
      ctx.fillStyle = background;
      ctx.fillRect(0, 0, width, height);
    }
    for (const layer of layers) {
      const { grid, dx = 0, dy = 0 } = layer;
      const lh = grid.length;
      for (let y = 0; y < lh; y++) {
        const row = grid[y];
        for (let x = 0; x < row.length; x++) {
          const ch = row[x];
          const col = window.PALETTE[ch];
          if (!col) continue;
          ctx.fillStyle = col.hex;
          ctx.fillRect(x + dx, y + dy, 1, 1);
        }
      }
    }
  }, [width, height, layers, background]);

  return (
    <canvas
      ref={ref}
      style={{
        width: width * scale,
        height: height * scale,
        imageRendering: 'pixelated',
        display: 'block',
        ...style,
      }}
    />
  );
}

Object.assign(window, { PixelArt, PixelLayered });
