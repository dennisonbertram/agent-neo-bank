import { cn } from '../../lib/cn'

interface MetaCardProps {
  title: string
  items: { label: string; value: string }[]
  className?: string
}

export function MetaCard({ title, items, className }: MetaCardProps) {
  return (
    <div className={cn('bg-[var(--bg-secondary)] rounded-[20px] p-5', className)}>
      <h3 className="text-[12px] font-medium uppercase tracking-[0.5px] text-[var(--text-secondary)] mb-4">
        {title}
      </h3>
      <div className="flex flex-col gap-3">
        {items.map((item) => (
          <div key={item.label} className="flex justify-between items-center">
            <span className="text-[14px] text-[var(--text-secondary)]">{item.label}</span>
            <span className="text-[14px] font-medium text-[var(--text-primary)]">{item.value}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
