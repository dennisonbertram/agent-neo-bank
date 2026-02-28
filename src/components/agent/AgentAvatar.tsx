import { cn } from '../../lib/cn'
import type { LucideIcon } from 'lucide-react'

interface AgentAvatarProps {
  icon: LucideIcon
  accentColor: string
  size?: number
  className?: string
}

export function AgentAvatar({ icon: Icon, accentColor, size = 44, className }: AgentAvatarProps) {
  return (
    <div
      className={cn('flex items-center justify-center rounded-[14px]', className)}
      style={{ width: size, height: size, backgroundColor: accentColor }}
    >
      <Icon size={Math.round(size * 0.5)} color="#000000" />
    </div>
  )
}
