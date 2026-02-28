import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Plus, Users, Search, Rocket, Landmark, ArrowDownLeft, Code } from 'lucide-react'
import { TopBar } from '../components/layout/TopBar'
import { BottomNav } from '../components/layout/BottomNav'
import { Button } from '../components/ui/Button'
import { SegmentControl } from '../components/ui/SegmentControl'
import { AgentPillRow } from '../components/agent/AgentPillRow'
import { TransactionItem } from '../components/transaction/TransactionItem'
import placeholderData from '../data/placeholder_data.json'

export default function Home() {
  const navigate = useNavigate()
  const [segment, setSegment] = useState('Overview')
  const wallet = placeholderData.wallet
  const user = placeholderData.user
  const transactions = placeholderData.transactions.samples

  return (
    <div className="flex flex-col h-full relative">
      {/* Scrollable content */}
      <div className="flex-1 overflow-y-auto scrollbar-hide pb-[100px]">
        {/* Top bar with gradient fade */}
        <div className="sticky top-0 z-10 pt-[60px] pb-4 bg-gradient-to-b from-white from-80% to-transparent">
          <TopBar walletName={placeholderData.app.walletName} initials={user.initials} />
        </div>

        <div className="px-6">
          {/* Balance Card */}
          <div className="bg-black text-white rounded-[32px] p-8 mb-6 relative overflow-hidden">
            {/* Decorative glow */}
            <div className="absolute -top-1/2 -right-[20%] w-[200px] h-[200px] bg-[radial-gradient(circle,rgba(143,181,170,0.2)_0%,transparent_70%)] pointer-events-none" />

            <div className="flex justify-between items-start mb-6 relative z-[1]">
              <div>
                <p className="text-[11px] text-white/60">Base Network Balance</p>
                <p className="text-[40px] font-semibold mt-1 leading-none">{wallet.totalBalanceUsd}</p>
              </div>
              <div className="bg-white/10 px-2 py-1 rounded-[8px] flex items-center gap-1.5">
                <div className="w-2 h-2 rounded-full bg-[#0052FF]" />
                <span className="text-[11px] font-semibold tracking-[0.5px]">BASE</span>
              </div>
            </div>

            <div className="flex justify-between items-center relative z-[1]">
              <span className="font-mono text-[13px] text-white/50">
                {wallet.address.slice(0, 6)}...{wallet.address.slice(-4)}
              </span>
              <div className="flex gap-3">
                <span className="text-[12px] font-medium">{wallet.balances.ETH.formatted} ETH</span>
                <span className="text-[12px] font-medium">{wallet.balances.USDC.formatted} USDC</span>
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

          {segment === 'Agents' ? (
            /* Agent Pills */
            <div className="flex flex-col gap-3 mb-8">
              <AgentPillRow
                icon={Search}
                label="Research"
                value="$3.50"
                subValue="today"
                accentColor="#8FB5AA"
              />
              <AgentPillRow
                icon={Rocket}
                label="Deploy Bot"
                value="$0.01"
                subValue="today"
                accentColor="#F2D48C"
              />
              <AgentPillRow
                icon={Landmark}
                label="Treasury"
                value="$0.00"
                subValue="paused"
                accentColor="#D9A58B"
              />
            </div>
          ) : (
            /* Activity Feed */
            <div className="mb-8">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-title" style={{ fontSize: 20 }}>Activity</h3>
                <button
                  type="button"
                  className="text-[13px] font-semibold text-[#0052FF] bg-transparent border-none cursor-pointer"
                >
                  View All
                </button>
              </div>

              {transactions.map((tx, i) => (
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
                  isLast={i === transactions.length - 1}
                  onClick={() => navigate(`/transactions/${tx.id}`)}
                />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Bottom Nav */}
      <BottomNav activeTab="home" />
    </div>
  )
}
