# Settings Page Review — Gemini 3.1 Pro

**Date**: 2026-03-01
**Reviewer**: Gemini 3.1 Pro Preview
**Context**: 390x640px fixed window, dark theme, React + Tailwind CSS v4

---

## TASK 1 — SETTINGS PAGE REVIEW

### 1. Profile Header & Dark Theme
Using `--accent-terracotta` (#ed5a5a) for the avatar works, but in a premium dark theme, it draws too much visual weight to a non-interactive element. It's better to use `--brand-container` with `--brand-on-container` text, which aligns the user's identity with the app's core brand color.

### 2. Notification Toggles (Spacing & Readability)
Currently, the settings are floating rows with bottom borders. Premium apps (like iOS, Mercury, or Phantom) group settings into elevated "cards" or "panels" (e.g., `bg-[var(--bg-secondary)] rounded-lg`). This creates a cleaner visual hierarchy and makes the 390px width feel less empty.

### 3. Destructive Actions
Hardcoding `#E5484D` bypasses your design system. We should use your existing `--color-danger` for the text/icon. Furthermore, adding a subtle background tint on hover (`hover:bg-[var(--variant-danger-container)]/30`) makes it feel like a polished, intentional destructive action.

### 4. What's Missing?
A premium Web3/Neobank settings screen needs:
- **Network Status**: Users need to explicitly see which chain they are connected to (e.g., Base Mainnet) and their wallet address, not just a tiny footnote.
- **Support/Help**: Links to documentation, Discord, or support.
- **Visual Icons**: Adding Lucide icons next to grouped actions (like Export and Logout) vastly improves scannability.

---

## TASK 2 — SETTINGS BUTTON PLACEMENT

### The UX Verdict on Settings placement in a 390px window:
1. **Is hover-to-reveal bad?** Yes. It's an anti-pattern ("mystery meat navigation") even on desktop. Users shouldn't have to sweep their mouse around to find core configuration.
2. **Is a permanent 36px icon button best?** It's okay, but the heavy border adds visual clutter to the header, competing with the screen titles and balances below it.
3. **The Best Approach (The "Interactive Profile Pill"):** Combine the Avatar, Wallet Name, and Settings gear into a single clickable profile button.
   - *Why?* It creates a larger, easier-to-click target. It reduces visual noise. It's a standard pattern used by Vercel, Linear, and Phantom. When the user hovers over the pill, it subtly highlights, indicating that clicking it will manage their account/settings.

---

## CODE PATCHES

### TopBar.tsx — Interactive Profile Pill Pattern

Replace your existing TopBar with this consolidated, premium interactive button.

```tsx
import { Settings } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { cn } from '../../lib/cn'

interface TopBarProps {
  walletName?: string
  initials?: string
  className?: string
}

export function TopBar({ walletName = 'Tally Wallet', initials = 'DB', className }: TopBarProps) {
  const navigate = useNavigate()

  return (
    <div
      className={cn(
        'flex items-center justify-between px-6 pt-6 pb-2',
        className
      )}
    >
      {/* Interactive Profile Pill */}
      <button
        type="button"
        onClick={() => navigate('/settings')}
        className="group flex items-center gap-2.5 p-1.5 pr-4 rounded-[var(--radius-pill)] hover:bg-[var(--surface-hover)] border border-transparent hover:border-[var(--border-subtle)] transition-all cursor-pointer bg-transparent"
        aria-label="Open Settings"
      >
        <div className="w-[32px] h-[32px] rounded-full bg-[var(--brand-container)] flex items-center justify-center text-[var(--brand-on-container)] text-[13px] font-semibold">
          {initials}
        </div>
        <span className="text-[15px] font-semibold text-[var(--text-primary)] group-hover:text-white transition-colors">
          {walletName}
        </span>
        <Settings size={16} className="text-[var(--text-tertiary)] group-hover:text-[var(--text-secondary)] transition-colors ml-1" />
      </button>
    </div>
  )
}
```

### Settings.tsx — Card-Based, Polished Hierarchy

Groups settings into elevated cards, fixes destructive styling, uses brand variables, adds missing Wallet & Support context.

```tsx
import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { Download, LogOut, LifeBuoy, Wallet } from 'lucide-react'
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

  const handleToggle = (setter: (v: boolean) => void, field: keyof NotificationPreferences) => (value: boolean) => {
    setter(value)
    savePrefs({ [field]: value })
  }

  const handleLogout = async () => {
    if (window.confirm('Are you sure you want to disconnect your Coinbase wallet?')) {
      if (isTauri()) {
        try { await tauriApi.auth.logout() } catch {}
      }
      logout()
      navigate('/onboarding', { replace: true })
    }
  }

  return (
    <div className="flex flex-col h-full relative">
      <ScreenHeader title="Settings" />
      <div className="flex-1 overflow-y-auto px-6 pb-8 scrollbar-hide">

        {/* Profile Card */}
        <div className="flex flex-col items-center py-6 mb-4">
          <div className="w-[80px] h-[80px] rounded-full bg-[var(--brand-container)] flex items-center justify-center text-[var(--brand-on-container)] text-[28px] font-semibold mb-4 border-2 border-[var(--bg-primary)] shadow-subtle">
            {user.initials}
          </div>
          <h2 className="text-[20px] font-semibold text-[var(--text-primary)] mb-1">{user.name}</h2>
          <p className="text-[14px] text-[var(--text-tertiary)] bg-[var(--bg-secondary)] px-3 py-1 rounded-full border border-[var(--border-subtle)]">
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
                <p className="text-[13px] text-[var(--text-tertiary)] mt-0.5 font-mono">0x1234...ABCD</p>
              </div>
            </div>
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
            <SettingsRow label="Low Balance" description="Warning when wallet balance is low" checked={lowBalance} onChange={handleToggle(setLowBalance, 'on_errors')} />
          </div>
        </div>

        {/* Data & Security Panel */}
        <div className="mb-8">
          <span className="text-[12px] font-semibold tracking-wider uppercase text-[var(--text-tertiary)] block mb-2 px-1">Account & Support</span>
          <div className="bg-[var(--bg-secondary)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] overflow-hidden flex flex-col">

            <ActionRow icon={Download} label="Export Wallet History" description="Download CSV of all agent activity" />
            <ActionRow icon={LifeBuoy} label="Help & Support" description="Read docs or contact us" />

            <button
              type="button"
              onClick={handleLogout}
              className="flex items-center gap-3 w-full px-4 py-3.5 bg-transparent border-t border-[var(--border-subtle)] cursor-pointer text-left hover:bg-[var(--variant-danger-container)]/30 transition-colors group"
            >
              <LogOut size={18} className="text-[var(--color-danger)] group-hover:scale-105 transition-transform" />
              <div>
                <p className="text-[15px] font-medium text-[var(--color-danger)]">Disconnect Coinbase</p>
                <p className="text-[13px] text-[var(--color-danger)]/70 mt-0.5">Reset your wallet connection</p>
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

function SettingsRow({ label, description, checked, onChange }: { label: string, description: string, checked: boolean, onChange: (v: boolean) => void }) {
  return (
    <div className="flex justify-between items-center px-4 py-3.5 border-b border-[var(--border-subtle)] last:border-b-0">
      <div className="pr-4">
        <p className="text-[15px] font-medium text-[var(--text-primary)]">{label}</p>
        <p className="text-[13px] text-[var(--text-tertiary)] mt-0.5 leading-snug">{description}</p>
      </div>
      <Toggle checked={checked} onChange={onChange} />
    </div>
  )
}

function ActionRow({ icon: Icon, label, description }: { icon: any, label: string, description: string }) {
  return (
    <button type="button" className="flex items-center gap-3 w-full px-4 py-3.5 bg-transparent border-b border-[var(--border-subtle)] last:border-b-0 cursor-pointer text-left hover:bg-[var(--surface-hover)] transition-colors">
      <Icon size={18} className="text-[var(--text-secondary)]" />
      <div>
        <p className="text-[15px] font-medium text-[var(--text-primary)]">{label}</p>
        <p className="text-[13px] text-[var(--text-tertiary)] mt-0.5">{description}</p>
      </div>
    </button>
  )
}
```
