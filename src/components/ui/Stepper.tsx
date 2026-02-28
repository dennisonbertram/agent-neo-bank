import { Minus, Plus } from 'lucide-react'
import { cn } from '../../lib/cn'

interface StepperProps {
  label: string
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
  prefix?: string
  className?: string
}

export function Stepper({
  label,
  value,
  onChange,
  min = 0,
  max = 10000,
  step = 1,
  prefix = '$',
  className,
}: StepperProps) {
  return (
    <div className={cn('flex items-center justify-between', className)}>
      <span className="text-[15px] font-medium text-[var(--text-primary)]">{label}</span>
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={() => onChange(Math.max(min, value - step))}
          disabled={value <= min}
          className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] flex items-center justify-center border-none cursor-pointer disabled:opacity-30"
        >
          <Minus size={16} />
        </button>
        <span className="text-[17px] font-semibold text-[var(--text-primary)] min-w-[60px] text-center">
          {prefix}{value.toFixed(2)}
        </span>
        <button
          type="button"
          onClick={() => onChange(Math.min(max, value + step))}
          disabled={value >= max}
          className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] flex items-center justify-center border-none cursor-pointer disabled:opacity-30"
        >
          <Plus size={16} />
        </button>
      </div>
    </div>
  )
}
