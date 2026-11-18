import type { ButtonHTMLAttributes, Ref } from 'react'

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  ref?: Ref<HTMLButtonElement>
  variant?: 'primary' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
}

export const Button = ({variant = 'primary', size = 'md', className = '', children, ref, ...props }: ButtonProps) => {
    const baseStyles = 'inline-flex items-center justify-center font-medium tracking-wide transition-all duration-200 rounded-lg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#f59e0b] focus-visible:ring-offset-2 focus-visible:ring-offset-[#26262b]'

    const variants = {
      primary: 'bg-[#f59e0b] text-black hover:bg-[#d97706] active:scale-[0.98]',
      ghost: 'bg-transparent text-fg-primary border border-[#45454d] hover:bg-[#34343a] hover:border-[#f59e0b] hover:text-zinc-400',
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