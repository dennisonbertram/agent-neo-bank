import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ChevronRight } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { Toggle } from '../components/ui/Toggle'
import { BottomNav } from '../components/layout/BottomNav'
import placeholderData from '../data/placeholder_data.json'

export default function Settings() {
  const navigate = useNavigate()
  const user = placeholderData.user
  const defaults = placeholderData.notifications.defaults

  const [agentRequests, setAgentRequests] = useState(defaults.on_agent_registration)
  const [txCompleted, setTxCompleted] = useState(defaults.on_all_tx)
  const [approvalRequired, setApprovalRequired] = useState(defaults.on_limit_requests)
  const [dailyLimit, setDailyLimit] = useState(false)
  const [lowBalance, setLowBalance] = useState(defaults.on_errors)

  const handleLogout = () => {
    if (window.confirm('Are you sure you want to reset your Coinbase connection?')) {
      navigate('/onboarding', { replace: true })
    }
  }

  return (
    <div className="flex flex-col h-full relative">
      <div className="flex-1 overflow-y-auto px-6 pt-[60px] pb-[100px] scrollbar-hide">
        {/* Back button */}
        <Button
          variant="sm-outline"
          onClick={() => navigate('/home')}
          className="mb-4"
        >
          ← Home
        </Button>

        {/* Profile header */}
        <div className="flex items-center gap-4 py-6 border-b border-[var(--surface-hover)] mb-6">
          <div className="w-[64px] h-[64px] rounded-full bg-[var(--accent-terracotta)] flex items-center justify-center text-white text-[24px] font-semibold">
            {user.initials}
          </div>
          <div>
            <p className="text-subtitle mb-0.5">{user.name}</p>
            <p className="text-[14px] text-[var(--text-secondary)]">{user.email}</p>
          </div>
        </div>

        {/* Notifications section */}
        <div className="mb-8">
          <span className="text-caption block mb-3">Notifications</span>

          <SettingsRow label="Agent Requests" description="New agent registration alerts" checked={agentRequests} onChange={setAgentRequests} />
          <SettingsRow label="Transaction Completed" description="Confirmation when transactions settle" checked={txCompleted} onChange={setTxCompleted} />
          <SettingsRow label="Approval Required" description="When agents need spending approval" checked={approvalRequired} onChange={setApprovalRequired} />
          <SettingsRow label="Daily Limit Reached" description="Alert when daily budget is exhausted" checked={dailyLimit} onChange={setDailyLimit} />
          <SettingsRow label="Low Balance" description="Warning when wallet balance is low" checked={lowBalance} onChange={setLowBalance} />
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

      <BottomNav activeTab="settings" />
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
