import { Check } from 'lucide-react'
import { cn } from '../../lib/cn'

interface SuccessCheckProps {
  className?: string
}

export function SuccessCheck({ className }: SuccessCheckProps) {
  return (
    <div
      className={cn(
        'w-[64px] h-[64px] bg-[var(--accent-green)] rounded-full flex items-center justify-center mx-auto',
        className
      )}
    >
      <Check size={32} color="white" strokeWidth={3} />
    </div>
  )
}
