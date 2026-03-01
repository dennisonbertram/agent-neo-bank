import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Plus, Users, Search, Rocket, Landmark, ArrowDownLeft, Code, Settings } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { SegmentControl } from '../components/ui/SegmentControl'
import { AgentPillRow } from '../components/agent/AgentPillRow'
import { TransactionItem } from '../components/transaction/TransactionItem'
import { useWalletStore } from '../stores/walletStore'
import { safeTauriCall, tauriApi, placeholderData } from '../lib/tauri'
import type { Transaction, Agent, AgentBudgetSummary } from '../types'

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
  research: '#8FB5AA',
  deployment: '#F2D48C',
  treasury: '#D9A58B',
}

export default function Home() {
  const navigate = useNavigate()
  const [segment, setSegment] = useState('Overview')
  const [loading, setLoading] = useState(true)

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
  const ethFormatted = balances?.ETH?.formatted ?? placeholderData.wallet.balances.ETH.formatted
  const usdcFormatted = balances?.USDC?.formatted ?? placeholderData.wallet.balances.USDC.formatted

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
          accentColor: AGENT_ACCENT[agent.agent_type] ?? '#8FB5AA',
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

  const user = placeholderData.user

  if (loading) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex-1 flex items-center justify-center">
          <div className="w-6 h-6 border-2 border-black/20 border-t-black rounded-full animate-spin" />
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Floating settings button — bottom right */}
      <button
        type="button"
        onClick={() => navigate('/settings')}
        className="fixed bottom-6 right-6 z-50 w-[44px] h-[44px] rounded-full bg-black/80 backdrop-blur-sm flex items-center justify-center cursor-pointer border-none shadow-lg hover:bg-black transition-colors"
      >
        <Settings size={20} color="white" />
      </button>

      {/* Fixed header area — never scrolls */}
      <div className="flex-none">
        <div className="px-6 pt-6">
          {/* Balance Card */}
          <div className="bg-black text-white rounded-[32px] p-8 mb-6 relative overflow-hidden flex flex-col justify-between min-h-[198px]">
            {/* Decorative glow */}
            <div className="absolute -top-1/2 -right-[20%] w-[200px] h-[200px] bg-[radial-gradient(circle,rgba(143,181,170,0.2)_0%,transparent_70%)] pointer-events-none" />

            <div className="flex justify-between items-start relative z-[1]">
              <div>
                <p className="text-[40px] font-semibold leading-none">${totalBalanceUsd}</p>
              </div>
              <div className="bg-white/10 px-2 py-1 rounded-[8px] flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full bg-[#0052FF]" />
                <span className="text-[11px] font-semibold tracking-[0.5px]">BASE</span>
              </div>
            </div>

            <div className="flex justify-between items-center relative z-[1]">
              <span className="font-mono text-[13px] text-white/50">
                {walletAddress ? `${walletAddress.slice(0, 6)}...${walletAddress.slice(-4)}` : '...'}
              </span>
              <div className="flex gap-3">
                <span className="text-[12px] font-medium">{ethFormatted} ETH</span>
                <span className="text-[12px] font-medium">{usdcFormatted} USDC</span>
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-3 mb-8">
            <Button variant="action" onClick={() => navigate('/add-funds')}>
              <Plus size={20} strokeWidth={2.5} />
              Add Funds
            </Button>
            <Button variant="action" onClick={() => navigate('/agents')}>
              <Users size={20} strokeWidth={2.5} />
              Agents
            </Button>
          </div>

          {/* Segment Control */}
          <SegmentControl
            options={['Overview', 'Agents']}
            value={segment}
            onChange={setSegment}
            className="mb-6"
          />
        </div>
      </div>

      {/* Scrollable content area — only this part scrolls */}
      <div className="flex-1 overflow-y-auto scrollbar-hide px-6">
        {segment === 'Agents' ? (
          /* Agent Pills */
          <div className="flex flex-col gap-3 pb-6">
            {agentPills.map(pill => (
              <AgentPillRow
                key={pill.id}
                icon={pill.icon}
                label={pill.label}
                value={pill.value}
                subValue={pill.subValue}
                accentColor={pill.accentColor}
              />
            ))}
          </div>
        ) : (
          /* Activity Feed */
          <div className="pb-6">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-title" style={{ fontSize: 20 }}>Activity</h3>
              <button
                type="button"
                className="text-[13px] font-semibold text-[#0052FF] bg-transparent border-none cursor-pointer"
              >
                View All
              </button>
            </div>

            {displayTransactions.map((tx, i) => (
              <TransactionItem
                key={tx.id}
                icon={
                  tx.tx_type === 'receive' ? ArrowDownLeft :
                  tx.agent_name === 'Deploy Bot' ? Code : Search
                }
                iconBgColor={
                  tx.tx_type === 'receive' ? 'var(--accent-blue-dim)' :
                  tx.agent_name === 'Deploy Bot' ? 'var(--accent-yellow-dim)' :
                  tx.agent_name === 'Treasury' ? 'var(--accent-terracotta-dim)' :
                  'var(--accent-green-dim)'
                }
                label={tx.agent_name || 'Deposit'}
                subLabel={tx.description}
                amount={`${tx.amount} ${tx.asset}`}
                tag={tx.category.toUpperCase()}
                isPositive={tx.tx_type === 'receive'}
                isLast={i === displayTransactions.length - 1}
                onClick={() => navigate(`/transactions/${tx.id}`)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
