import { useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { ChevronLeft } from 'lucide-react'
import { StatusPill } from '../components/ui/StatusPill'
import { Toggle } from '../components/ui/Toggle'
import { Stepper } from '../components/ui/Stepper'
import { Button } from '../components/ui/Button'
import placeholderData from '../data/placeholder_data.json'

export default function AgentDetail() {
  const { agentId } = useParams()
  const navigate = useNavigate()

  // Find agent from placeholder data
  const agent = placeholderData.agents.samples.find((a) => a.id === agentId)
  const budget = placeholderData.budgetSummaries.samples.find((b) => b.agent_id === agentId)

  const [isPaused, setIsPaused] = useState(agent?.status === 'pending')
  const [dailyLimit, setDailyLimit] = useState(parseFloat(budget?.daily_cap || '25'))
  const [perTxLimit, setPerTxLimit] = useState(5)
  const [requireApproval, setRequireApproval] = useState(true)

  const dailySpent = parseFloat(budget?.daily_spent || '0')
  const percentage = dailyLimit > 0 ? (dailySpent / dailyLimit) * 100 : 0

  // Sample transaction history
  const history = [
    { id: 1, name: 'Arxiv API Call', time: 'Today, 2:45 PM', amount: '-$1.20' },
    { id: 2, name: 'Cross-Chain Query', time: 'Today, 11:20 AM', amount: '-$3.80' },
    { id: 3, name: 'Metadata Storage', time: 'Yesterday, 9:15 PM', amount: '-$1.50' },
  ]

  return (
    <div className="flex flex-col h-full">
      {/* Sticky header */}
      <header className="sticky top-0 z-10 pt-[60px] pb-4 px-6 flex items-center justify-between bg-white/90 backdrop-blur-[10px]">
        <button
          type="button"
          onClick={() => navigate(-1)}
          className="w-[40px] h-[40px] rounded-full bg-[var(--bg-secondary)] flex items-center justify-center border-none cursor-pointer"
        >
          <ChevronLeft size={20} />
        </button>
        <StatusPill status={isPaused ? 'paused' : 'active'} />
      </header>

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
          <div className="h-[6px] bg-black/5 rounded-[3px] mt-3 overflow-hidden">
            <div
              className="h-full rounded-[3px] transition-all duration-300"
              style={{
                width: `${Math.min(percentage, 100)}%`,
                backgroundColor: agent?.accentColor || 'var(--accent-green)',
                opacity: isPaused ? 0.3 : 1,
              }}
            />
          </div>
          <div className="flex justify-between mt-2">
            <span className="text-[11px] font-semibold uppercase tracking-[0.5px]" style={{ color: agent?.accentColor || 'var(--accent-green)' }}>
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
            <button type="button" className="text-[11px] font-semibold uppercase tracking-[0.5px] text-black border-b border-black bg-transparent cursor-pointer">
              Filter
            </button>
          </div>

          {history.map((tx, i) => (
            <div
              key={tx.id}
              className={`flex justify-between items-center py-3 ${i < history.length - 1 ? 'border-b border-[var(--surface-hover)]' : ''}`}
            >
              <div>
                <p className="text-[15px] font-medium text-[var(--text-primary)]">{tx.name}</p>
                <p className="text-[10px] font-semibold uppercase tracking-[0.5px] text-[var(--text-secondary)] mt-0.5">
                  {tx.time} • Success
                </p>
              </div>
              <span className="text-[15px] font-bold text-[var(--text-primary)]">{tx.amount}</span>
            </div>
          ))}
        </div>

        {/* Save button */}
        <Button variant="primary" className="mt-8" onClick={() => navigate(-1)}>
          Save Changes
        </Button>
      </main>
    </div>
  )
}
