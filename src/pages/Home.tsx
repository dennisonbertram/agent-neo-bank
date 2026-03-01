import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Search, Rocket, Landmark } from 'lucide-react'
import { SegmentControl } from '../components/ui/SegmentControl'
import { AgentPillRow } from '../components/agent/AgentPillRow'
import { TransactionItem } from '../components/transaction/TransactionItem'
import { TopBar } from '../components/layout/TopBar'
import { ScreenHeader } from '../components/layout/ScreenHeader'
import { useWalletStore } from '../stores/walletStore'
import { safeTauriCall, tauriApi, placeholderData } from '../lib/tauri'
import type { Transaction, Agent, AgentBudgetSummary } from '../types'

/** Format tx amount: strip existing sign, use typographical minus for sends */
function formatTxAmount(amount: string, txType: string): string {
  const abs = amount.replace(/^-/, '')
  return txType === 'receive' ? `+$${abs}` : `\u2212$${abs}`
}

/** Map placeholder agent data to the shape the Agent pills need */
const placeholderAgents = placeholderData.agents.samples
const placeholderBudgets = placeholderData.budgetSummaries.samples

/** Icon lookup for agent types */
const AGENT_ICON: Record<string, typeof Search> = {
  research: Search,
  deployment: Rocket,
  treasury: Landmark,
}

/** Accent colour lookup for agent types */
const AGENT_ACCENT: Record<string, string> = {
  research: '#2c98d6',
  deployment: '#df9e33',
  treasury: '#ed5a5a',
}

