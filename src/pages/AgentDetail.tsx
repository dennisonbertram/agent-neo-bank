import { useState, useEffect, useCallback } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { StatusPill } from '../components/ui/StatusPill'
import { ScreenHeader } from '../components/layout/ScreenHeader'
import { Toggle } from '../components/ui/Toggle'
import { Stepper } from '../components/ui/Stepper'
import { Button } from '../components/ui/Button'
import { safeTauriCall, tauriApi, placeholderData } from '../lib/tauri'
import type { Agent, SpendingPolicy, Transaction, AgentBudgetSummary } from '../types'

export default function AgentDetail() {
  const { agentId } = useParams()
  const navigate = useNavigate()

  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [agent, setAgent] = useState<Agent | null>(null)
  const [budget, setBudget] = useState<AgentBudgetSummary | null>(null)
  const [transactions, setTransactions] = useState<Transaction[]>([])

  const [isPaused, setIsPaused] = useState(false)
  const [dailyLimit, setDailyLimit] = useState(25)
  const [perTxLimit, setPerTxLimit] = useState(5)
  const [requireApproval, setRequireApproval] = useState(true)

  // Accent color from placeholder data (not stored in backend Agent type)
  const placeholderAgent = placeholderData.agents.samples.find((a) => a.id === agentId)
  const accentColor = placeholderAgent?.accentColor || 'var(--accent-green)'

  const loadData = useCallback(async () => {
    if (!agentId) return
    setLoading(true)

    const fallbackAgent = placeholderData.agents.samples.find((a) => a.id === agentId)
    const fallbackBudget = placeholderData.budgetSummaries.samples.find((b) => b.agent_id === agentId)

    const [agentData, policyData, txData, budgetSummaries] = await Promise.all([
      safeTauriCall(
        () => tauriApi.agents.get(agentId),
        fallbackAgent
          ? ({
              id: fallbackAgent.id,
              name: fallbackAgent.name,
              description: fallbackAgent.description,
              purpose: fallbackAgent.purpose || '',
              agent_type: fallbackAgent.agent_type,
              capabilities: fallbackAgent.capabilities,
              status: fallbackAgent.status as Agent['status'],
              api_token_hash: null,
              token_prefix: null,
              balance_visible: true,
              invitation_code: null,
              created_at: 0,
              updated_at: 0,
              last_active_at: null,
              metadata: '{}',
            } satisfies Agent)
          : null,
      ),
      safeTauriCall(
        () => tauriApi.agents.getPolicy(agentId),
        fallbackBudget
          ? ({
              agent_id: agentId,
              per_tx_max: '5.00',
              daily_cap: fallbackBudget.daily_cap,
              weekly_cap: fallbackBudget.weekly_cap,
              monthly_cap: fallbackBudget.monthly_cap,
              auto_approve_max: '5.00',
              allowlist: [],
              updated_at: 0,
            } satisfies SpendingPolicy)
          : null,
      ),
      safeTauriCall(
        () => tauriApi.agents.getTransactions(agentId, 5),
        placeholderData.transactions.samples
          .filter((t) => t.agent_id === agentId)
          .slice(0, 5) as unknown as Transaction[],
      ),
      safeTauriCall(
        () => tauriApi.budget.getAgentSummaries(),
        placeholderData.budgetSummaries.samples as unknown as AgentBudgetSummary[],
      ),
    ])

    setAgent(agentData as Agent | null)
    setTransactions(txData)

    const matchedBudget = budgetSummaries.find((b) => b.agent_id === agentId) ?? null
    setBudget(matchedBudget)

    if (agentData) {
      setIsPaused(agentData.status === 'pending' || agentData.status === 'suspended')
    }
    if (policyData) {
      setDailyLimit(parseFloat(policyData.daily_cap))
      setPerTxLimit(parseFloat(policyData.per_tx_max))
      setRequireApproval(parseFloat(policyData.auto_approve_max) < parseFloat(policyData.per_tx_max))
    }

    setLoading(false)
  }, [agentId])

  useEffect(() => {
    loadData()
  }, [loadData])

  const handleSave = async () => {
    if (!agentId) return
    setSaving(true)
    await safeTauriCall(
      () =>
        tauriApi.agents.updatePolicy({
          agent_id: agentId,
          per_tx_max: perTxLimit.toFixed(2),
          daily_cap: dailyLimit.toFixed(2),
          weekly_cap: (dailyLimit * 7).toFixed(2),
          monthly_cap: (dailyLimit * 30).toFixed(2),
          auto_approve_max: requireApproval ? perTxLimit.toFixed(2) : dailyLimit.toFixed(2),
          allowlist: [],
          updated_at: Math.floor(Date.now() / 1000),
        }),
      undefined,
    )
    setSaving(false)
    navigate(-1)
  }

  const dailySpent = parseFloat(budget?.daily_spent || '0')
  const percentage = dailyLimit > 0 ? (dailySpent / dailyLimit) * 100 : 0

  if (loading) {
    return (
      <div className="flex flex-col h-full items-center justify-center">
        <div className="w-8 h-8 border-2 border-[var(--text-secondary)] border-t-transparent rounded-full animate-spin" />
        <p className="text-caption mt-3">Loading agent...</p>
      </div>
    )
  }

  const formatTxTime = (createdAt: number) => {
    if (!createdAt) return ''
    const date = new Date(createdAt * 1000)
    const now = new Date()
    const isToday = date.toDateString() === now.toDateString()
    const yesterday = new Date(now)
    yesterday.setDate(yesterday.getDate() - 1)
    const isYesterday = date.toDateString() === yesterday.toDateString()

    const timeStr = date.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })
    if (isToday) return `Today, ${timeStr}`
    if (isYesterday) return `Yesterday, ${timeStr}`
    return `${date.toLocaleDateString([], { month: 'short', day: 'numeric' })}, ${timeStr}`
  }

  return (
    <div className="flex flex-col h-full">
      <ScreenHeader
        title={agent?.name || 'Agent'}
        rightElement={<StatusPill status={isPaused ? 'paused' : 'active'} />}
      />

      {/* Scrollable content */}
      <main className="flex-1 overflow-y-auto px-6 pb-[40px] animate-in">
        {/* Agent identity */}
        <div className="mt-3">
          <p className="text-caption">Local Agent</p>
          <h1 className="text-[28px] font-bold text-[var(--text-primary)] mt-1">
            {agent?.name || 'Agent'}
          </h1>
          <p className="text-body mt-2">{agent?.description || ''}</p>
        </div>

        {/* Daily Spend Card */}
        <div className="bg-[var(--bg-secondary)] rounded-[24px] p-5 mt-6">
          <div className="flex justify-between items-end">
            <div>
              <p className="text-caption mb-1">Daily Spend</p>
              <p className="text-[28px] font-bold text-[var(--text-primary)]">
                ${dailySpent.toFixed(2)}
                <span className="text-[14px] text-[var(--text-secondary)] font-medium"> / ${dailyLimit.toFixed(2)}</span>
              </p>
            </div>
            <Toggle checked={isPaused} onChange={setIsPaused} />
          </div>

          {/* Progress bar */}
          <div className="h-[6px] bg-[var(--surface-hover)] rounded-[3px] mt-3 overflow-hidden">
            <div
              className="h-full rounded-[3px] transition-all duration-300"
              style={{
                width: `${Math.min(percentage, 100)}%`,
                backgroundColor: accentColor,
                opacity: isPaused ? 0.3 : 1,
              }}
            />
          </div>
          <div className="flex justify-between mt-2">
            <span className="text-[11px] font-semibold uppercase tracking-[0.5px]" style={{ color: accentColor }}>
              {Math.round(percentage)}% Used
            </span>
            <span className="text-[11px] font-semibold uppercase tracking-[0.5px] text-[var(--text-secondary)]">
              Reset in 14h
            </span>
          </div>
        </div>

        {/* Spending Controls */}
        <div className="mt-8">
          <h2 className="text-title mb-4">Spending Controls</h2>

          <div className="border-b border-black/5 py-4">
            <Stepper
              label="Daily Limit"
              value={dailyLimit}
              onChange={setDailyLimit}
              step={5}
              min={0}
              max={1000}
            />
            <p className="text-[12px] text-[var(--text-secondary)] mt-1">Max spend per 24h</p>
          </div>

          <div className="border-b border-black/5 py-4">
            <Stepper
              label="Per Transaction"
              value={perTxLimit}
              onChange={setPerTxLimit}
              step={1}
              min={0}
              max={500}
            />
            <p className="text-[12px] text-[var(--text-secondary)] mt-1">Auto-approve cap</p>
          </div>

          <div className="py-4 flex justify-between items-center">
            <div>
              <span className="text-[15px] font-semibold text-[var(--text-primary)]">Approval Threshold</span>
              <p className="text-[12px] text-[var(--text-secondary)] mt-1">Prompt for any tx &gt; ${perTxLimit.toFixed(2)}</p>
            </div>
            <Toggle checked={requireApproval} onChange={setRequireApproval} />
          </div>
        </div>

        {/* Agent History */}
        <div className="mt-10">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-title">Agent History</h2>
            <button type="button" className="text-[11px] font-semibold uppercase tracking-[0.5px] text-[var(--text-secondary)] border-b border-[var(--text-secondary)] bg-transparent cursor-pointer">
              Filter
            </button>
          </div>

          {transactions.length === 0 && (
            <p className="text-body text-center py-6">No transactions yet</p>
          )}

          {transactions.map((tx, i) => (
            <div
              key={tx.id}
              className={`flex justify-between items-center py-3 ${i < transactions.length - 1 ? 'border-b border-[var(--surface-hover)]' : ''}`}
            >
              <div>
                <p className="text-[15px] font-medium text-[var(--text-primary)]">{tx.description || tx.category}</p>
                <p className="text-[10px] font-semibold uppercase tracking-[0.5px] text-[var(--text-secondary)] mt-0.5">
                  {formatTxTime(tx.created_at)} {tx.status === 'confirmed' ? '• Success' : `• ${tx.status}`}
                </p>
              </div>
              <span className="text-[15px] font-bold text-[var(--text-primary)]">
                {tx.tx_type === 'receive' ? '+' : '-'}${Math.abs(parseFloat(tx.amount)).toFixed(2)}
              </span>
            </div>
          ))}
        </div>

        {/* Save button */}
        <Button variant="primary" className="mt-8" onClick={handleSave} disabled={saving}>
          {saving ? 'Saving...' : 'Save Changes'}
        </Button>
      </main>
    </div>
  )
}
