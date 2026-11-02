import { useEffect, useState } from 'react'

export interface TelemetryMetric {
  label: string
  value: string
  /** 0–100 fill for the metric bar. Omit for non-numeric rows. */
  bar?: number
}

interface Drift {
  label: string
  min: number
  max: number
  decimals: number
  format: (v: number) => string
  barScale: (v: number) => number
  static?: false
}

const DRIFTS: Drift[] = [
  {
    label: 'latency_p50',
    min: 9,
    max: 16,
    decimals: 0,
    format: (v) => `${v.toFixed(0)} ms`,
    barScale: (v) => (v / 60) * 100,
  },
  {
    label: 'latency_p99',
    min: 38,
    max: 58,
    decimals: 0,
    format: (v) => `${v.toFixed(0)} ms`,
    barScale: (v) => (v / 120) * 100,
  },
  {
    label: 'throughput',
    min: 2100,
    max: 2600,
    decimals: 0,
    format: (v) => `${Math.round(v).toLocaleString()} tok/s`,
    barScale: (v) => (v / 3000) * 100,
  },
  {
    label: 'memory_resident',
    min: 1.05,
    max: 1.45,
    decimals: 2,
    format: (v) => `${v.toFixed(2)} GB`,
    barScale: (v) => (v / 8) * 100,
  },
  {
    label: 'cache_hit',
    min: 93.1,
    max: 95.8,
    decimals: 1,
    format: (v) => `${v.toFixed(1)} %`,
    barScale: (v) => v,
  },
  {
    label: 'agents_active',
    min: 5,
    max: 7,
    decimals: 0,
    format: (v) => `${Math.round(v)} / 8`,
    barScale: (v) => (v / 8) * 100,
  },
]

function jitter(v: number, min: number, max: number): number {
  const span = max - min
  const next = v + (Math.random() - 0.5) * span * 0.35
  return Math.min(max, Math.max(min, next))
}

/**
 * Simulated live telemetry feed. Random-walks every 1.6s, pauses when the
 * tab is hidden. Replace with the real harness WebSocket when it lands —
 * the TelemetryMetric shape is the contract.
 */
export function useTelemetry(): TelemetryMetric[] {
  const [vals, setVals] = useState<number[]>(() =>
    DRIFTS.map((d) => d.min + Math.random() * (d.max - d.min))
  )

  useEffect(() => {
    const id = window.setInterval(() => {
      if (document.hidden) return
      setVals((prev) => prev.map((v, i) => jitter(v, DRIFTS[i].min, DRIFTS[i].max)))
    }, 1600)
    return () => window.clearInterval(id)
  }, [])

  return DRIFTS.map((d, i) => ({
    label: d.label,
    value: d.format(vals[i]),
    bar: Math.min(100, Math.max(0, d.barScale(vals[i]))),
  }))
}
