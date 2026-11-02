import { GlassCard } from '../ui/GlassCard'
import { TelemetryRow } from '../ui/TelemetryRow'
import { Pill } from '../ui/Pill'
import { useTelemetry } from '../../hooks/useTelemetry'

export default function TelemetryTable() {
  const metrics = useTelemetry()

  return (
    <GlassCard padded={false} data-reveal className="overflow-hidden">
      <div className="flex items-center justify-between px-5 py-4 border-b border-glass-border">
        <div className="flex items-center gap-2.5">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-60" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-green-400" />
          </span>
          <span className="font-mono text-[11px] tracking-[0.2em] text-fg-muted">
            LIVE TELEMETRY
          </span>
        </div>
        <Pill variant="verified">STREAMING</Pill>
      </div>

      <div>
        {metrics.map((m) => (
          <TelemetryRow
            key={m.label}
            label={m.label}
            value={m.value}
            barPercent={m.bar}
          />
        ))}
      </div>

      <div className="px-5 py-4 font-mono text-[11px] text-fg-muted">
        substrate: hyperbolic-kv · O(n log n) · local-first
      </div>
    </GlassCard>
  )
}
