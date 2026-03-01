import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { ChevronRight } from 'lucide-react'
import { ScreenHeader } from '../components/layout/ScreenHeader'
import { Toggle } from '../components/ui/Toggle'
import { safeTauriCall, tauriApi, isTauri, placeholderData } from '../lib/tauri'
import { useAuthStore } from '../stores/authStore'
import type { NotificationPreferences } from '../types'

export default function Settings() {
  const navigate = useNavigate()
  const { email, logout } = useAuthStore()
  const user = placeholderData.user

  const [agentRequests, setAgentRequests] = useState(false)
  const [txCompleted, setTxCompleted] = useState(false)
  const [approvalRequired, setApprovalRequired] = useState(false)
  const [dailyLimit, setDailyLimit] = useState(false)
  const [lowBalance, setLowBalance] = useState(false)
  const [prefsId, setPrefsId] = useState('default')

  // Load notification preferences from backend
  useEffect(() => {
    const loadPrefs = async () => {
      const defaults = placeholderData.notifications.defaults
      const fallback: NotificationPreferences = {
        id: 'default',
        enabled: defaults.enabled,
        on_all_tx: defaults.on_all_tx,
        on_large_tx: defaults.on_large_tx,
        large_tx_threshold: defaults.large_tx_threshold,
        on_errors: defaults.on_errors,
        on_limit_requests: defaults.on_limit_requests,
        on_agent_registration: defaults.on_agent_registration,
      }
      const prefs = await safeTauriCall(() => tauriApi.notifications.getPreferences(), fallback)
      setPrefsId(prefs.id)
      setAgentRequests(prefs.on_agent_registration)
      setTxCompleted(prefs.on_all_tx)
      setApprovalRequired(prefs.on_limit_requests)
      setLowBalance(prefs.on_errors)
      setDailyLimit(prefs.on_large_tx)
    }
    loadPrefs()
  }, [])

  // Save preferences whenever a toggle changes
  const savePrefs = useCallback(
    async (updated: Partial<NotificationPreferences>) => {
      if (!isTauri()) return
      const prefs: NotificationPreferences = {
        id: prefsId,
        enabled: true,
        on_all_tx: txCompleted,
        on_large_tx: dailyLimit,
        large_tx_threshold: '10.00',
        on_errors: lowBalance,
        on_limit_requests: approvalRequired,
        on_agent_registration: agentRequests,
        ...updated,
      }
      try {
        await tauriApi.notifications.updatePreferences(prefs)
      } catch {
        // Silently fail — toggle state remains optimistic
      }
    },
    [prefsId, txCompleted, dailyLimit, lowBalance, approvalRequired, agentRequests],
  )

  const handleToggle = (
    setter: (v: boolean) => void,
    field: keyof NotificationPreferences,
  ) => (value: boolean) => {
    setter(value)
    savePrefs({ [field]: value })
  }

  const handleLogout = async () => {
    if (window.confirm('Are you sure you want to reset your Coinbase connection?')) {
      if (isTauri()) {
        try {
          await tauriApi.auth.logout()
        } catch {
          // Continue with local logout even if backend fails
        }
      }
      logout()
      navigate('/onboarding', { replace: true })
    }
  }

  return (
    <div className="flex flex-col h-full relative">
      <ScreenHeader title="Settings" />
      <div className="flex-1 overflow-y-auto px-6 pb-6 scrollbar-hide">
        {/* Profile header */}
        <div className="flex items-center gap-4 py-6 border-b border-[var(--surface-hover)] mb-6">
          <div className="w-[64px] h-[64px] rounded-full bg-[var(--accent-terracotta)] flex items-center justify-center text-white text-[24px] font-semibold">
            {user.initials}
          </div>
          <div>
            <p className="text-subtitle mb-0.5">{user.name}</p>
            <p className="text-[14px] text-[var(--text-secondary)]">{email || user.email}</p>
          </div>
        </div>

        {/* Notifications section */}
        <div className="mb-8">
          <span className="text-caption block mb-3">Notifications</span>

          <SettingsRow label="Agent Requests" description="New agent registration alerts" checked={agentRequests} onChange={handleToggle(setAgentRequests, 'on_agent_registration')} />
          <SettingsRow label="Transaction Completed" description="Confirmation when transactions settle" checked={txCompleted} onChange={handleToggle(setTxCompleted, 'on_all_tx')} />
          <SettingsRow label="Approval Required" description="When agents need spending approval" checked={approvalRequired} onChange={handleToggle(setApprovalRequired, 'on_limit_requests')} />
          <SettingsRow label="Daily Limit Reached" description="Alert when daily budget is exhausted" checked={dailyLimit} onChange={handleToggle(setDailyLimit, 'on_large_tx')} />
          <SettingsRow label="Low Balance" description="Warning when wallet balance is low" checked={lowBalance} onChange={handleToggle(setLowBalance, 'on_errors')} />
        </div>

        {/* Account & Security */}
        <div className="mb-8">
          <span className="text-caption block mb-3">Account & Security</span>

          <button
            type="button"
            onClick={handleLogout}
            className="flex justify-between items-center w-full py-4 border-b border-[var(--surface-hover)] bg-transparent border-x-0 border-t-0 cursor-pointer text-left"
          >
            <div>
              <p className="text-[15px] font-semibold text-[#E5484D]">Reset Coinbase Connection</p>
              <p className="text-[13px] text-[var(--text-secondary)] mt-0.5">Disconnect and re-authenticate your wallet</p>
            </div>
            <ChevronRight size={20} className="text-[var(--text-tertiary)]" />
          </button>

          <button
            type="button"
            className="flex justify-between items-center w-full py-4 border-b border-[var(--surface-hover)] bg-transparent border-x-0 border-t-0 cursor-pointer text-left"
          >
            <div>
              <p className="text-[15px] font-semibold text-[var(--text-primary)]">Export Wallet History</p>
              <p className="text-[13px] text-[var(--text-secondary)] mt-0.5">Download CSV of all agent activity</p>
            </div>
            <ChevronRight size={20} className="text-[var(--text-tertiary)]" />
          </button>
        </div>

        {/* Version */}
        <p className="text-center text-[12px] text-[var(--text-tertiary)] mt-5">
          v{placeholderData.app.version} (Base Mainnet)
        </p>
      </div>

    </div>
  )
}

function SettingsRow({
  label,
  description,
  checked,
  onChange,
}: {
  label: string
  description: string
  checked: boolean
  onChange: (v: boolean) => void
}) {
  return (
    <div className="flex justify-between items-center py-4 border-b border-[var(--surface-hover)]">
      <div>
        <p className="text-[15px] font-semibold text-[var(--text-primary)]">{label}</p>
        <p className="text-[13px] text-[var(--text-secondary)] mt-0.5">{description}</p>
      </div>
      <Toggle checked={checked} onChange={onChange} />
    </div>
  )
}
