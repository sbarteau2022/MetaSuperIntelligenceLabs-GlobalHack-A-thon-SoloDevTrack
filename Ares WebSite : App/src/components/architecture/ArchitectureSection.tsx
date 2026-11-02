import { useEffect, useRef } from 'react'
import { revealOnScroll } from '../../lib/reveal'
import ArchCard from './ArchCard'

export default function ArchitectureSection() {
  const ref = useRef<HTMLElement>(null)

  useEffect(() => {
    if (!ref.current) return
    return revealOnScroll(ref.current)
  }, [])

  return (
    <section ref={ref} className="py-28">
      <div className="mx-auto max-w-[1200px] px-6">
        <p
          data-reveal
          className="font-mono text-[12px] tracking-[0.2em] text-cyan-400 uppercase"
        >
          ARCHITECTURE
        </p>
        <h2
          data-reveal
          className="mt-4 font-display font-semibold text-fg-primary"
          style={{ fontSize: 'clamp(32px, 5vw, 56px)', lineHeight: 1.1 }}
        >
          One mind map. Three load-bearing walls.
        </h2>
        <div className="mt-12 grid gap-6 md:grid-cols-3">
          <ArchCard
            index="01"
            title="Unified mind map"
            body="Many open models distilled into one map. One addressing scheme, one lineage, no adapter sprawl."
            accent="cyan"
          />
          <ArchCard
            index="02"
            title="Hyperbolic KV substrate"
            body="O(n²) cache flattened to O(n log n). Long context fits on constrained hardware."
            accent="amber"
          />
          <ArchCard
            index="03"
            title="Self-contained swarm"
            body="Harness plug-in with live telemetry. Eight agents, one receipt per run, no datacenter."
            accent="cyan"
          />
        </div>
      </div>
    </section>
  )
}
