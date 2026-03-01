import type { LucideIcon } from 'lucide-react'
import { cn } from '../../lib/cn'

interface AgentPillRowProps {
  icon: LucideIcon
  label: string
  value: string
  subValue?: string
  accentColor: string
  onClick?: () => void
  className?: string
}

export function AgentPillRow({
  icon: Icon,
  label,
  value,
  subValue,
  accentColor,
  onClick,
  className,
}: AgentPillRowProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'flex items-center h-[48px] rounded-[999px] pr-4 min-w-[140px] border border-[var(--border-subtle)] cursor-pointer text-left transition-all duration-200 hover:brightness-110',
        className
      )}
      style={{ backgroundColor: `${accentColor}20` }}
    >
      <div
        className="w-[32px] h-[32px] rounded-full flex items-center justify-center ml-2"
        style={{ backgroundColor: accentColor }}
      >
        <Icon size={16} color="#fff" />
      </div>
      <span className="text-[14px] font-semibold text-[var(--text-primary)] ml-2">{label}</span>
      <div className="ml-auto flex flex-col items-end">
        <span className="text-[15px] font-semibold text-[var(--text-primary)] tabular-nums">{value}</span>
        {subValue && (
          <span className="text-[12px] text-[var(--text-tertiary)]">{subValue}</span>
        )}
      </div>
    </button>
  )
}
