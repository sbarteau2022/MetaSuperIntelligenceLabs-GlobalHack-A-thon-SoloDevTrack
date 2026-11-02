import { ReactFlow, Background, Controls } from '@xyflow/react'
import type { Node, Edge } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { GlassCard } from '../ui/GlassCard'
import { Pill } from '../ui/Pill'

const CYAN = '#22d3ee'
const AMBER = '#fbbf24'
const GREEN = '#4ade80'

function dot(color: string) {
  return (
    <span
      style={{
        display: 'inline-block',
        width: 7,
        height: 7,
        borderRadius: 9999,
        background: color,
        marginRight: 8,
        flexShrink: 0,
      }}
    />
  )
}

const nodeStyle = {
  background: '#0b121a',
  border: `1px solid rgba(34,211,238,0.35)`,
  borderRadius: 10,
  color: '#e8edf5',
  fontFamily: 'JetBrains Mono, monospace',
  fontSize: 11,
  padding: '10px 14px',
}

const nodes: Node[] = [
  {
    id: 'mind-map',
    type: 'input',
    position: { x: 20, y: 170 },
    data: { label: <span style={{ display: 'inline-flex', alignItems: 'center' }}>{dot(GREEN)}mind-map</span> },
    style: nodeStyle,
  },
  {
    id: 'distill-a',
    position: { x: 200, y: 60 },
    data: { label: <span style={{ display: 'inline-flex', alignItems: 'center' }}>{dot(GREEN)}distill-a</span> },
    style: nodeStyle,
  },
  {
    id: 'distill-b',
    position: { x: 200, y: 280 },
    data: { label: <span style={{ display: 'inline-flex', alignItems: 'center' }}>{dot(GREEN)}distill-b</span> },
    style: nodeStyle,
  },
  {
    id: 'hyperbolic-kv',
    position: { x: 380, y: 170 },
    data: { label: <span style={{ display: 'inline-flex', alignItems: 'center' }}>{dot(GREEN)}hyperbolic-kv</span> },
    style: nodeStyle,
  },
  {
    id: 'swarm-harness',
    position: { x: 540, y: 60 },
    data: { label: <span style={{ display: 'inline-flex', alignItems: 'center' }}>{dot(AMBER)}swarm-harness</span> },
    style: nodeStyle,
  },
  {
    id: 'telemetry',
    position: { x: 540, y: 280 },
    data: { label: <span style={{ display: 'inline-flex', alignItems: 'center' }}>{dot(GREEN)}telemetry</span> },
    style: nodeStyle,
  },
  {
    id: 'ares-app',
    type: 'output',
    position: { x: 700, y: 170 },
    data: { label: <span style={{ display: 'inline-flex', alignItems: 'center' }}>{dot(GREEN)}ares-app</span> },
    style: nodeStyle,
  },
]

const edge = (id: string, source: string, target: string, stroke: string): Edge => ({
  id,
  source,
  target,
  animated: true,
  style: { stroke, strokeWidth: 1.5 },
})

const edges: Edge[] = [
  edge('e1', 'mind-map', 'distill-a', CYAN),
  edge('e2', 'mind-map', 'distill-b', CYAN),
  edge('e3', 'distill-a', 'hyperbolic-kv', CYAN),
  edge('e4', 'distill-b', 'hyperbolic-kv', CYAN),
  edge('e5', 'hyperbolic-kv', 'swarm-harness', CYAN),
  edge('e6', 'hyperbolic-kv', 'telemetry', CYAN),
  edge('e7', 'swarm-harness', 'telemetry', AMBER),
  edge('e8', 'telemetry', 'ares-app', CYAN),
  edge('e9', 'swarm-harness', 'ares-app', CYAN),
]

export default function SwarmDAG() {
  return (
    <GlassCard padded={false} data-reveal data-delay="0.1" className="overflow-hidden">
      <div className="flex items-center justify-between px-5 py-4 border-b border-glass-border">
        <span className="font-mono text-[11px] tracking-[0.2em] text-fg-muted">
          SWARM DAG — LIVE
        </span>
        <Pill variant="verified">8 AGENTS</Pill>
      </div>
      <div style={{ height: 420 }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          fitView
          minZoom={0.6}
          maxZoom={1.4}
          nodesDraggable
          attributionPosition="bottom-left"
          colorMode="dark"
          proOptions={{ hideAttribution: false }}
        >
          <Background gap={24} size={1} color="rgba(34,211,238,0.08)" />
          <Controls showInteractive={false} />
        </ReactFlow>
      </div>
    </GlassCard>
  )
}
