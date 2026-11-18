import type { HTMLAttributes, Ref } from 'react'

export interface PillProps extends HTMLAttributes<HTMLSpanElement> {
  ref?: Ref<HTMLSpanElement>
  variant?: 'verified' | 'blocked' | 'pending'
}

export const Pill = ({variant = 'verified', className = '', children, ref, ...props }: PillProps) => {
    const variants = {
      verified: 'bg-green-500/15 text-green-400 border-green-500/30',
      blocked: 'bg-red-500/15 text-red-400 border-red-500/30',
      pending: 'bg-amber-500/15 text-amber-400 border-amber-500/30',
    }

    return (
      <span
        ref={ref}
        className={`
          inline-flex items-center px-3 py-1 rounded-full text-xs font-medium tracking-wider uppercase
          border ${variants[variant]} ${className}
        `}
        {...props}
      >
        {children}
      </span>
    )
  }

Pill.displayName = 'Pill'