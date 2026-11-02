import { forwardRef } from 'react'
import type { HTMLAttributes } from 'react'

export interface GlassCardProps extends HTMLAttributes<HTMLDivElement> {
  padded?: boolean
}

export const GlassCard = forwardRef<HTMLDivElement, GlassCardProps>(
  ({ padded = true, className = '', children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={`
          glass-card
          ${padded ? 'p-6' : ''}
          ${className}
        `}
        {...props}
      >
        {children}
      </div>
    )
  }
)

GlassCard.displayName = 'GlassCard'