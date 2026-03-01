import { useNavigate, useLocation } from 'react-router-dom'
import { Home, Layers, Activity, Plus } from 'lucide-react'
import { cn } from '../../lib/cn'

interface BottomNavProps {
  className?: string
}

const tabs = [
  { id: 'home', label: 'Home', icon: Home, path: '/home' },
  { id: 'agents', label: 'Agents', icon: Layers, path: '/agents' },
  { id: 'fab', label: '', icon: Plus, path: '/add-funds' },
  { id: 'stats', label: 'Stats', icon: Activity, path: '/stats' },
]

export function BottomNav({ className }: BottomNavProps) {
  const navigate = useNavigate()
  const location = useLocation()

  // Only render on root screens
  const isRootScreen = ['/home', '/agents', '/stats'].includes(location.pathname)
  if (!isRootScreen) return null

  return (
    <nav
      className={cn(
        'absolute bottom-0 left-0 right-0 h-[84px] z-50 pb-5',
        'bg-[var(--bg-elevated)]/90 backdrop-blur-md border-t border-[var(--border-subtle)] flex justify-around items-center',
        className
      )}
    >
      {tabs.map((tab) => {
        if (tab.id === 'fab') {
          return (
            <button
              key="fab"
              type="button"
              onClick={() => navigate(tab.path)}
              className="w-[52px] h-[52px] rounded-full bg-[var(--brand-main)] text-[var(--text-primary)] flex items-center justify-center -mt-[26px] shadow-[var(--shadow-fab)] border border-[var(--brand-container)] cursor-pointer transition-transform hover:scale-105 active:scale-95"
            >
              <Plus size={24} strokeWidth={3} />
            </button>
          )
        }

        const isActive = location.pathname.startsWith(tab.path)
        const Icon = tab.icon

        return (
          <button
            key={tab.id}
            type="button"
            onClick={() => navigate(tab.path)}
            className={cn(
              'flex flex-col items-center gap-1 w-[64px] border-none bg-transparent cursor-pointer transition-colors',
              isActive ? 'text-[var(--brand-on-container)]' : 'text-[var(--text-tertiary)] hover:text-[var(--text-secondary)]'
            )}
          >
            <Icon size={22} strokeWidth={isActive ? 2.5 : 2} />
            <span className="text-[10px] font-medium">{tab.label}</span>
          </button>
        )
      })}
    </nav>
  )
}
