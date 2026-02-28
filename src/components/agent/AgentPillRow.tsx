import type { LucideIcon } from 'lucide-react'
import { cn } from '../../lib/cn'

interface AgentPillRowProps {
  icon: LucideIcon
  label: string
  value: string
  subValue?: string
  accentColor: string
  className?: string
}

export function AgentPillRow({
  icon: Icon,
  label,
  value,
  subValue,
  accentColor,
  className,
}: AgentPillRowProps) {
  return (
    <div
      className={cn(
        'flex items-center h-[48px] rounded-[999px] pr-4 min-w-[140px]',
        className
      )}
      style={{ backgroundColor: `${accentColor}15` }}
    >
      <div
        className="w-[32px] h-[32px] rounded-full flex items-center justify-center ml-2"
        style={{ backgroundColor: accentColor }}
      >
        <Icon size={16} color="#000" />
      </div>
      <span className="text-[14px] font-semibold text-[var(--text-primary)] ml-2">{label}</span>
      <div className="ml-auto flex flex-col items-end">
        <span className="text-[15px] font-semibold text-[var(--text-primary)]">{value}</span>
        {subValue && (
          <span className="text-[12px] text-[var(--text-secondary)]">{subValue}</span>
        )}
      </div>
    </div>
  )
}
