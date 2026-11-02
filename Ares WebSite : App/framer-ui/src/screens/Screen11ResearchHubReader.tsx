import { ChatPanel } from '../components/ChatPanel'

export function Screen11ResearchHubReader() {
  return (
    <div className="screen-11" style={{ display: 'grid', gridTemplateColumns: '1fr 420px', height: '100vh' }}>
      {/* Left: Research Reader */}
      <div style={{
        background: 'var(--bg-secondary)',
        borderRight: '1px solid var(--border-subtle)',
        display: 'flex',
        flexDirection: 'column',
      }}>
        <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border-subtle)', display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: '12px', color: 'var(--fg-muted)', fontFamily: 'var(--font-mono)' }}>research/atoma-brain-benchmarks.md</span>
          <span style={{ background: 'var(--accent-green)', color: 'var(--bg-primary)', padding: '2px 8px', borderRadius: 'var(--radius-sm)', fontSize: '10px', fontWeight: 600 }}>LIVE</span>
        </div>
        <div style={{ flex: 1, padding: '24px', overflow: 'auto' }}>
          <article style={{ maxWidth: '800px', margin: '0 auto', lineHeight: 1.8 }}>
            <h1 style={{ fontSize: '28px', fontWeight: 700, marginBottom: '16px' }}>Atoma 12-Basin Brain Benchmarks</h1>
            <p style={{ color: 'var(--fg-muted)', marginBottom: '24px' }}>Live distillation telemetry from monster_hybrid_server (:8088)</p>

            <section style={{ marginBottom: '32px' }}>
              <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '12px', color: 'var(--accent-blue)' }}>📊 GSM8K — Exact Arithmetic</h2>
              <p style={{ color: 'var(--fg-secondary)', marginBottom: '8px' }}>
                Q32_31 precision arithmetic problems. Model solves via deterministic reasoning trace,
                verified against ground truth. 0-cell trajectory distillation captures the exact
                computational path.
              </p>
              <div style={{ background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)', padding: '16px', fontFamily: 'var(--font-mono)', fontSize: '12px', color: 'var(--fg-secondary)' }}>
{`Problem: "32_31 + 47_31 = ?"
Trace:  32 + 47 = 79 (base 31 carry)
Result: 79_31 = 79 decimal
Verify: ✓ Ground truth match
Distill: 0-cell [+32, +47] → 1-cell [sum]`}
              </div>
            </section>

            <section style={{ marginBottom: '32px' }}>
              <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '12px', color: 'var(--accent-blue)' }}>🎯 NIAH — Needle in Haystack</h2>
              <p style={{ color: 'var(--fg-secondary)', marginBottom: '8px' }}>
                Variable tracking across 128K context window. 1-cell geodesic trajectory
                shows exact attention path to needle location.
              </p>
              <div style={{ background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)', padding: '16px', fontFamily: 'var(--font-mono)', fontSize: '12px', color: 'var(--fg-secondary)' }}>
{`Haystack: 128K tokens (random UUIDs)
Needle:   "ATOMA_SECRET=42_31" at pos 87,234
Retrieval: 1-cell path [layer 17, head 3] → pos 87,234
Latency:   12ms (prefix cache warm)
Accuracy:  100% (100/100 trials)`}
              </div>
            </section>

            <section style={{ marginBottom: '32px' }}>
              <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '12px', color: 'var(--accent-blue)' }}>🔗 RULER — Multi-Hop Tracking</h2>
              <p style={{ color: 'var(--fg-secondary)', marginBottom: '8px' }}>
                Multi-hop variable dependency chains. 1-cell trajectories compose
                into 0-cell geodesics for arbitrary-depth reasoning.
              </p>
              <div style={{ background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)', padding: '16px', fontFamily: 'var(--font-mono)', fontSize: '12px', color: 'var(--fg-secondary)' }}>
{`Chain: A → B → C → D → E
Hops:  4 (max tracked: 8)
Trace: 1-cell[A→B] ∘ 1-cell[B→C] ∘ 1-cell[C→D] ∘ 1-cell[D→E]
Geodesic: 0-cell[A→E] (composed)
Verify:  Symbolic execution match ✓`}
              </div>
            </section>

            <section>
              <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '12px', color: 'var(--accent-blue)' }}>🛡️ Anti-Lobotomy Checkpointing</h2>
              <p style={{ color: 'var(--fg-secondary)', marginBottom: '8px' }}>
                Pre-compaction fold at 52% context (vs Zed 88% auto-compact).
                Post-compaction auto-restart injects checkpoint with original
                objective, verified tool ledger, active basin coordinates.
              </p>
              <div style={{ background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', borderRadius: 'var(--radius-md)', padding: '16px', fontFamily: 'var(--font-mono)', fontSize: '12px', color: 'var(--fg-secondary)' }}>
{`Checkpoint: /tmp/atoma_brain_latest_checkpoint.json
Endpoint:  GET http://127.0.0.1:8088/api/checkpoint
Contains:  • Original objective (pinned)
           • Verified tool ledger (hashed)
           • Active 12-basin coordinates
           • Recent state (last 8 turns)
Trigger:   Before context > 52%
Restore:   Auto on Zed compact / /compact`}
              </div>
            </section>
          </article>
        </div>
      </div>

      {/* Right: Agent Chat Panel */}
      <div style={{ background: 'var(--bg-secondary)', display: 'flex', flexDirection: 'column' }}>
        <ChatPanel sessionId="research-hub-session" />
      </div>
    </div>
  )
}
