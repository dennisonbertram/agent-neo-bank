import { cn } from '../../lib/cn'

interface ProgressBarProps {
  spent: string
  limit: string
  percentage: number
  accentColor?: string
  disabled?: boolean
  className?: string
}

export function ProgressBar({
  spent,
  limit,
  percentage,
  accentColor = 'var(--accent-green)',
  disabled = false,
  className,
}: ProgressBarProps) {
  const clampedPercent = Math.min(Math.max(percentage, 0), 100)

  return (
    <div className={cn('w-full', className)}>
      <div
        className={cn(
          'h-[6px] w-full rounded-[3px] bg-black/5 overflow-hidden',
          disabled && 'opacity-30'
        )}
      >
        <div
          className="h-full rounded-[3px] transition-all duration-300"
          style={{
            width: `${clampedPercent}%`,
            backgroundColor: accentColor,
          }}
        />
      </div>
      <div className="flex justify-between mt-2 text-[13px] font-medium">
        <span className={cn(disabled ? 'text-[var(--text-secondary)]' : 'text-[var(--text-primary)]')}>
          {disabled ? 'Spending disabled' : `$${spent} spent`}
        </span>
        <span className="text-[var(--text-secondary)]">${limit} limit</span>
      </div>
    </div>
  )
}
