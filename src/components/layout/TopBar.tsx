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
      <div className="flex items-center gap-3">
        <div className="w-[36px] h-[36px] rounded-full bg-[var(--brand-container)] flex items-center justify-center text-[var(--brand-on-container)] text-[14px] font-semibold border border-[var(--brand-main)]/30">
          {initials}
        </div>
        <span className="text-[16px] font-semibold text-[var(--text-primary)]">{walletName}</span>
      </div>
      <button
        type="button"
        onClick={() => navigate('/settings')}
        className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] border border-[var(--border-subtle)] flex items-center justify-center cursor-pointer hover:bg-[var(--surface-hover)] transition-colors"
        aria-label="Settings"
      >
        <Settings size={18} className="text-[var(--text-secondary)]" />
      </button>
    </div>
  )
}
