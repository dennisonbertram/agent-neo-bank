import { useNavigate } from 'react-router-dom'
import { ChevronLeft } from 'lucide-react'
import { cn } from '../../lib/cn'

interface BreadcrumbItem {
  label: string
  path?: string
}

interface ScreenHeaderProps {
  breadcrumbs: BreadcrumbItem[]
  rightElement?: React.ReactNode
  onBack?: () => void
  className?: string
}

export function ScreenHeader({ breadcrumbs, rightElement, onBack, className }: ScreenHeaderProps) {
  const navigate = useNavigate()

  const handleBack = () => {
    if (onBack) { onBack(); return }
    // Navigate to the second-to-last breadcrumb's path, or go back
    const parent = breadcrumbs.length >= 2 ? breadcrumbs[breadcrumbs.length - 2] : null
    if (parent?.path) {
      navigate(parent.path)
    } else {
      navigate(-1)
    }
  }

  return (
    <header
      className={cn(
        'sticky top-0 z-10 pt-[16px] pb-4 px-6 flex items-center justify-between bg-white/90 backdrop-blur-[10px]',
        className
      )}
    >
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={handleBack}
          className="w-[40px] h-[40px] rounded-full border border-[var(--surface-hover)] bg-transparent flex items-center justify-center cursor-pointer flex-shrink-0"
        >
          <ChevronLeft size={20} />
        </button>
        <nav className="flex items-center gap-1 text-[14px] font-medium min-w-0">
          {breadcrumbs.map((crumb, i) => {
            const isLast = i === breadcrumbs.length - 1
            return (
              <span key={`${crumb.label}-${i}`} className="flex items-center gap-1 min-w-0">
                {i > 0 && (
                  <span className="text-[var(--text-tertiary)] flex-shrink-0">/</span>
                )}
                {isLast || !crumb.path ? (
                  <span
                    className={cn(
                      'truncate',
                      isLast
                        ? 'text-[var(--text-primary)]'
                        : 'text-[var(--text-secondary)]'
                    )}
                  >
                    {crumb.label}
                  </span>
                ) : (
                  <button
                    type="button"
                    onClick={() => navigate(crumb.path!)}
                    className="text-[var(--text-secondary)] hover:underline bg-transparent border-none cursor-pointer p-0 text-[14px] font-medium truncate"
                  >
                    {crumb.label}
                  </button>
                )}
              </span>
            )
          })}
        </nav>
      </div>
      {rightElement || <div className="w-[40px] flex-shrink-0" />}
    </header>
  )
}
