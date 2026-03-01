import { useNavigate } from 'react-router-dom'
import type { LucideIcon } from 'lucide-react'
import { cn } from '../../lib/cn'
import { AgentAvatar } from './AgentAvatar'
import { StatusPill } from '../ui/StatusPill'
import { ProgressBar } from '../ui/ProgressBar'
import type { AgentStatus } from '../../types'

interface AgentCardProps {
  id: string
  name: string
  description: string
  status: AgentStatus
  icon: LucideIcon
  accentColor: string
  dailySpent: string
  dailyCap: string
  className?: string
}

const statusMap: Record<AgentStatus, 'active' | 'pending' | 'paused'> = {
  active: 'active',
  pending: 'pending',
  suspended: 'paused',
  revoked: 'paused',
}

export function AgentCard({
  id,
  name,
  description,
  status,
  icon,
  accentColor,
  dailySpent,
  dailyCap,
  className,
}: AgentCardProps) {
  const navigate = useNavigate()
  const spent = parseFloat(dailySpent)
  const cap = parseFloat(dailyCap)
  const percentage = cap > 0 ? (spent / cap) * 100 : 0
  const isPaused = status === 'suspended' || status === 'revoked'

  return (
    <button
      type="button"
      onClick={() => navigate(`/agents/${id}`)}
      className={cn(
        'bg-[var(--bg-elevated)] rounded-[16px] p-5 flex flex-col gap-4 w-full text-left border border-[var(--border-subtle)] shadow-[var(--shadow-subtle)] cursor-pointer transition-all duration-200 hover:bg-[var(--surface-hover)] active:scale-[0.98]',
        className
      )}
    >
      <div className="flex justify-between items-start">
        <div className="flex items-center gap-3">
          <AgentAvatar icon={icon} accentColor={accentColor} />
          <div>
            <div className="text-[15px] font-semibold text-[var(--text-primary)]">{name}</div>
            <div className="text-[13px] text-[var(--text-secondary)] mt-0.5">{description}</div>
          </div>
        </div>
        <StatusPill status={statusMap[status]} />
      </div>
      <ProgressBar
        spent={dailySpent}
        limit={dailyCap}
        percentage={percentage}
        accentColor={accentColor}
        disabled={isPaused}
      />
    </button>
  )
}
