import { useRef, useCallback } from 'react'
import { cn } from '../../lib/cn'

interface OtpInputProps {
  length?: number
  value: string
  onChange: (value: string) => void
  onComplete?: (value: string) => void
  className?: string
}

export function OtpInput({ length = 6, value, onChange, onComplete, className }: OtpInputProps) {
  const inputRefs = useRef<(HTMLInputElement | null)[]>([])
  const digits = Array.from({ length }, (_, i) => value[i] || '')

  const firstEmptyIndex = digits.findIndex((d) => !d)
  const nextIndex = firstEmptyIndex === -1 ? length - 1 : firstEmptyIndex

  const handleInput = useCallback(
    (index: number, char: string) => {
      if (!/^\d?$/.test(char)) return
      const newDigits = [...digits]
      newDigits[index] = char
      const newValue = newDigits.join('')
      onChange(newValue)

      if (char && index < length - 1) {
        inputRefs.current[index + 1]?.focus()
      }
      const trimmed = newValue.replace(/\s/g, '')
      if (trimmed.length === length && onComplete) {
        onComplete(trimmed)
      }
    },
    [digits, length, onChange, onComplete]
  )

  const handleKeyDown = useCallback(
    (index: number, e: React.KeyboardEvent) => {
      if (e.key === 'Backspace' && !digits[index] && index > 0) {
        inputRefs.current[index - 1]?.focus()
        const newDigits = [...digits]
        newDigits[index - 1] = ''
        onChange(newDigits.join(''))
      }
    },
    [digits, onChange]
  )

  const handleFocus = useCallback(
    (index: number) => {
      // Enforce sequential entry — redirect to first empty slot
      if (index > nextIndex) {
        inputRefs.current[nextIndex]?.focus()
      }
    },
    [nextIndex]
  )

  return (
    <div className={cn('flex gap-3 justify-center', className)}>
      {digits.map((digit, i) => (
        <input
          key={i}
          ref={(el) => { inputRefs.current[i] = el }}
          type="text"
          inputMode="numeric"
          maxLength={1}
          value={digit}
          onChange={(e) => handleInput(i, e.target.value)}
          onKeyDown={(e) => handleKeyDown(i, e)}
          onFocus={() => handleFocus(i)}
          className="w-[48px] h-[56px] bg-[var(--bg-secondary)] rounded-[12px] text-center text-[24px] font-semibold text-[var(--text-primary)] border-none outline-none focus:ring-2 focus:ring-black/10"
          autoFocus={i === 0}
        />
      ))}
    </div>
  )
}
