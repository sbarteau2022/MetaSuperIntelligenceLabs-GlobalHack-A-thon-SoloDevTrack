import type { HTMLAttributes, Ref } from 'react'

export interface GlassCardProps extends HTMLAttributes<HTMLDivElement> {
  ref?: Ref<HTMLDivElement>
  padded?: boolean
}

export const GlassCard = ({padded = true, className = '', children, ref, ...props }: GlassCardProps) => {
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

GlassCard.displayName = 'GlassCard'