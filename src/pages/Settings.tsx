import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { Download, LogOut, LifeBuoy, Wallet, Plug, Loader2 } from 'lucide-react'
import { ScreenHeader } from '../components/layout/ScreenHeader'
import { Toggle } from '../components/ui/Toggle'
import { safeTauriCall, tauriApi, isTauri, placeholderData } from '../lib/tauri'
import { useAuthStore } from '../stores/authStore'
import { useWalletStore } from '../stores/walletStore'
import { useProvisioningStore } from '../stores/provisioningStore'
import type { NotificationPreferences, ToolId, ToolProvisioningState, DetectionResult } from '../types'

const TOOL_NAMES: Record<ToolId, string> = {
  claude_code: 'Claude Code',
  cursor: 'Cursor',
  windsurf: 'Windsurf',
  claude_desktop: 'Claude Desktop',
  codex: 'Codex CLI',
  continue_dev: 'Continue',
  cline: 'Cline',
  aider: 'Aider',
  copilot: 'GitHub Copilot',
}

export default function Settings() {
  const navigate = useNavigate()
  const { email, logout } = useAuthStore()
  const { address: walletAddress } = useWalletStore()
  const { detectionResults, state, isDetecting, actionInProgress, actionError, initialize: initProvisioning, provisionTool, unprovisionTool } = useProvisioningStore()
  const user = placeholderData.user

  const [agentRequests, setAgentRequests] = useState(false)
  const [txCompleted, setTxCompleted] = useState(false)
  const [approvalRequired, setApprovalRequired] = useState(false)
  const [dailyLimit, setDailyLimit] = useState(false)
  const [lowBalance, setLowBalance] = useState(false)
  const [prefsId, setPrefsId] = useState('default')

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

  useEffect(() => { initProvisioning() }, [initProvisioning])

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
        // Silently fail — optimistic UI
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
    if (window.confirm('Are you sure you want to disconnect your Coinbase wallet?')) {
      if (isTauri()) {
        try { await tauriApi.auth.logout() } catch { /* continue */ }
      }
      logout()
      navigate('/onboarding', { replace: true })
    }
  }

  const sortedTools = [...detectionResults].sort((a, b) => {
    const stateA = state?.tools[a.tool]
    const stateB = state?.tools[b.tool]
    const scoreA = stateA?.status === 'provisioned' ? 0 : a.detected ? 1 : 2
    const scoreB = stateB?.status === 'provisioned' ? 0 : b.detected ? 1 : 2
    if (scoreA !== scoreB) return scoreA - scoreB
    return a.tool.localeCompare(b.tool)
  })

  const truncatedAddress = walletAddress
    ? `${walletAddress.slice(0, 6)}...${walletAddress.slice(-4)}`
    : '...'

  return (
    <div className="flex flex-col h-full relative">
      <ScreenHeader title="Settings" />
      <div className="flex-1 overflow-y-auto px-6 pb-8 scrollbar-hide">

        {/* Profile Card */}
        <div className="flex flex-col items-center py-6 mb-4">
          <div className="w-[72px] h-[72px] rounded-full bg-[var(--brand-container)] flex items-center justify-center text-[var(--brand-on-container)] text-[26px] font-semibold mb-3 border-2 border-[var(--bg-primary)]">
            {user.initials}
          </div>
          <h2 className="text-[20px] font-semibold text-[var(--text-primary)] mb-1">{user.name}</h2>
          <p className="text-[13px] text-[var(--text-tertiary)] bg-[var(--bg-secondary)] px-3 py-1 rounded-full border border-[var(--border-subtle)]">
            {email || user.email}
          </p>
        </div>

        {/* Network & Wallet */}
        <div className="mb-6">
          <span className="text-[12px] font-semibold tracking-wider uppercase text-[var(--text-tertiary)] block mb-2 px-1">Network</span>
          <div className="bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] overflow-hidden">
            <div className="flex items-center gap-3 px-4 py-3.5">
              <Wallet size={18} className="text-[var(--text-secondary)]" />
              <div>
                <p className="text-[15px] font-medium text-[var(--text-primary)]">Base Mainnet</p>
                <p className="text-[13px] text-[var(--text-tertiary)] mt-0.5 font-mono">{truncatedAddress}</p>
              </div>
            </div>
          </div>
        </div>

        {/* Connected Tools */}
        <div className="mb-6">
          <span className="text-[12px] font-semibold tracking-wider uppercase text-[var(--text-tertiary)] block mb-2 px-1">Connected Tools</span>
          <div className="bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] overflow-hidden">
            {isDetecting && detectionResults.length === 0 ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 size={20} className="animate-spin text-[var(--text-tertiary)]" />
              </div>
            ) : detectionResults.length === 0 ? (
              <p className="text-[13px] text-[var(--text-tertiary)] text-center py-8">No AI coding tools detected</p>
            ) : (
              sortedTools.map((result, i) => (
                <ToolRow
                  key={result.tool}
                  result={result}
                  toolState={state?.tools[result.tool]}
                  actionInProgress={actionInProgress[result.tool] ?? false}
                  actionError={actionError[result.tool] ?? null}
                  onProvision={() => provisionTool(result.tool)}
                  onUnprovision={() => unprovisionTool(result.tool)}
                  isLast={i === sortedTools.length - 1}
                />
              ))
            )}
          </div>
        </div>

        {/* Notifications Panel */}
        <div className="mb-6">
          <span className="text-[12px] font-semibold tracking-wider uppercase text-[var(--text-tertiary)] block mb-2 px-1">Notifications</span>
          <div className="bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] overflow-hidden flex flex-col">
            <SettingsRow label="Agent Requests" description="New agent registration alerts" checked={agentRequests} onChange={handleToggle(setAgentRequests, 'on_agent_registration')} />
            <SettingsRow label="Transaction Completed" description="Confirmation when txs settle" checked={txCompleted} onChange={handleToggle(setTxCompleted, 'on_all_tx')} />
            <SettingsRow label="Approval Required" description="When agents need spending approval" checked={approvalRequired} onChange={handleToggle(setApprovalRequired, 'on_limit_requests')} />
            <SettingsRow label="Daily Limit Reached" description="Alert when budget is exhausted" checked={dailyLimit} onChange={handleToggle(setDailyLimit, 'on_large_tx')} />
            <SettingsRow label="Low Balance" description="Warning when wallet balance is low" checked={lowBalance} onChange={handleToggle(setLowBalance, 'on_errors')} isLast />
          </div>
        </div>

        {/* Account & Support Panel */}
        <div className="mb-8">
          <span className="text-[12px] font-semibold tracking-wider uppercase text-[var(--text-tertiary)] block mb-2 px-1">Account & Support</span>
          <div className="bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] overflow-hidden flex flex-col">
            <ActionRow icon={Download} label="Export Wallet History" description="Download CSV of all agent activity" />
            <ActionRow icon={LifeBuoy} label="Help & Support" description="Read docs or contact us" />

            <button
              type="button"
              onClick={handleLogout}
              className="flex items-center gap-3 w-full px-4 py-3.5 bg-transparent border-t border-[var(--border-subtle)] cursor-pointer text-left hover:bg-[var(--variant-danger-container)]/30 transition-colors group border-x-0 border-b-0"
            >
              <LogOut size={18} className="text-[#E5484D] group-hover:scale-105 transition-transform" />
              <div>
                <p className="text-[15px] font-medium text-[#E5484D]">Disconnect Coinbase</p>
                <p className="text-[13px] text-[#E5484D]/70 mt-0.5">Reset your wallet connection</p>
              </div>
            </button>
          </div>
        </div>

        {/* Version Footer */}
        <div className="text-center text-[12px] text-[var(--text-tertiary)] font-mono">
          v{placeholderData.app.version}
        </div>
      </div>
    </div>
  )
}

