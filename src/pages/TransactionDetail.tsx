import { useState, useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { ChevronLeft, ExternalLink, Search } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { MetaCard } from '../components/transaction/MetaCard'
import { safeTauriCall, placeholderData } from '../lib/tauri'
import { tauriApi } from '../lib/tauri'
import type { Transaction } from '../types'

export default function TransactionDetail() {
  const { txId } = useParams()
  const navigate = useNavigate()
  const [tx, setTx] = useState<Transaction | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const loadTransaction = async () => {
      if (!txId) {
        setIsLoading(false)
        return
      }
      // Build fallback from placeholder data
      const sample = placeholderData.transactions.samples.find((t) => t.id === txId)
      const fallbackTx: Transaction | null = sample
        ? {
            id: sample.id,
            agent_id: sample.agent_id ?? null,
            tx_type: sample.tx_type as Transaction['tx_type'],
            amount: sample.amount,
            asset: sample.asset,
            recipient: null,
            sender: null,
            chain_tx_hash: null,
            status: sample.status as Transaction['status'],
            category: sample.category,
            memo: sample.memo,
            description: sample.description,
            service_name: sample.service_name,
            service_url: '',
            reason: '',
            webhook_url: null,
            error_message: null,
            period_daily: '',
            period_weekly: '',
            period_monthly: '',
            created_at: sample.created_at,
            updated_at: sample.created_at,
          }
        : null

      const result = await safeTauriCall(
        () => tauriApi.transactions.get(txId),
        fallbackTx as Transaction,
      )
      setTx(result)
      setIsLoading(false)
    }
    loadTransaction()
  }, [txId])

  if (isLoading) {
    return (
      <div className="screen-scroll screen-pad-detail flex items-center justify-center h-full">
        <p className="text-body text-[var(--text-secondary)]">Loading...</p>
      </div>
    )
  }

  if (!tx) {
    return (
      <div className="screen-scroll screen-pad-detail">
        <p className="text-body">Transaction not found.</p>
      </div>
    )
  }

  // Resolve agent name: try placeholder data for display name
  const agentName = tx.agent_id
    ? placeholderData.agents.samples.find((a) => a.id === tx.agent_id)?.name ?? tx.agent_id
    : null

  const formattedDate = new Date(tx.created_at * 1000).toLocaleDateString('en-US', {
    year: 'numeric', month: 'long', day: 'numeric',
  })
  const formattedTime = new Date(tx.created_at * 1000).toLocaleTimeString('en-US', {
    hour: 'numeric', minute: '2-digit',
  })

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-y-auto px-6 pt-[60px] pb-[40px] animate-in">
        {/* Back nav */}
        <button
          type="button"
          onClick={() => navigate(-1)}
          className="flex items-center gap-2 text-[var(--text-primary)] font-semibold text-[15px] mb-6 bg-transparent border-none cursor-pointer p-0"
        >
          <ChevronLeft size={20} strokeWidth={2.5} />
          Details
        </button>

        {/* Amount hero */}
        <div className="text-center mb-8">
          <p className="text-caption">Transaction Amount</p>
          <h1 className="text-display mt-1">
            {tx.amount}
            <span className="text-[24px] text-[var(--text-secondary)] align-top ml-1">{tx.asset}</span>
          </h1>
          <p className="text-[14px] text-[var(--text-secondary)] mt-2">
            {formattedDate} • {formattedTime}
          </p>
        </div>

        {/* Agent identity row */}
        {agentName && (
          <div className="flex items-center gap-3 mb-4">
            <div className="w-[48px] h-[48px] bg-[var(--accent-green)] rounded-[14px] flex items-center justify-center">
              <Search size={24} color="black" strokeWidth={2} />
            </div>
            <div>
              <p className="text-subtitle">{agentName}</p>
              <span className="inline-block bg-[var(--accent-green-dim)] text-[#4A6E65] px-[10px] py-1 rounded-[6px] text-[11px] font-bold mt-1">
                Verified Agent
              </span>
            </div>
          </div>
        )}

        {/* Agent Metadata */}
        <h3 className="text-caption mb-2 mt-6">Agent Metadata</h3>
        <MetaCard
          title=""
          items={[
            { label: 'Category', value: tx.category },
            { label: 'Purpose', value: tx.description },
            { label: 'Request ID', value: `REQ_${tx.id.slice(-4).toUpperCase()}` },
          ]}
          className="mb-4"
        />

        {/* Cost Breakdown */}
        <h3 className="text-caption mb-2">Cost Breakdown</h3>
        <MetaCard
          title=""
          items={[
            { label: 'Service Fee', value: `${tx.amount} ${tx.asset}` },
            { label: 'Network Fee', value: '$0.00' },
          ]}
          className="mb-4"
        />

        {/* Notes */}
        {tx.memo && (
          <>
            <h3 className="text-caption mb-2">Notes</h3>
            <div className="bg-[var(--bg-secondary)] rounded-[20px] p-4 mb-4">
              <p className="text-[14px] text-[var(--text-primary)] leading-relaxed">
                {tx.memo}
              </p>
            </div>
          </>
        )}

        {/* View on Explorer */}
        <div className="mt-8">
          <Button variant="outline" onClick={() => window.open('https://basescan.org', '_blank')}>
            <ExternalLink size={18} className="mr-2" />
            View on Explorer
          </Button>
        </div>
      </div>
    </div>
  )
}
