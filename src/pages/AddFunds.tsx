import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Copy, Grid3X3, Check, CreditCard } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { safeTauriCall, tauriApi, placeholderData } from '../lib/tauri'

export default function AddFunds() {
  const navigate = useNavigate()
  const [copied, setCopied] = useState(false)
  const [address, setAddress] = useState(placeholderData.wallet.address)
  useEffect(() => {
    const loadAddress = async () => {
      const result = await safeTauriCall(
        () => tauriApi.wallet.getAddress(),
        { address: placeholderData.wallet.address },
      )
      setAddress(result.address)
    }
    loadAddress()
  }, [])

  const handleCopy = async () => {
    try {
      // Try Tauri clipboard first, fall back to browser API
      const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
      await writeText(address)
    } catch {
      await navigator.clipboard.writeText(address)
    }
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="flex flex-col h-full relative">
      <div className="flex-1 overflow-y-auto px-6 pt-[80px] pb-[40px]">
        {/* QR Code Placeholder */}
        <div className="w-[200px] h-[200px] mx-auto mb-6 bg-[var(--bg-secondary)] rounded-[20px] border-2 border-dashed border-[var(--surface-hover)] flex items-center justify-center">
          <Grid3X3 size={48} className="text-[var(--text-tertiary)]" />
        </div>

        {/* Warning pill */}
        <div className="flex items-center justify-center gap-2 bg-[var(--accent-yellow-dim)] text-[var(--status-pending-text)] rounded-[8px] px-4 py-2 mb-6 mx-auto w-fit">
          <span className="text-[13px] font-medium">Send only USDC or ETH on Base</span>
        </div>

        {/* Wallet Address */}
        <div className="bg-[var(--bg-secondary)] rounded-[20px] p-4 mb-4">
          <label className="block text-[12px] text-[var(--text-secondary)] mb-2 uppercase tracking-[0.5px]">
            Wallet Address
          </label>
          <div className="flex items-center justify-between">
            <span className="text-mono text-[13px] text-[var(--text-primary)] truncate mr-2">
              {address}
            </span>
            <button
              type="button"
              onClick={handleCopy}
              className="shrink-0 w-[36px] h-[36px] rounded-full bg-[var(--bg-primary)] border border-[var(--surface-hover)] flex items-center justify-center cursor-pointer"
            >
              {copied ? <Check size={16} className="text-[var(--accent-green)]" /> : <Copy size={16} />}
            </button>
          </div>
        </div>

        {/* Buy with Card - disabled */}
        <Button variant="outline" disabled className="opacity-50 mb-4">
          <CreditCard size={18} className="mr-2" />
          Buy with Card (Coming Soon)
        </Button>

        {/* Close button */}
        <Button variant="outline" onClick={() => navigate('/home')}>
          Close
        </Button>
      </div>
    </div>
  )
}
