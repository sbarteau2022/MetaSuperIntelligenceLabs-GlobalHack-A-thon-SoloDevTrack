import { ChatPanel } from '../components/ChatPanel'

export function Screen10MindContextGraph() {
  return (
    <div className="screen-10" style={{ display: 'grid', gridTemplateColumns: '1fr 420px', height: '100vh' }}>
      {/* Left: Mind Graph */}
      <div style={{
        background: 'var(--bg-secondary)',
        borderRight: '1px solid var(--border-subtle)',
        display: 'flex',
        flexDirection: 'column',
      }}>
        <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border-subtle)', display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: '12px', color: 'var(--fg-muted)', fontFamily: 'var(--font-mono)' }}>cogmap/concept-map.json</span>
          <span style={{ background: 'var(--accent-amber)', color: 'var(--bg-primary)', padding: '2px 8px', borderRadius: 'var(--radius-sm)', fontSize: '10px', fontWeight: 600 }}>1,247 NODES</span>
        </div>
        <div style={{ flex: 1, padding: '24px', overflow: 'auto' }}>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '12px', lineHeight: 1.8, color: 'var(--fg-secondary)' }}>
{`┌─────────────────────────────────────────────────┐
│         COGNITIVE MAP — CONCEPT GRAPH            │
│                                                   │
│   Dimensions: 12 basins × 41 languages           │
│   Nodes:    1,247 concepts                       │
│   Edges:    3,891 prerequisite links             │
│                                                   │
│   ┌─ Rust (basin 1)                              │
│   │  ├─ ownership → borrowing                    │
│   │  ├─ lifetimes → async                        │
│   │  └─ traits → generics                        │
│   │                                              │
│   ├─ TypeScript (basin 2)                        │
│   │  ├─ types → inference                        │
│   │  ├─ generics → constraints                   │
│   │  └─ decorators → metadata                    │
│   │                                              │
│   ├─ Python (basin 3)                            │
│   │  ├─ functions → decorators                   │
│   │  ├─ classes → metaclasses                    │
│   │  └─ async → await                            │
│   │                                              │
│   ├─ ... (38 more language basins)               │
│   │                                              │
│   ├─ Engineering (cross-basin)                   │
│   │  ├─ testing → property-based                 │
│   │  ├─ debugging → bisect                       │
│   │  └─ architecture → DDD                       │
│   │                                              │
│   Progress: ████████░░░░░░  42% enrolled         │
│   Next:     "async patterns in Rust" (predicted) │
└─────────────────────────────────────────────────┘

Research knowledge graph with filters, linked sources,
and node inspector. Uses cogmap_* tools for structured
learning/assessment with prerequisite tracking.`}
          </div>
        </div>
      </div>

      {/* Right: Agent Chat Panel */}
      <div style={{ background: 'var(--bg-secondary)', display: 'flex', flexDirection: 'column' }}>
        <ChatPanel sessionId="mind-graph-session" />
      </div>
    </div>
  )
}
