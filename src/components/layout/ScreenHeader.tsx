import { useNavigate } from 'react-router-dom'
import { ChevronLeft } from 'lucide-react'
import { cn } from '../../lib/cn'

interface ScreenHeaderProps {
  title: string
  showBack?: boolean
  rightElement?: React.ReactNode
  onBack?: () => void
  className?: string
}

export function ScreenHeader({
  title,
  showBack = true,
  rightElement,
  onBack,
  className
}: ScreenHeaderProps) {
  const navigate = useNavigate()

  const handleBack = () => {
    if (onBack) { onBack(); return }
    navigate(-1)
  }

  return (
    <header
      className={cn(
        'sticky top-0 z-50 h-[56px] px-6 flex items-center justify-between',
        'bg-[var(--bg-primary)]/80 backdrop-blur-md border-b border-[var(--border-subtle)]/50',
        className
      )}
    >
      {/* Left Action - Fixed Width to maintain center balance */}
      <div className="flex-1 flex items-center justify-start">
        {showBack && (
          <button
            type="button"
            onClick={handleBack}
            className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] border border-[var(--border-subtle)] flex items-center justify-center cursor-pointer hover:bg-[var(--surface-hover)] transition-colors"
            aria-label="Go back"
          >
            <ChevronLeft size={20} className="text-[var(--text-secondary)]" />
          </button>
        )}
      </div>

      {/* Centered Title */}
      <div className="flex-[2] flex justify-center text-center">
        <h2 className="text-[16px] font-semibold text-[var(--text-primary)] truncate max-w-[200px]">
          {title}
        </h2>
      </div>

      {/* Right Action - Fixed Width */}
      <div className="flex-1 flex items-center justify-end">
        {rightElement}
      </div>
    </header>
  )
}
