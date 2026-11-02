export function Screen04ResearchLab() {
  return (
    <div className="screen-04" style={{ padding: '40px', maxWidth: '1200px', margin: '0 auto', height: '100vh', overflow: 'auto' }}>
      <header style={{ marginBottom: '32px' }}>
        <h1 style={{ fontSize: '32px', fontWeight: 700, marginBottom: '8px' }}>Research Lab</h1>
        <p style={{ color: 'var(--fg-muted)', fontSize: '16px' }}>Research runbooks, experimentation, and evidence workflows</p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(350px, 1fr))', gap: '24px' }}>
        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>🔬 Research Runbooks</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            Structured investigation templates with pre-registered hypotheses and validation criteria:
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li>Design Spec → Preregister → Test Suites → Implement → Adversarial</li>
            <li>Claim sealing with evidence requirements</li>
            <li>Mutation testing for test suite quality</li>
            <li>Failure recording as first-class entries</li>
          </ul>
        </section>

        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>📊 Evidence Workflows</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            Hybrid RAG (HNSW + GIN) with reciprocal rank fusion for grounded claims:
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li>PostgreSQL 17 + pgvector at 127.0.0.1:5433</li>
            <li>Collections: papers, research, knowledge, models, platforms, build, engineering</li>
            <li>model2vec embedder (potion-code-16M default)</li>
            <li>Idempotent ingestion with change detection</li>
          </ul>
        </section>

        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>🧪 Experimentation</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            Controlled experimentation with durable state:
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li>Lima microVM (6 CPU, 16 GiB, 80 GiB)</li>
            <li>41 language toolchains isolated</li>
            <li>Language-specific validation commands</li>
            <li>Mutation testing (planned)</li>
          </ul>
        </section>

        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>📝 Documentation</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            Zero-dependency doc generation from Markdown:
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li>Styled HTML output</li>
            <li>DOCX generation</li>
            <li>PDF via VM browser rendering</li>
            <li>Artifact storage in workspace</li>
          </ul>
        </section>
      </div>
    </div>
  )
}
