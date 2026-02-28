import { cn } from '../../lib/cn'

type StatusType = 'active' | 'pending' | 'paused' | 'running'

const statusStyles: Record<StatusType, string> = {
  active: 'bg-[var(--accent-green-dim)] text-[var(--status-active-text)]',
  running: 'bg-[var(--accent-green-dim)] text-[var(--status-active-text)]',
  pending: 'bg-[var(--accent-yellow-dim)] text-[var(--status-pending-text)]',
  paused: 'bg-[var(--accent-terracotta-dim)] text-[var(--status-paused-text)]',
}

const statusLabels: Record<StatusType, string> = {
  active: 'Active',
  running: 'Running',
  pending: 'Pending',
  paused: 'Paused',
}

interface StatusPillProps {
  status: StatusType
  className?: string
}

export function StatusPill({ status, className }: StatusPillProps) {
  return (
    <span
      className={cn(
        'inline-block px-[10px] py-1 rounded-[8px] text-[11px] font-bold uppercase',
        statusStyles[status],
        className
      )}
    >
      {statusLabels[status]}
    </span>
  )
}