function SettingsRow({ label, description, checked, onChange, isLast }: {
  label: string
  description: string
  checked: boolean
  onChange: (v: boolean) => void
  isLast?: boolean
}) {
  return (
    <div className={`flex justify-between items-center px-4 py-3.5 ${isLast ? '' : 'border-b border-[var(--border-subtle)]'}`}>
      <div className="pr-4">
        <p className="text-[15px] font-medium text-[var(--text-primary)]">{label}</p>
        <p className="text-[13px] text-[var(--text-tertiary)] mt-0.5 leading-snug">{description}</p>
      </div>
      <Toggle checked={checked} onChange={onChange} />
    </div>
  )
}

function ActionRow({ icon: Icon, label, description }: {
  icon: React.ComponentType<{ size?: number; className?: string }>
  label: string
  description: string
}) {
  return (
    <button type="button" className="flex items-center gap-3 w-full px-4 py-3.5 bg-transparent border-b border-[var(--border-subtle)] cursor-pointer text-left hover:bg-[var(--surface-hover)] transition-colors border-x-0 border-t-0">
      <Icon size={18} className="text-[var(--text-secondary)]" />
      <div>
        <p className="text-[15px] font-medium text-[var(--text-primary)]">{label}</p>
        <p className="text-[13px] text-[var(--text-tertiary)] mt-0.5">{description}</p>
      </div>
    </button>
  )
}

