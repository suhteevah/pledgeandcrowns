// PaletteLegend — full 32-color Heraldic Code palette banner that sits at the
// top of the canvas. Grouped by harmonic role.

function PaletteLegend() {
  const G = window.PALETTE_GROUPS;
  const P = window.PALETTE;

  const Group = ({ id }) => {
    const g = G[id];
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <div style={{
          fontSize: 10, fontWeight: 700, letterSpacing: 0.7,
          color: '#FCEFC8', opacity: 0.7,
          textTransform: 'uppercase', fontFamily: 'ui-monospace, monospace',
        }}>{g.label}</div>
        <div style={{ display: 'flex', gap: 1 }}>
          {g.keys.map(k => {
            const c = P[k];
            return (
              <div key={k} title={`${c.name} · ${c.hex} · char "${k}"`}
                style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', minWidth: 64 }}>
                <div style={{
                  width: 64, height: 36, background: c.hex,
                  borderTop: '1px solid rgba(255,255,255,0.08)',
                  borderRight: '1px solid rgba(0,0,0,0.5)',
                  position: 'relative',
                }}>
                  <span style={{
                    position: 'absolute', top: 2, left: 4,
                    fontSize: 9, fontWeight: 700, color: 'rgba(255,255,255,0.4)',
                    fontFamily: 'ui-monospace, monospace',
                  }}>{k}</span>
                </div>
                <div style={{
                  fontSize: 9, color: '#FCEFC8', marginTop: 3, fontFamily: 'ui-monospace, monospace',
                }}>{c.name}</div>
                <div style={{
                  fontSize: 8, color: 'rgba(252,239,200,0.5)', fontFamily: 'ui-monospace, monospace',
                }}>{c.hex}</div>
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  return (
    <div style={{
      position: 'relative',
      margin: '0 60px 56px',
      padding: '20px 28px 24px',
      background: 'linear-gradient(180deg, #1B0810 0%, #0d0608 100%)',
      border: '2px solid #3E1220',
      boxShadow: '0 8px 24px rgba(0,0,0,0.4), inset 0 0 0 1px rgba(210,165,63,0.3)',
      width: 'fit-content',
    }}>
      {/* Heraldic header */}
      <div style={{
        display: 'flex', alignItems: 'baseline', gap: 16, marginBottom: 18,
        paddingBottom: 14, borderBottom: '1px solid rgba(210,165,63,0.25)',
      }}>
        <div>
          <div style={{
            fontFamily: 'Georgia, "Times New Roman", serif',
            fontSize: 24, fontWeight: 700, color: '#D2A53F', letterSpacing: -0.3,
          }}>Heraldic Code</div>
          <div style={{
            fontSize: 11, color: '#C56883', fontFamily: 'ui-monospace, monospace',
            letterSpacing: 0.5, marginTop: 2,
          }}>Palette v2.0 · 32 colors · split-complementary @ 350° + 140°/178°</div>
        </div>
        <div style={{ flex: 1 }} />
        <div style={{
          fontSize: 10, color: 'rgba(252,239,200,0.5)', fontFamily: 'ui-monospace, monospace',
        }}>
          ANCHOR <span style={{ color: '#6B1F35', fontWeight: 700 }}>OXBLOOD #6B1F35</span>
          {' · '} CROWN <span style={{ color: '#D2A53F', fontWeight: 700 }}>OLD GOLD #D2A53F</span>
        </div>
      </div>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '18px 28px' }}>
        <Group id="burgundy" />
        <Group id="gold" />
        <Group id="teal" />
        <Group id="forest" />
        <Group id="neutral" />
        <Group id="cool" />
        <Group id="magic" />
        <Group id="alarm" />
      </div>
    </div>
  );
}

window.PaletteLegend = PaletteLegend;
