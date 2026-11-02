import { Button } from '../ui/Button'
import { Pill } from '../ui/Pill'
import { useAtomaStore } from '../../store/useAtomaStore'

export default function HeroContent() {
  const { setActivePage } = useAtomaStore()
  const scrollToProof = () => {
    document.querySelector('#proof')?.scrollIntoView({ behavior: 'smooth' })
  }

  return (
    <div className="max-w-[720px]">
      <p
        data-reveal
        data-delay="0"
        data-hero-rise
        className="font-mono text-[12px] tracking-[0.2em] uppercase text-fg-muted"
      >
        AtomaLabs // Ares — SpecOps
      </p>

      <h1
        data-reveal
        data-delay="0.1"
        data-hero-rise
        className="font-display font-medium text-fg-primary leading-[1.05] tracking-tight mt-6"
        style={{ fontSize: 'clamp(48px, 8vw, 96px)' }}
      >
        Hyperbolic memory.
        <br />
        <span className="text-cyan-400">No O(n²) tax.</span>
        <br />
        Runs on a MacBook.
      </h1>

      <p
        data-reveal
        data-delay="0.2"
        data-hero-rise
        className="font-mono text-[15px] leading-relaxed text-fg-secondary mt-6"
        style={{ maxWidth: '68ch' }}
      >
        Ares SpecOps distills many open models into one unified mind map on a
        hyperbolic KV substrate — O(n log n) cache, self-contained swarm, live
        telemetry. Research-grade lab, constrained hardware.
      </p>

      <div data-reveal data-delay="0.2" data-hero-rise className="flex flex-wrap gap-3 mt-8">
        <Pill variant="verified">Harness verified</Pill>
        <Pill variant="pending">Meta hackathon Nov 4</Pill>
      </div>

      <div data-reveal data-delay="0.2" data-hero-rise className="flex flex-wrap gap-4 mt-8">
        <Button variant="primary" size="md" onClick={() => setActivePage('downloads')}>
          Download Ares
        </Button>
        <Button variant="ghost" size="md" onClick={scrollToProof}>
          View live telemetry
        </Button>
      </div>
    </div>
  )
}
