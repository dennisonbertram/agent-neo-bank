import { cn } from '../../lib/cn'

interface ToggleProps {
  checked: boolean
  onChange: (checked: boolean) => void
  className?: string
}

export function Toggle({ checked, onChange, className }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={cn(
        'relative w-[50px] h-[30px] rounded-[30px] transition-colors duration-200 cursor-pointer border-none',
        checked ? 'bg-black' : 'bg-[var(--bg-secondary)]',
        className
      )}
    >
      <span
        className={cn(
          'absolute top-[2px] w-[26px] h-[26px] rounded-full bg-white shadow-[0_2px_4px_rgba(0,0,0,0.1)] transition-all duration-200',
          checked ? 'left-[22px]' : 'left-[2px]'
        )}
      />
    </button>
  )
}
