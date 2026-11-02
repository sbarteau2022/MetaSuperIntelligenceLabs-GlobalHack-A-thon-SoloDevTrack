import { GlassCard } from '../ui/GlassCard'

export interface ArchCardProps {
  index: string
  title: string
  body: string
  accent: 'cyan' | 'amber'
}

export default function ArchCard({ index, title, body, accent }: ArchCardProps) {
  const bar =
    accent === 'cyan'
      ? 'bg-gradient-to-r from-cyan-500 to-cyan-400'
      : 'bg-gradient-to-r from-amber-500 to-amber-400'

  return (
    <GlassCard padded={false} data-reveal className="relative overflow-hidden p-6">
      <div className={`absolute inset-x-0 top-0 h-px ${bar}`} aria-hidden="true" />
      <p className="font-mono text-[13px] text-fg-muted">{index}</p>
      <h3 className="mt-3 font-display text-[20px] font-semibold text-fg-primary">
        {title}
      </h3>
      <p className="mt-3 font-mono text-[14px] leading-relaxed text-fg-secondary">
        {body}
      </p>
    </GlassCard>
  )
}
