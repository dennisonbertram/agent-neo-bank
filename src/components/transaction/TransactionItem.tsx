import { cn } from '../../lib/cn'

const tagStyles: Record<string, string> = {
  X402: 'bg-[var(--variant-info-container)] text-[var(--variant-info-text)]',
  PAYMENT: 'bg-[var(--variant-success-container)] text-[var(--variant-success-text)]',
  DEPOSIT: 'bg-[var(--variant-success-container)] text-[var(--variant-success-text)]',
  TRADE: 'bg-[var(--variant-warning-container)] text-[var(--variant-warning-text)]',
  SWAP: 'bg-[var(--variant-warning-container)] text-[var(--variant-warning-text)]',
  GAS: 'bg-[var(--variant-neutral-container)] text-[var(--variant-neutral-text)]',
}

const defaultTagStyle = 'bg-[var(--variant-neutral-container)] text-[var(--variant-neutral-text)]'

interface TransactionItemProps {
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
        'group flex items-center gap-3 bg-transparent border-none cursor-pointer text-left py-3.5 -mx-3 px-3 rounded-[var(--radius-lg)] transition-all duration-200 w-[calc(100%+24px)]',
        'hover:bg-[var(--surface-hover)]',
        !isLast && 'border-b border-[var(--border-subtle)] hover:border-transparent',
        className
      )}
    >
      <div className="flex-1 min-w-0 flex flex-col justify-center gap-1">
        {/* Top row: label */}
        <span className="text-[15px] font-semibold text-[var(--text-primary)] truncate leading-none">
          {label}
        </span>

        {/* Bottom row: tag + description */}
        <div className="flex items-center gap-2">
          {tag && (
            <span className={cn(
              'text-[11px] px-2 py-0.5 rounded-[var(--radius-pill)] shrink-0 font-semibold uppercase tracking-wide leading-tight',
              tagStyles[tag.toUpperCase()] || defaultTagStyle
            )}>
              {tag}
            </span>
          )}
          <span className="text-[13px] text-[var(--text-tertiary)] truncate leading-none">
            {subLabel}
          </span>
        </div>
      </div>

      {/* Amount */}
      <div className="shrink-0 text-right pl-2">
        <span
          className={cn(
            'text-[16px] font-semibold tabular-nums tracking-tight',
            isPositive ? 'text-[var(--color-positive)]' : 'text-[var(--text-primary)]'
          )}
        >
          {amount}
        </span>
      </div>
    </button>
  )
}
