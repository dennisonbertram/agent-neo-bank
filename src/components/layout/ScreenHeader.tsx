import { useNavigate } from 'react-router-dom'
import { ChevronLeft } from 'lucide-react'
import { cn } from '../../lib/cn'

interface ScreenHeaderProps {
  title?: string
  onBack?: () => void
  rightElement?: React.ReactNode
  className?: string
}

export function ScreenHeader({ title, onBack, rightElement, className }: ScreenHeaderProps) {
  const navigate = useNavigate()

  const handleBack = onBack || (() => navigate(-1))

  return (
    <header
      className={cn(
        'sticky top-0 z-10 pt-[16px] pb-4 px-6 flex items-center justify-between bg-white/90 backdrop-blur-[10px]',
        className
      )}
    >
      <button
        type="button"
        onClick={handleBack}
        className="w-[40px] h-[40px] rounded-full border border-[var(--surface-hover)] bg-transparent flex items-center justify-center cursor-pointer"
      >
        <ChevronLeft size={20} />
      </button>
      {title && (
        <h1 className="text-[17px] font-semibold text-[var(--text-primary)]">{title}</h1>
      )}
      {rightElement || <div className="w-[40px]" />}
    </header>
  )
}
