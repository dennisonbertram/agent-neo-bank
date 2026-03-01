import { cn } from '../../lib/cn'

interface SegmentControlProps {
  options: string[]
  value: string
  onChange: (value: string) => void
  className?: string
}

export function SegmentControl({ options, value, onChange, className }: SegmentControlProps) {
  return (
    <div className={cn('bg-[var(--bg-secondary)] rounded-[999px] p-1 flex', className)}>
      {options.map((option) => (
        <button
          key={option}
          type="button"
          onClick={() => onChange(option)}
          className={cn(
            'flex-1 text-center py-2 text-[14px] font-medium rounded-[999px] border-none cursor-pointer transition-all duration-200',
            value === option
              ? 'bg-[var(--bg-elevated)] text-[var(--text-primary)] shadow-[var(--shadow-subtle)]'
              : 'bg-transparent text-[var(--text-tertiary)] hover:text-[var(--text-secondary)]'
          )}
        >
          {option}
        </button>
      ))}
    </div>
  )
}
