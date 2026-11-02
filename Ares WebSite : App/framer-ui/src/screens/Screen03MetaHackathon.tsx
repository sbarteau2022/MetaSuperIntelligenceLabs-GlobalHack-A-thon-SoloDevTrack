export function Screen03MetaHackathon() {
  return (
    <div className="screen-03" style={{ padding: '40px', maxWidth: '1200px', margin: '0 auto', height: '100vh', overflow: 'auto' }}>
      <header style={{ marginBottom: '32px' }}>
        <h1 style={{ fontSize: '32px', fontWeight: 700, marginBottom: '8px' }}>Meta Hackathon</h1>
        <p style={{ color: 'var(--fg-muted)', fontSize: '16px' }}>ARES SpecOps research prototype and evidence-led workflow</p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(350px, 1fr))', gap: '24px' }}>
        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>🏆 ARES SpecOps</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            The Autonomous Research & Engineering System for Special Operations. A prototype for
            running autonomous research loops with evidence-gated claims, adversarial pressure passes,
            and durable checkpointing.
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li>Split-phase coding loop (THINK → ACT → VERIFY & FOLD)</li>
            <li>Revenue research loop (Pillars 3 & 6)</li>
            <li>Defensive security assurance suite</li>
            <li>48GB KV/prefill optimizer</li>
            <li>Live Atoma 12-basin brain operators</li>
          </ul>
        </section>

        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>📋 Runbooks</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            Structured workflows for repeatable engineering tasks:
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li><strong>orient</strong> — Baseline assessment</li>
            <li><strong>fix-and-verify</strong> — Bug fix with verification</li>
            <li><strong>investigate-failure</strong> — Root cause analysis</li>
            <li><strong>review-diff</strong> — Code review automation</li>
            <li><strong>web-check</strong> — External research validation</li>
            <li><strong>status</strong> — System health check</li>
          </ul>
        </section>

        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>🧠 Brain Benchmarks</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            12-basin active mind operators for continuous evaluation:
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li>GSM8K — Exact Q32_31 arithmetic</li>
            <li>NIAH — Needle-in-a-haystack retrieval</li>
            <li>RULER — Multi-hop variable tracking</li>
            <li>Trajectory distillation — 0-cell/1-cell geodesics</li>
            <li>Anti-lobotomy checkpointing — Pre/post compaction</li>
          </ul>
        </section>

        <section style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-lg)', padding: '24px' }}>
          <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>⚡ Superpowers</h2>
          <p style={{ color: 'var(--fg-secondary)', lineHeight: 1.6, marginBottom: '16px' }}>
            Revenue research pillars for product discovery:
          </p>
          <ul style={{ color: 'var(--fg-secondary)', lineHeight: 2 }}>
            <li>Pillar 3: ResearchPlanner + Synthesizer</li>
            <li>Pillar 6: CrossPollination + PrototypeFactory</li>
            <li>Iterative opportunity ranking</li>
            <li>Dossier generation (Markdown)</li>
            <li>Integration with Atoma Brain (:8088)</li>
          </ul>
        </section>
      </div>
    </div>
  )
}
