import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { ScreenHeader } from '../components/layout/ScreenHeader'
import { TransactionItem } from '../components/transaction/TransactionItem'
import { safeTauriCall, tauriApi, placeholderData } from '../lib/tauri'
import type { Transaction, Agent } from '../types'

const PAGE_SIZE = 20

/** Format tx amount: strip existing sign, use typographical minus for sends */
function formatTxAmount(amount: string, txType: string): string {
  const abs = amount.replace(/^-/, '')
  return txType === 'receive' ? `+$${abs}` : `\u2212$${abs}`
}

export default function AllTransactions() {
  const navigate = useNavigate()
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [agents, setAgents] = useState<Agent[]>([])
  const [loading, setLoading] = useState(true)
  const [page, setPage] = useState(0)
  const [total, setTotal] = useState(0)

  useEffect(() => {
    let cancelled = false

    async function load() {
      setLoading(true)
      const [txRes, agentRes] = await Promise.all([
        safeTauriCall(
          () => tauriApi.transactions.list({ limit: PAGE_SIZE, offset: page * PAGE_SIZE }),
          null,
        ),
        safeTauriCall(
          () => tauriApi.agents.list(),
          [] as Agent[],
        ),
      ])

      if (cancelled) return

      setTransactions(txRes?.transactions ?? [])
      setTotal(txRes?.total ?? 0)
      setAgents(agentRes ?? [])
      setLoading(false)
    }

    load()
    return () => { cancelled = true }
  }, [page])

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

  const totalPages = Math.max(1, Math.ceil((total || displayTransactions.length) / PAGE_SIZE))
  const hasPrev = page > 0
  const hasNext = page < totalPages - 1

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <ScreenHeader title="All Activity" />

      {/* Scrollable transaction list */}
      <div className="flex-1 overflow-y-auto scrollbar-hide px-6">
        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="w-6 h-6 border-2 border-[var(--text-tertiary)]/30 border-t-[var(--text-primary)] rounded-full animate-spin" />
          </div>
        ) : displayTransactions.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 gap-3">
            <span className="text-[var(--text-tertiary)] text-[14px]">No activity yet</span>
          </div>
        ) : (
          displayTransactions.map((tx, i, arr) => (
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
          ))
        )}
      </div>

      {/* Footer pagination */}
      {totalPages > 1 && (
        <div className="flex justify-between items-center px-6 py-3 flex-none border-t border-[var(--border-subtle)]">
          <button
            type="button"
            disabled={!hasPrev}
            onClick={() => setPage(p => p - 1)}
            className="px-4 py-2 bg-[var(--bg-secondary)] rounded-[var(--radius-sm)] text-[13px] font-medium text-[var(--text-secondary)] border-none cursor-pointer disabled:opacity-30 disabled:cursor-default hover:bg-[var(--surface-hover)] transition-colors"
          >
            Previous
          </button>
          <span className="text-[13px] text-[var(--text-tertiary)] tabular-nums">
            Page {page + 1} of {totalPages}
          </span>
          <button
            type="button"
            disabled={!hasNext}
            onClick={() => setPage(p => p + 1)}
            className="px-4 py-2 bg-[var(--bg-secondary)] rounded-[var(--radius-sm)] text-[13px] font-medium text-[var(--text-secondary)] border-none cursor-pointer disabled:opacity-30 disabled:cursor-default hover:bg-[var(--surface-hover)] transition-colors"
          >
            Next
          </button>
        </div>
      )}
    </div>
  )
}
