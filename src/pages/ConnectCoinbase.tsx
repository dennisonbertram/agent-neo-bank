import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ChevronLeft } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { InputGroup } from '../components/ui/InputGroup'
import { useAuthStore } from '../stores/authStore'
import { tauriApi, isTauri } from '../lib/tauri'

export default function ConnectCoinbase() {
  const [email, setEmail] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const navigate = useNavigate()
  const { setFlowId, setAuthenticated } = useAuthStore()

  const handleSendCode = async () => {
    if (!email.trim()) return
    setIsLoading(true)
    setError(null)

    try {
      if (isTauri()) {
        const result = await tauriApi.auth.login(email.trim())
        if (result.status === 'already_authenticated') {
          setAuthenticated(email.trim())
          navigate('/home')
          return
        }
        if (result.flow_id) {
          setFlowId(result.flow_id)
        }
      }
      navigate('/setup/verify', { state: { email: email.trim() } })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send code. Please try again.')
    } finally {
      setIsLoading(false)
    }
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

        {/* Headline */}
        <h1 className="text-[34px] font-semibold leading-tight tracking-[-1px] text-[var(--text-primary)] whitespace-pre-line mb-4">
          {"Connect your\nCoinbase account"}
        </h1>

        <p className="text-body mb-8">
          Link your Coinbase wallet to fund agent operations and track spending in real time.
        </p>

        {/* Email input */}
        <InputGroup label="EMAIL ADDRESS">
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="name@email.com"
            autoFocus
            onKeyDown={(e) => e.key === 'Enter' && handleSendCode()}
            className="w-full bg-transparent border-none outline-none text-[16px] text-[var(--text-primary)] placeholder:text-[var(--text-tertiary)]"
          />
        </InputGroup>

        {error && (
          <p className="text-[13px] text-[var(--color-danger)] mt-3">{error}</p>
        )}

        {/* Send code button */}
        <div className="mt-6">
          <Button
            variant="primary"
            onClick={handleSendCode}
            disabled={!email.trim() || isLoading}
          >
            {isLoading ? 'Sending...' : 'Send code'}
          </Button>
        </div>

        <p className="text-[13px] text-[var(--text-secondary)] mt-4 text-center">
          A secure login link will be sent to your inbox.
        </p>
      </div>

      {/* Trust badge */}
      <div className="absolute bottom-[40px] left-0 right-0 flex items-center justify-center gap-2">
        <div className="w-[20px] h-[20px] rounded-full bg-[#0052FF] flex items-center justify-center">
          <span className="text-white text-[11px] font-bold">C</span>
        </div>
        <span className="text-[13px] text-[var(--text-secondary)]">Secured by Coinbase Cloud</span>
      </div>
    </div>
  )
}
