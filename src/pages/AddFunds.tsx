import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { QRCodeSVG } from 'qrcode.react'
import { Copy, Check } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { ScreenHeader } from '../components/layout/ScreenHeader'
import { useWalletStore } from '../stores/walletStore'

export default function AddFunds() {
  const navigate = useNavigate()
  const [copied, setCopied] = useState(false)
  const { address } = useWalletStore()

  const handleCopy = async () => {
    if (!address) return
    try {
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
      <ScreenHeader title="Add Funds" />
      <div className="flex-1 overflow-y-auto px-6 pt-4 pb-[40px]">
        {/* QR Code */}
        <div className="w-[220px] h-[220px] mx-auto mb-6 bg-white rounded-[20px] flex items-center justify-center p-3">
          {address ? (
            <QRCodeSVG
              value={address}
              size={190}
              level="M"
              bgColor="#FFFFFF"
              fgColor="#111111"
            />
          ) : (
            <div className="w-6 h-6 border-2 border-black/20 border-t-black rounded-full animate-spin" />
          )}
        </div>

        {/* Warning pill */}
        <div className="flex items-center justify-center gap-2 bg-[var(--accent-yellow-dim)] text-[var(--status-pending-text)] rounded-[8px] px-4 py-2 mb-6 mx-auto w-fit">
          <span className="text-[13px] font-medium">Base network only — USDC, ETH, or any Base token</span>
        </div>

        {/* Info text */}
        <p className="text-[13px] text-[var(--text-secondary)] text-center mb-6 leading-relaxed max-w-[300px] mx-auto">
          Send funds to this address on the <strong className="text-[var(--text-primary)]">Base</strong> network. Tokens sent on other networks will be lost.
        </p>

        {/* Wallet Address */}
        <button
          type="button"
          onClick={handleCopy}
          className="w-full bg-[var(--bg-secondary)] rounded-[20px] p-4 mb-4 cursor-pointer border-0 text-center hover:bg-[var(--surface-hover)] transition-colors"
        >
          <label className="block text-[12px] text-[var(--text-secondary)] mb-2 uppercase tracking-[0.5px] pointer-events-none">
            Base Wallet Address
          </label>
          <p className="text-mono text-[13px] text-[var(--text-primary)] break-all mb-3">
            {address ?? '...'}
          </p>
          <span className="inline-flex items-center gap-1.5 text-[12px] text-[var(--text-secondary)]">
            {copied ? (
              <>
                <Check size={14} className="text-[var(--accent-green)]" />
                <span className="text-[var(--accent-green)]">Copied!</span>
              </>
            ) : (
              <>
                <Copy size={14} />
                <span>Tap to copy</span>
              </>
            )}
          </span>
        </button>

        {/* Close button */}
        <Button variant="outline" onClick={() => navigate('/home')}>
          Close
        </Button>
      </div>
    </div>
  )
}
