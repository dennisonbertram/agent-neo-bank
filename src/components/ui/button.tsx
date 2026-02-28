import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '../../lib/cn'

const buttonVariants = cva(
  'inline-flex items-center justify-center font-semibold cursor-pointer border-none transition-transform duration-100 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed',
  {
    variants: {
      variant: {
        primary: 'bg-black text-white h-[56px] rounded-[999px] w-full text-[16px]',
        outline: 'bg-transparent border border-[var(--text-tertiary)] text-[var(--text-primary)] h-[56px] rounded-[999px] w-full text-[16px]',
        action: 'bg-[var(--bg-secondary)] text-[var(--text-primary)] h-[52px] rounded-[16px] flex-1 text-[15px] gap-2',
        'sm-outline': 'bg-transparent border border-[var(--text-tertiary)] text-[var(--text-primary)] h-[36px] rounded-[999px] w-auto px-4 text-[13px]',
      },
    },
    defaultVariants: {
      variant: 'primary',
    },
  }
)

interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  children: React.ReactNode
}

export function Button({ className, variant, children, ...props }: ButtonProps) {
  return (
    <button className={cn(buttonVariants({ variant }), className)} {...props}>
      {children}
    </button>
  )
}
