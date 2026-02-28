import { cn } from '../../lib/cn'

interface AppLogoProps {
  size?: number
  className?: string
}

export function AppLogo({ size = 60, className }: AppLogoProps) {
  const iconSize = Math.round(size * 0.53)

  return (
    <div
      className={cn(
        'flex items-center justify-center bg-[var(--accent-green)] rounded-[20px]',
        className
      )}
      style={{ width: size, height: size }}
    >
      <svg
        width={iconSize}
        height={iconSize}
        viewBox="0 0 32 32"
        fill="none"
        stroke="black"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        {/* Hexagon */}
        <path d="M16 2 L28 9 L28 23 L16 30 L4 23 L4 9 Z" />
        {/* Circle in center */}
        <circle cx="16" cy="16" r="6" />
      </svg>
    </div>
  )
}
