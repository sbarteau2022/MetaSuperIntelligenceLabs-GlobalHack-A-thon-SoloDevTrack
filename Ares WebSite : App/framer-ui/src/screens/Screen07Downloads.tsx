export function Screen07Downloads() {
  const downloads = [
    { title: 'Atoma Desktop App', version: 'v0.1.0-alpha', platform: 'macOS (Apple Silicon)', size: '142 MB', description: 'Tauri 2 native app with embedded Model Home harness' },
    { title: 'Model Home CLI', version: 'v2026.10', platform: 'Cross-platform', size: '8.4 MB', description: 'Standalone `home` binary for harness operations' },
    { title: 'Research Harness SDK', version: 'v1.0.0', platform: 'npm / PyPI', size: '—', description: 'TypeScript & Python SDKs for custom integrations' },
    { title: 'Framer Prototype', version: '2026-10-31', platform: 'Web', size: '—', description: 'Static visual prototype (this app\'s source)' },
  ]

  return (
    <div className="screen-07" style={{ padding: '40px', maxWidth: '1200px', margin: '0 auto', height: '100vh', overflow: 'auto' }}>
      <header style={{ marginBottom: '32px' }}>
        <h1 style={{ fontSize: '32px', fontWeight: 700, marginBottom: '8px' }}>Downloads</h1>
        <p style={{ color: 'var(--fg-muted)', fontSize: '16px' }}>Platform availability, research hub, and harness overview</p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(350px, 1fr))', gap: '24px' }}>
        {downloads.map((item, i) => (
          <div key={i} style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px', transition: 'var(--transition-normal)' }}>
            <div style={{ display: 'flex', gap: '8px', marginBottom: '12px', flexWrap: 'wrap' }}>
              <span style={{ background: 'var(--accent-blue-glow)', color: 'var(--accent-blue)', padding: '2px 8px', borderRadius: 'var(--radius-sm)', fontSize: '11px', fontWeight: 500 }}>{item.platform}</span>
              <span style={{ background: 'var(--bg-primary)', color: 'var(--fg-muted)', padding: '2px 8px', borderRadius: 'var(--radius-sm)', fontSize: '11px' }}>{item.size}</span>
            </div>
            <h3 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '4px' }}>{item.title}</h3>
            <p style={{ color: 'var(--accent-blue)', fontSize: '13px', fontWeight: 500, marginBottom: '12px' }}>{item.version}</p>
            <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>{item.description}</p>
            <button style={{ padding: '10px 16px', borderRadius: 'var(--radius-md)', background: 'transparent', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '13px', fontWeight: 500, cursor: 'pointer', width: '100%', transition: 'var(--transition-fast)' }}>
              Download
            </button>
          </div>
        ))}
      </div>

      <section style={{ marginTop: '48px', paddingTop: '24px', borderTop: '1px solid var(--border-subtle)' }}>
        <h2 style={{ fontSize: '20px', fontWeight: 600, marginBottom: '16px' }}>🔬 Research Hub</h2>
        <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, maxWidth: '800px' }}>
          The Atoma Research Hub provides access to curated papers, benchmark results, and engineering
          runbooks. Connect via the Model Home harness to query the Postgres knowledge base (PAMI)
          with hybrid semantic + keyword search across 7 collections.
        </p>
      </section>
    </div>
  )
}