function ToolRow({ result, toolState, actionInProgress: loading, actionError: error, onProvision, onUnprovision: _onUnprovision, isLast }: {
  result: DetectionResult
  toolState: ToolProvisioningState | undefined
  actionInProgress: boolean
  actionError: string | null
  onProvision: () => void
  onUnprovision: () => void
  isLast: boolean
}) {
  const isProvisioned = toolState?.status === 'provisioned'
  const needsUpdate = toolState?.status === 'needs_update'
  const isRemoved = toolState?.status === 'removed'
  const detected = result.detected

  let description = 'Not installed on this machine'
  if (detected && isProvisioned) description = 'Wallet connected'
  else if (detected && needsUpdate) description = 'Update available'
  else if (detected && isRemoved) description = 'Previously connected'
  else if (detected) description = 'Detected, not connected'

  return (
    <div className={`px-4 py-3.5 ${!detected ? 'opacity-50' : ''} ${isLast ? '' : 'border-b border-[var(--border-subtle)]'}`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3 min-w-0">
          <Plug size={18} className="text-[var(--text-secondary)] shrink-0" />
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <p className="text-[15px] font-medium text-[var(--text-primary)]">{TOOL_NAMES[result.tool] ?? result.tool}</p>
              {isProvisioned && (
                <span className="text-[11px] font-semibold px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400">Connected</span>
              )}
              {needsUpdate && (
                <span className="text-[11px] font-semibold px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-400">Update</span>
              )}
            </div>
            <p className="text-[13px] text-[var(--text-tertiary)] mt-0.5">{description}</p>
          </div>
        </div>
        {detected && (
          <div className="shrink-0 ml-3">
            {isProvisioned ? (
              <button
                type="button"
                onClick={onProvision}
                disabled={loading}
                className="text-[12px] font-semibold px-3 py-1.5 rounded-full bg-[var(--bg-elevated)] text-[var(--text-secondary)] hover:bg-[var(--surface-hover)] transition-colors border border-[var(--border-subtle)] disabled:opacity-50"
              >
                {loading ? <Loader2 size={14} className="animate-spin" /> : 'Reinstall'}
              </button>
            ) : needsUpdate ? (
              <button
                type="button"
                onClick={onProvision}
                disabled={loading}
                className="text-[12px] font-semibold px-3 py-1.5 rounded-full bg-[var(--brand)] text-white hover:opacity-90 transition-opacity disabled:opacity-50"
              >
                {loading ? <Loader2 size={14} className="animate-spin" /> : 'Update'}
              </button>
            ) : (
              <button
                type="button"
                onClick={onProvision}
                disabled={loading}
                className="text-[12px] font-semibold px-3 py-1.5 rounded-full bg-[var(--brand)] text-white hover:opacity-90 transition-opacity disabled:opacity-50"
              >
                {loading ? <Loader2 size={14} className="animate-spin" /> : isRemoved ? 'Reinstall' : 'Install'}
              </button>
            )}
          </div>
        )}
      </div>
      {error && <p className="text-[12px] text-[#E5484D] mt-1">{error}</p>}
    </div>
  )
}
