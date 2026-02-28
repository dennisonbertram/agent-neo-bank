import { Bell } from 'lucide-react'
import { cn } from '../../lib/cn'

interface TopBarProps {
  walletName?: string
  initials?: string
  className?: string
}

export function TopBar({ walletName = 'Agent Wallet', initials = 'DB', className }: TopBarProps) {
  return (
    <div
      className={cn(
        'flex items-center justify-between px-6',
        className
      )}
    >
      <div className="flex items-center gap-3">
        <div className="w-[40px] h-[40px] rounded-full bg-[var(--accent-terracotta)] flex items-center justify-center text-white text-[14px] font-semibold">
          {initials}
        </div>
        <span className="text-[17px] font-semibold text-[var(--text-primary)]">{walletName}</span>
      </div>
      <button
        type="button"
        className="w-[40px] h-[40px] rounded-full border border-[var(--surface-hover)] bg-transparent flex items-center justify-center cursor-pointer"
      >
        <Bell size={20} color="var(--text-primary)" />
      </button>
    </div>
  )
}
