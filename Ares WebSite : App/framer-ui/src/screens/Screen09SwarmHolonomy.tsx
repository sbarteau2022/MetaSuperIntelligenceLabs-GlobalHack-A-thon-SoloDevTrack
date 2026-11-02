import { ChatPanel } from '../components/ChatPanel'

export function Screen09SwarmHolonomy() {
  return (
    <div className="screen-09" style={{ display: 'grid', gridTemplateColumns: '1fr 420px', height: '100vh' }}>
      {/* Left: Swarm Visualization */}
      <div style={{
        background: 'var(--bg-secondary)',
        borderRight: '1px solid var(--border-subtle)',
        display: 'flex',
        flexDirection: 'column',
      }}>
        <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border-subtle)', display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: '12px', color: 'var(--fg-muted)', fontFamily: 'var(--font-mono)' }}>swarm/holonomy-dag.json</span>
          <span style={{ background: 'var(--accent-blue)', color: 'white', padding: '2px 8px', borderRadius: 'var(--radius-sm)', fontSize: '10px', fontWeight: 600 }}>12 AGENTS</span>
        </div>
        <div style={{ flex: 1, padding: '24px', overflow: 'auto' }}>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', lineHeight: 1.8, color: 'var(--fg-secondary)' }}>
{`┌─────────────────────────────────────────────────┐
│         SWARM HOLOONOMY DAG                       │
│                                                   │
│   [COORDINATOR]                                   │
│        │                                          │
│        ├─▶ [RESEARCHER] ──▶ [KB: papers]         │
│        │           │                              │
│        │           └─▶ [SYNTHESIZER]             │
│        │                    │                     │
│        ├─▶ [CODER] ──▶ [RUN_SHELL]               │
│        │           │                              │
│        │           └─▶ [TESTER]                  │
│        │                                          │
│        ├─▶ [ANALYST] ──▶ [KB: engineering]       │
│        │           │                              │
│        │           └─▶ [CRITIC]                  │
│        │                                          │
│        └─▶ [LIBRARIAN] ──▶ [MEMORY]              │
│                                                   │
│   Status:  ████████████░░░░  68% complete        │
│   Turn:    7 of 12                                │
│   Tokens:  42,341 / 131,072                       │
└─────────────────────────────────────────────────┘

Agent coordination graph with execution timeline
and system telemetry. Each node = specialist role
with curated tool subset (principle of least privilege).

Semaphore-limited to 2 concurrent (matches N=2 server).
Sub-agents return concise reports to parent.`}
          </div>
        </div>
      </div>

      {/* Right: Agent Chat Panel */}
      <div style={{ background: 'var(--bg-secondary)', display: 'flex', flexDirection: 'column' }}>
        <ChatPanel sessionId="swarm-session" />
      </div>
    </div>
  )
}
