import { Settings } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { cn } from '../../lib/cn'

interface TopBarProps {
  walletName?: string
  initials?: string
  className?: string
}

export function TopBar({ walletName = 'Tally Wallet', initials = 'DB', className }: TopBarProps) {
  const navigate = useNavigate()

  return (
    <div
      className={cn(
        'flex items-center justify-between px-6 pt-6 pb-2',
        className
      )}
    >
      {/* Interactive Profile Pill — single clickable element */}
      <button
        type="button"
        onClick={() => navigate('/settings')}
        className="group flex items-center gap-2.5 p-1.5 pr-4 rounded-[var(--radius-pill)] hover:bg-[var(--surface-hover)] border border-transparent hover:border-[var(--border-subtle)] transition-all cursor-pointer bg-transparent"
        aria-label="Open Settings"
      >
        <div className="w-[32px] h-[32px] rounded-full bg-[var(--brand-container)] flex items-center justify-center text-[var(--brand-on-container)] text-[13px] font-semibold">
          {initials}
        </div>
        <span className="text-[15px] font-semibold text-[var(--text-primary)] transition-colors">
          {walletName}
        </span>
        <Settings size={16} className="text-[var(--text-tertiary)] group-hover:text-[var(--text-secondary)] transition-colors ml-0.5" />
      </button>
    </div>
  )
}
