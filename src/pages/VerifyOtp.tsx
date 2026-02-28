import { useState, useEffect, useCallback } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { ChevronLeft } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { OtpInput } from '../components/ui/OtpInput'
import { useAuthStore } from '../stores/authStore'
import { tauriApi, isTauri } from '../lib/tauri'

export default function VerifyOtp() {
  const [otp, setOtp] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [countdown, setCountdown] = useState(42)
  const navigate = useNavigate()
  const location = useLocation()
  const { setAuthenticated } = useAuthStore()

  const email = (location.state as { email?: string })?.email || useAuthStore.getState().email || ''

  // Countdown timer
  useEffect(() => {
    if (countdown <= 0) return
    const timer = setInterval(() => {
      setCountdown((prev) => prev - 1)
    }, 1000)
    return () => clearInterval(timer)
  }, [countdown])

  const handleVerify = useCallback(async () => {
    if (otp.length !== 6) return
    setIsLoading(true)
    setError(null)

    try {
      if (isTauri()) {
        const result = await tauriApi.auth.verify(otp)
        if (result.status === 'verified') {
          setAuthenticated(email)
          navigate('/home', { replace: true })
        } else {
          setError('Verification failed. Please try again.')
        }
      } else {
        // Browser mode: skip verification, proceed directly
        setAuthenticated(email)
        navigate('/home', { replace: true })
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed. Please check your code.')
    } finally {
      setIsLoading(false)
    }
  }, [otp, email, setAuthenticated, navigate])

  const handleResend = async () => {
    if (countdown > 0) return
    try {
      if (isTauri()) {
        await tauriApi.auth.login(email)
      }
      setCountdown(42)
      setOtp('')
      setError(null)
    } catch {
      setError('Failed to resend code.')
    }
  }

  const formatCountdown = (seconds: number) => {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  return (
    <div className="flex flex-col h-full relative">
      <div className="flex-1 px-6 pt-[16px]">
        {/* Back button */}
        <button
          type="button"
          onClick={() => navigate(-1)}
          className="w-[40px] h-[40px] rounded-full border border-[var(--surface-hover)] bg-transparent flex items-center justify-center cursor-pointer mb-8"
        >
          <ChevronLeft size={20} />
        </button>

        {/* Title */}
        <h1 className="text-[28px] font-bold text-[var(--text-primary)] mb-3">
          Verify it's you
        </h1>
        <p className="text-body mb-10">
          Enter the 6-digit code sent to{' '}
          <span className="font-semibold text-[var(--text-primary)]">{email}</span>
        </p>

        {/* OTP Input */}
        <OtpInput
          value={otp}
          onChange={setOtp}
          onComplete={handleVerify}
          className="mb-8"
        />

        {error && (
          <p className="text-[13px] text-[var(--color-danger)] text-center mb-4">{error}</p>
        )}

        {/* Verify button */}
        <Button
          variant="primary"
          onClick={handleVerify}
          disabled={otp.length !== 6 || isLoading}
        >
          {isLoading ? 'Verifying...' : 'Verify code'}
        </Button>

        {/* Resend row */}
        <div className="flex items-center justify-center gap-2 mt-6">
          <button
            type="button"
            onClick={handleResend}
            disabled={countdown > 0}
            className={`bg-transparent border-none text-[14px] font-medium cursor-pointer ${
              countdown > 0
                ? 'text-[var(--text-tertiary)] cursor-default'
                : 'text-[var(--text-primary)]'
            }`}
          >
            Resend code
          </button>
          {countdown > 0 && (
            <span className="text-[14px] text-[var(--text-secondary)]">
              in {formatCountdown(countdown)}
            </span>
          )}
        </div>
      </div>
    </div>
  )
}