export default function Home() {
  const navigate = useNavigate()
  const [segment, setSegment] = useState('Overview')
  const [loading, setLoading] = useState(true)
  const [showDetails, setShowDetails] = useState(false)

  // Wallet data from global store (fetched once at app level)
  const { address: walletAddress, balances, totalBalance } = useWalletStore()

  // Page-specific data (transactions, agents, budgets)
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [agents, setAgents] = useState<Agent[]>([])
  const [budgets, setBudgets] = useState<AgentBudgetSummary[]>([])

  useEffect(() => {
    let cancelled = false

    async function load() {
      const [txRes, agentRes, budgetRes] = await Promise.all([
        safeTauriCall(
          () => tauriApi.transactions.list({ limit: 5, offset: 0 }),
          null,
        ),
        safeTauriCall(
          () => tauriApi.agents.list(),
          [] as Agent[],
        ),
        safeTauriCall(
          () => tauriApi.budget.getAgentSummaries(),
          [] as AgentBudgetSummary[],
        ),
      ])

      if (cancelled) return

      setTransactions(txRes?.transactions ?? [])
      setAgents(agentRes ?? [])
      setBudgets(budgetRes ?? [])
      setLoading(false)
    }

    load()
    return () => { cancelled = true }
  }, [])

  // Derive display values — fall back to placeholder when backend returns empty / null
  const totalBalanceUsd = totalBalance ?? placeholderData.wallet.totalBalanceUsd

  // Use real transactions if available, otherwise placeholder samples
  const displayTransactions: Array<{
    id: string
    agent_name: string | null
    tx_type: string
    amount: string
    asset: string
    category: string
    description: string
  }> = transactions.length > 0
    ? transactions.map(tx => ({
        id: tx.id,
        agent_name: agents.find(a => a.id === tx.agent_id)?.name ?? null,
        tx_type: tx.tx_type,
        amount: tx.amount,
        asset: tx.asset,
        category: tx.category,
        description: tx.description,
      }))
    : placeholderData.transactions.samples

  // Build agent pills from real data or placeholder
  const agentPills = agents.length > 0
    ? agents.map(agent => {
        const budget = budgets.find(b => b.agent_id === agent.id)
        return {
          id: agent.id,
          label: agent.name,
          value: budget ? `$${budget.daily_spent}` : '$0.00',
          subValue: agent.status === 'active' ? 'today' : agent.status,
          accentColor: AGENT_ACCENT[agent.agent_type] ?? '#2c98d6',
          icon: AGENT_ICON[agent.agent_type] ?? Search,
        }
      })
    : placeholderAgents.map((a, i) => ({
        id: a.id,
        label: a.name,
        value: `$${placeholderBudgets[i]?.daily_spent ?? '0.00'}`,
        subValue: a.status === 'active' ? 'today' : a.status,
        accentColor: a.accentColor,
        icon: AGENT_ICON[a.agent_type] ?? Search,
      }))

  if (loading) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex-1 flex items-center justify-center">
          <div className="w-6 h-6 border-2 border-[var(--text-tertiary)]/30 border-t-[var(--text-primary)] rounded-full animate-spin" />
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Top bar with avatar + settings */}
      <TopBar />

      {/* Fixed header area — never scrolls */}
      <div className="flex-none">
        <div className="px-6 pt-4">
          {/* Balance Card — elevated dark surface */}
          <div className="bg-[var(--bg-secondary)] text-[var(--text-primary)] rounded-[var(--radius-xl)] p-8 mb-6 relative overflow-hidden flex flex-col justify-between min-h-[198px] border border-[var(--border-subtle)]">
            {/* Decorative glow — brand purple */}
            <div className="absolute -top-1/2 -right-[20%] w-[200px] h-[200px] bg-[radial-gradient(circle,rgba(73,73,241,0.15)_0%,transparent_70%)] pointer-events-none" />

            <div className="flex justify-between items-start relative z-[1]">
              <div>
                <p className="text-[40px] font-semibold leading-none tabular-nums">${totalBalanceUsd}</p>
              </div>
              <div className="bg-[var(--brand-container)] px-2.5 py-1 rounded-[var(--radius-sm)] flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full bg-[#0052FF]" />
                <span className="text-[11px] font-semibold tracking-[0.5px] text-[var(--brand-on-container)]">BASE</span>
              </div>
            </div>

            <div className="flex justify-between items-center relative z-[1]">
              <span className="font-mono text-[13px] text-[var(--text-tertiary)]">
                {walletAddress ? `${walletAddress.slice(0, 6)}...${walletAddress.slice(-4)}` : '...'}
              </span>
              <div className="flex gap-2 items-center">
                <button
                  type="button"
                  onClick={() => navigate('/add-funds')}
                  className="text-[11px] font-medium text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] bg-transparent border-none cursor-pointer p-0 transition-colors"
                >
                  Add Funds
                </button>
                <button
                  type="button"
                  onClick={() => setShowDetails(d => !d)}
                  className="text-[11px] font-semibold text-[var(--text-secondary)] bg-[var(--surface-hover)] hover:bg-[var(--border-subtle)] border-none cursor-pointer px-2.5 py-1 rounded-full transition-colors"
                >
                  Details
                </button>
              </div>
            </div>
          </div>

          {!showDetails && (
            <SegmentControl
              options={['Overview', 'Agents']}
              value={segment}
              onChange={setSegment}
              className="mb-6"
            />
          )}
        </div>
      </div>

      {showDetails ? (
        <>
          <div className="px-6 pt-2">
            <ScreenHeader
              title="Balances"
              onBack={() => setShowDetails(false)}
            />
          </div>
          <div className="flex-1 overflow-y-auto scrollbar-hide px-6">
            <div className="bg-[var(--bg-secondary)] rounded-[var(--radius-lg)] p-5 border border-[var(--border-subtle)]">
              <h3 className="text-caption mb-4">Token Balances</h3>
              {Object.entries(balances ?? {}).map(([symbol, asset], i, arr) => (
                <div key={symbol} className={`flex justify-between items-center py-3 ${i < arr.length - 1 ? 'border-b border-[var(--border-subtle)]' : ''}`}>
                  <div className="flex items-center gap-3">
                    <div className="w-2 h-2 rounded-full" style={{ backgroundColor: symbol === 'ETH' ? '#627EEA' : '#2775CA' }} />
                    <span className="text-[15px] font-medium text-[var(--text-primary)]">{symbol}</span>
                  </div>
                  <span className="font-mono text-[15px] font-semibold text-[var(--text-primary)] tabular-nums">
                    {asset.formatted} {symbol}
                  </span>
                </div>
              ))}
              {(!balances || Object.keys(balances).length === 0) && (
                <p className="text-body text-center py-4">No token balances available</p>
              )}
            </div>
          </div>
        </>
      ) : (
        /* Content area */
        <div className="flex-1 min-h-0 flex flex-col px-6">
          {segment === 'Agents' ? (
            /* Agent Pills */
            <div className="flex flex-col gap-3 pb-6 flex-1 overflow-y-auto scrollbar-hide">
              {agentPills.map(pill => (
                <AgentPillRow
                  key={pill.id}
                  icon={pill.icon}
                  label={pill.label}
                  value={pill.value}
                  subValue={pill.subValue}
                  accentColor={pill.accentColor}
                  onClick={() => navigate(`/agents/${pill.id}`)}
                />
              ))}
            </div>
          ) : (
            /* Recent Activity Feed — max 4 items */
            <div className="flex flex-col">
              <div className="flex justify-between items-end mb-3">
                <h3 className="text-[16px] font-semibold text-[var(--text-primary)] leading-none">
                  Recent Activity
                </h3>
                <button
                  type="button"
                  onClick={() => navigate('/transactions')}
                  className="text-[13px] font-medium text-[var(--brand-main)] hover:text-[var(--brand-on-container)] bg-transparent border-none cursor-pointer transition-colors leading-none"
                >
                  View All
                </button>
              </div>

              {displayTransactions.slice(0, 4).map((tx, i, arr) => (
                <TransactionItem
                  key={tx.id}
                  label={tx.agent_name || 'Deposit'}
                  subLabel={tx.description}
                  amount={formatTxAmount(tx.amount, tx.tx_type)}
                  tag={tx.category.toUpperCase()}
                  isPositive={tx.tx_type === 'receive'}
                  isLast={i === arr.length - 1}
                  onClick={() => navigate(`/transactions/${tx.id}`)}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
