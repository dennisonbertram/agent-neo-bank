import { cn } from '../../lib/cn'
import type { LucideIcon } from 'lucide-react'

interface TransactionItemProps {
  icon: LucideIcon
  iconBgColor?: string
  label: string
  subLabel: string
  amount: string
  tag?: string
  isPositive?: boolean
  isLast?: boolean
  onClick?: () => void
  className?: string
}

export function TransactionItem({
  icon: Icon,
  iconBgColor = 'var(--bg-secondary)',
  label,
  subLabel,
  amount,
  tag,
  isPositive = false,
  isLast = false,
  onClick,
  className,
}: TransactionItemProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'flex items-center gap-3 py-3 w-full bg-transparent border-none cursor-pointer text-left',
        !isLast && 'border-b border-[var(--surface-hover)]',
        className
      )}
    >
      <div
        className="w-[40px] h-[40px] rounded-[12px] flex items-center justify-center shrink-0"
        style={{ backgroundColor: iconBgColor }}
      >
        <Icon size={20} />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-[15px] font-medium text-[var(--text-primary)] truncate">{label}</span>
          {tag && (
            <span className="text-mono text-[10px] bg-[var(--bg-secondary)] px-1.5 py-0.5 rounded-[6px] text-[var(--text-secondary)] shrink-0">
              {tag}
            </span>
          )}
        </div>
        <span className="text-[11px] font-medium text-[var(--text-secondary)]">{subLabel}</span>
      </div>
      <span
        className={cn(
          'text-[17px] font-medium shrink-0',
          isPositive ? 'text-[var(--color-positive)]' : 'text-[var(--text-primary)]'
        )}
      >
        {amount}
      </span>
    </button>
  )
}
