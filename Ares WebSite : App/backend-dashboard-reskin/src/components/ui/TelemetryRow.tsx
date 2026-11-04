import type { HTMLAttributes, Ref } from 'react'

export interface TelemetryRowProps extends HTMLAttributes<HTMLDivElement> {
  ref?: Ref<HTMLDivElement>
  label: string
  value: string | number
  barPercent?: number
  unit?: string
}

export const TelemetryRow = ({label, value, barPercent, unit = '', className = '', ref, ...props }: TelemetryRowProps) => {
    return (
      <div
        ref={ref}
        className={`
          flex items-center justify-between py-3 px-4 text-sm
          font-mono border-b border-glass-border
          transition-all duration-200
          hover:bg-zinc-500/10 hover:border-l-2 hover:border-l-[#f59e0b] hover:pl-[14px]
          ${className}
        `}
        {...props}
      >
        <span className="text-fg-muted uppercase tracking-wider text-[11px] flex-1 min-w-0 pr-4">
          {label}
        </span>
        <span className="text-fg-primary font-medium tabular-nums flex-shrink-0 mx-4">
          {value}{unit && <span className="text-fg-muted ml-1 text-[12px]">{unit}</span>}
        </span>
        {barPercent !== undefined && (
          <div className="w-32 h-1.5 rounded-full bg-slate-panel overflow-hidden flex-shrink-0">
            <div
              className="h-full rounded-full transition-all duration-500 ease-out"
              style={{
                width: `${Math.min(100, Math.max(0, barPercent))}%`,
                background: 'linear-gradient(90deg, rgb(6, 182, 212), rgb(34, 211, 238))',
              }}
            />
          </div>
        )}
      </div>
    )
  }

TelemetryRow.displayName = 'TelemetryRow'