import { useNavigate } from 'react-router-dom'
import { Home, Layers, Activity, Settings, Plus } from 'lucide-react'
import { cn } from '../../lib/cn'

interface BottomNavProps {
  activeTab: 'home' | 'agents' | 'stats' | 'settings'
  className?: string
}

const tabs = [
  { id: 'home' as const, label: 'Home', icon: Home, path: '/home' },
  { id: 'agents' as const, label: 'Agents', icon: Layers, path: '/agents' },
  { id: 'fab' as const, label: '', icon: Plus, path: '' },
  { id: 'stats' as const, label: 'Stats', icon: Activity, path: '/stats' },
  { id: 'settings' as const, label: 'Settings', icon: Settings, path: '/settings' },
]

export function BottomNav({ activeTab, className }: BottomNavProps) {
  const navigate = useNavigate()

  return (
    <nav
      className={cn(
        'absolute bottom-0 left-0 right-0 h-[84px] bg-white/95 backdrop-blur-[10px] border-t border-black/5 flex justify-around items-center pb-5 z-[100]',
        className
      )}
    >
      {tabs.map((tab) => {
        if (tab.id === 'fab') {
          return (
            <button
              key="fab"
              type="button"
              onClick={() => navigate('/add-funds')}
              className="w-[56px] h-[56px] rounded-full bg-black text-white flex items-center justify-center -mt-[28px] shadow-[0_8px_24px_rgba(0,0,0,0.15)] border-none cursor-pointer"
            >
              <Plus size={24} strokeWidth={3} />
            </button>
          )
        }

        const isActive = activeTab === tab.id
        const Icon = tab.icon

        return (
          <button
            key={tab.id}
            type="button"
            onClick={() => navigate(tab.path)}
            className={cn(
              'flex flex-col items-center gap-1 w-[60px] border-none bg-transparent cursor-pointer',
              isActive ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'
            )}
          >
            <Icon size={24} />
            <span className="text-[10px] font-medium">{tab.label}</span>
          </button>
        )
      })}
    </nav>
  )
}
