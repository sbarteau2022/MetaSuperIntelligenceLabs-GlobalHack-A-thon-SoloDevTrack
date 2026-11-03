import type { ButtonHTMLAttributes, Ref } from 'react'

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  ref?: Ref<HTMLButtonElement>
  variant?: 'primary' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
}

export const Button = ({variant = 'primary', size = 'md', className = '', children, ref, ...props }: ButtonProps) => {
    const baseStyles = 'inline-flex items-center justify-center font-medium tracking-wide transition-all duration-200 rounded-lg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-500 focus-visible:ring-offset-2 focus-visible:ring-offset-obsidian-900'

    const variants = {
      primary: 'bg-cyan-500 text-fg-inverse hover:bg-cyan-400 active:scale-[0.98] hover:shadow-[0_0_30px_rgba(34,211,238,0.4)]',
      ghost: 'bg-transparent text-fg-primary border border-obsidian-600 hover:bg-obsidian-700 hover:border-cyan-500 hover:text-cyan-400',
    }

    const sizes = {
      sm: 'px-4 py-2 text-xs',
      md: 'px-6 py-3 text-sm',
      lg: 'px-8 py-4 text-base',
    }

    return (
      <button
        ref={ref}
        className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
        {...props}
      >
        {children}
      </button>
    )
  }

Button.displayName = 'Button'