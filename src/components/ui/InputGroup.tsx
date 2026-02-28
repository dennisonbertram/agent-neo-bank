import { cn } from '../../lib/cn'

interface InputGroupProps {
  label: string
  children: React.ReactNode
  className?: string
}

export function InputGroup({ label, children, className }: InputGroupProps) {
  return (
    <div className={cn('bg-[var(--bg-secondary)] rounded-[20px] p-4', className)}>
      <label className="block text-[12px] text-[var(--text-secondary)] mb-2">
        {label}
      </label>
      {children}
    </div>
  )
}
