// RefArtboard — wraps a pixel-art reference with all the bible-discipline
// extras: REF-NN badge, native+scaled views, palette-roles-used strip,
// Style Suffix collapsible, signoff checklist.

const RA_BG = '#1a1614'; // dark display chamber so light pixels pop
const RA_PARCH = '#FCEFC8';
const RA_INK = '#1B0810';
const RA_BORDER = '#3E3833';
const RA_OXBLOOD = '#6B1F35';
const RA_GOLD = '#D2A53F';

function RefArtboard({
  ref_id,           // 'REF-01'
  title,            // 'Player character (default)'
  resolution,       // '32×32'
  zone,             // optional zone string
  rolesUsed,        // array of palette keys actually present
  promptText,       // the per-REF prompt (the bracketed bit)
  children,         // the rendered pixel art component
  nativeSize,       // [w, h] in source px
  displayScale,     // primary display multiplier
  notes,            // [string] artist commentary
  status,           // 'pending' | 'approved' | 'changes-requested'
}) {
  const [w, h] = nativeSize;
  const [showPrompt, setShowPrompt] = React.useState(false);
  const [signoff, setSignoff] = React.useState({
    palette: false, outline: false, lighting: false, grid: false, silhouette: false,
  });

  const statusBadge = {
    pending:  { bg: '#5E4116', fg: '#F0D27D', label: 'Awaiting Matt' },
    approved: { bg: '#27502E', fg: '#C9DC97', label: '✓ Approved'   },
    'changes-requested': { bg: '#982D52', fg: '#EBC2CC', label: 'Changes' },
  }[status || 'pending'];

  return (
    <div style={{
      width: '100%', height: '100%',
      background: RA_PARCH,
      display: 'flex', flexDirection: 'column',
      fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
      color: RA_INK,
      overflow: 'hidden',
    }}>
      {/* ─── Header ─────────────────────────── */}
      <div style={{
        padding: '14px 18px 12px', borderBottom: `2px solid ${RA_BORDER}`,
        background: `linear-gradient(180deg, ${RA_PARCH} 0%, #f6e6b9 100%)`,
        position: 'relative',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 4 }}>
          <span style={{
            background: RA_OXBLOOD, color: RA_PARCH, padding: '3px 8px',
            borderRadius: 2, fontSize: 11, fontWeight: 700, letterSpacing: 0.5,
          }}>{ref_id}</span>
          <span style={{
            background: statusBadge.bg, color: statusBadge.fg, padding: '3px 8px',
            borderRadius: 2, fontSize: 10, fontWeight: 600, letterSpacing: 0.4,
          }}>{statusBadge.label}</span>
          <span style={{ flex: 1 }} />
          <span style={{
            fontSize: 11, color: RA_BORDER, fontWeight: 600,
            background: '#fff7df', padding: '3px 7px', border: `1px solid ${RA_BORDER}`,
          }}>{resolution} · {w}×{h}px</span>
        </div>
        <div style={{
          fontFamily: 'Georgia, "Times New Roman", serif',
          fontSize: 18, fontWeight: 700, color: RA_INK, letterSpacing: -0.2,
        }}>{title}</div>
        {zone && (
          <div style={{ fontSize: 11, color: RA_BORDER, marginTop: 2, fontStyle: 'italic' }}>
            zone palette · {zone}
          </div>
        )}
      </div>

      {/* ─── Display Chamber: native + scaled ───── */}
      <div style={{
        background: RA_BG,
        backgroundImage: `linear-gradient(45deg, #221d1a 25%, transparent 25%), linear-gradient(-45deg, #221d1a 25%, transparent 25%), linear-gradient(45deg, transparent 75%, #221d1a 75%), linear-gradient(-45deg, transparent 75%, #221d1a 75%)`,
        backgroundSize: '12px 12px',
        backgroundPosition: '0 0, 0 6px, 6px -6px, -6px 0',
        padding: 18,
        flex: 1,
        display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 18,
        borderBottom: `2px solid ${RA_BORDER}`,
        position: 'relative',
      }}>
        {/* Scaled-up display */}
        <div style={{
          padding: 6, background: '#0d0a09',
          boxShadow: 'inset 0 0 0 1px #2a2522, 0 4px 12px rgba(0,0,0,0.4)',
        }}>
          <div style={{ width: w * displayScale, height: h * displayScale, position: 'relative' }}>
            {children}
          </div>
        </div>

        {/* Native + label badge */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6, alignItems: 'center' }}>
          <div style={{
            padding: 4, background: '#0d0a09',
            boxShadow: 'inset 0 0 0 1px #2a2522',
          }}>
            <div style={{ width: w, height: h, position: 'relative' }}>
              {React.cloneElement(children, { scale: 1 })}
            </div>
          </div>
          <div style={{ fontSize: 9, color: '#7A7064', letterSpacing: 0.5 }}>1×</div>
          <div style={{ fontSize: 9, color: RA_GOLD, letterSpacing: 0.5, marginTop: 'auto' }}>
            {displayScale}× ←
          </div>
        </div>
      </div>

      {/* ─── Palette roles strip ────────────── */}
      {rolesUsed && rolesUsed.length > 0 && (
        <div style={{
          padding: '8px 12px', borderBottom: `1px solid ${RA_BORDER}`,
          background: '#f3e3b6',
        }}>
          <div style={{ fontSize: 9, fontWeight: 700, color: RA_BORDER, letterSpacing: 0.8, marginBottom: 5 }}>
            ON-PALETTE COLORS USED ({rolesUsed.length}/32)
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
            {rolesUsed.map((k) => {
              const c = window.PALETTE[k];
              if (!c) return null;
              return (
                <div key={k} title={`${c.name} · ${c.hex}`}
                  style={{
                    display: 'flex', alignItems: 'center', gap: 4,
                    background: '#fff7df', padding: '2px 5px',
                    border: `1px solid ${RA_BORDER}`, fontSize: 9,
                  }}>
                  <span style={{
                    width: 10, height: 10, background: c.hex,
                    border: '1px solid rgba(0,0,0,0.3)',
                  }} />
                  <span style={{ color: RA_INK, fontWeight: 600 }}>{c.name}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* ─── Signoff checklist ────────────── */}
      <div style={{ padding: '10px 14px', background: '#fff7df', borderBottom: `1px solid ${RA_BORDER}` }}>
        <div style={{ fontSize: 9, fontWeight: 700, color: RA_BORDER, letterSpacing: 0.8, marginBottom: 6 }}>
          MATT'S SIGNOFF — CHECK ONCE VERIFIED
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px 10px' }}>
          {[
            ['palette',    'On-palette (32-color set)'],
            ['outline',    '1px Coalblack outline (not #000)'],
            ['lighting',   'Top-left 45° lighting'],
            ['grid',       'Pixel grid · no AA'],
            ['silhouette', 'Readable silhouette'],
          ].map(([key, label]) => (
            <label key={key} style={{
              display: 'flex', alignItems: 'center', gap: 6, fontSize: 11,
              cursor: 'pointer', color: signoff[key] ? RA_INK : RA_BORDER,
              textDecoration: signoff[key] ? 'line-through' : 'none',
            }}>
              <input
                type="checkbox"
                checked={signoff[key]}
                onChange={(e) => setSignoff({ ...signoff, [key]: e.target.checked })}
                style={{ accentColor: RA_OXBLOOD }}
              />
              {label}
            </label>
          ))}
        </div>
      </div>

      {/* ─── Notes + prompt ──────────────── */}
      <div style={{ padding: '10px 14px', flex: '0 0 auto', background: RA_PARCH }}>
        {notes && notes.length > 0 && (
          <div style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 9, fontWeight: 700, color: RA_BORDER, letterSpacing: 0.8, marginBottom: 4 }}>
              ARTIST NOTES
            </div>
            {notes.map((n, i) => (
              <div key={i} style={{
                fontSize: 11, lineHeight: 1.5, color: RA_INK, marginBottom: 3,
                fontFamily: 'Georgia, serif',
              }}>· {n}</div>
            ))}
          </div>
        )}
        <button
          onClick={() => setShowPrompt(!showPrompt)}
          style={{
            width: '100%', padding: '6px 10px', background: RA_INK, color: RA_GOLD,
            border: 'none', fontSize: 10, fontWeight: 700, letterSpacing: 1,
            fontFamily: 'inherit', cursor: 'pointer', textAlign: 'left',
            display: 'flex', justifyContent: 'space-between', alignItems: 'center',
          }}
        >
          <span>STYLE SUFFIX + PER-REF PROMPT</span>
          <span>{showPrompt ? '▼' : '▶'}</span>
        </button>
        {showPrompt && (
          <pre style={{
            margin: '4px 0 0', padding: '8px 10px', background: '#1B0810', color: '#C9DC97',
            fontSize: 9.5, lineHeight: 1.4, maxHeight: 180, overflow: 'auto',
            whiteSpace: 'pre-wrap', wordBreak: 'break-word',
            fontFamily: 'ui-monospace, monospace',
          }}>{promptText}</pre>
        )}
      </div>
    </div>
  );
}

window.RefArtboard = RefArtboard;
