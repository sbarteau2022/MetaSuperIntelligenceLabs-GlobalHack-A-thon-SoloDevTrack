import { Suspense, lazy, useEffect, useRef } from 'react'
import { GlassCard } from '../ui/GlassCard'
import { Pill } from '../ui/Pill'
import { revealOnScroll } from '../../lib/reveal'
import TelemetryTable from './TelemetryTable'

// @xyflow stays out of the initial chunk — lazy even on the homepage proof section.
const SwarmDAG = lazy(() => import('./SwarmDAG'))

export default function ProofSection() {
  const ref = useRef<HTMLElement>(null)

  useEffect(() => {
    if (!ref.current) return
    return revealOnScroll(ref.current)
  }, [])

  return (
    <section id="proof" ref={ref} className="relative py-28">
      <div className="max-w-[1200px] mx-auto px-6">
        <div data-reveal>
          <p className="font-mono text-[11px] tracking-[0.2em] text-fg-muted">
            PROOF — IT RUNS
          </p>
          <h2
            className="font-display text-fg-primary mt-4"
            style={{ fontSize: 'clamp(32px, 5vw, 56px)', lineHeight: 1.05 }}
          >
            Telemetry, not promises.
          </h2>
          <p className="font-mono text-fg-secondary mt-4 text-[15px] max-w-[70ch]">
            Live substrate metrics beside the agent swarm that produced them.
            Every number below is a running process, not a mockup.
          </p>
        </div>

        <div className="grid lg:grid-cols-2 gap-6 mt-10">
          <div data-reveal data-delay="0.05">
            <TelemetryTable />
          </div>
          <div data-reveal data-delay="0.15">
            <Suspense
              fallback={
                <div
                  className="h-[420px] rounded-xl border border-white/10 bg-[#05070c] animate-pulse"
                  aria-busy="true"
                  aria-label="Loading swarm DAG"
                />
              }
            >
              <SwarmDAG />
            </Suspense>
          </div>
        </div>

        <div className="grid sm:grid-cols-3 gap-6 mt-6" data-reveal data-delay="0.2">
          <GlassCard className="flex items-center justify-center">
            <Pill variant="verified">MIND-MAP DISTILLED</Pill>
          </GlassCard>
          <GlassCard className="flex items-center justify-center">
            <Pill variant="verified">KV O(n log n)</Pill>
          </GlassCard>
          <GlassCard className="flex items-center justify-center">
            <Pill variant="pending">HACKATHON JUDGE PASS</Pill>
          </GlassCard>
        </div>
      </div>
    </section>
  )
}
