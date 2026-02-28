import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Settings, Search, Rocket, Landmark, Database } from 'lucide-react'
import { SegmentControl } from '../components/ui/SegmentControl'
import { AgentCard } from '../components/agent/AgentCard'
import { BottomNav } from '../components/layout/BottomNav'
import placeholderData from '../data/placeholder_data.json'
import type { AgentStatus } from '../types'

const iconMap: Record<string, typeof Search> = {
  Search, Rocket, Landmark, Database,
}

const agentDisplayData = placeholderData.agents.samples.map((agent, i) => ({
  ...agent,
  icon: ['Search', 'Rocket', 'Landmark'][i] ?? 'Search',
  budget: placeholderData.budgetSummaries.samples[i],
}))

export default function AgentsList() {
  const [segment, setSegment] = useState('All Agents')
  const navigate = useNavigate()

  const filteredAgents = agentDisplayData.filter((agent) => {
    if (segment === 'Active') return agent.status === 'active'
    if (segment === 'Archived') return agent.status === 'revoked'
    return true
  })

  return (
    <div className="flex flex-col h-full relative">
      {/* Header */}
      <div className="pt-[60px] px-6 pb-4 flex justify-between items-center">
        <h1 className="text-[24px] font-bold tracking-[-0.5px] text-[var(--text-primary)]">
          My Agents
        </h1>
        <button
          type="button"
          onClick={() => navigate('/settings')}
          className="w-[40px] h-[40px] rounded-full bg-[var(--bg-secondary)] border border-[var(--surface-hover)] flex items-center justify-center cursor-pointer"
        >
          <Settings size={20} />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 pb-[100px] scrollbar-hide">
        <SegmentControl
          options={['Active', 'All Agents', 'Archived']}
          value={segment}
          onChange={setSegment}
          className="mb-6"
        />

        {filteredAgents.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-title mb-2">No agents found</p>
            <p className="text-body">
              {segment === 'Archived'
                ? 'No archived agents yet.'
                : 'Connect your first agent to get started.'}
            </p>
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            {filteredAgents.map((agent) => {
              const Icon = iconMap[agent.icon] || Search
              return (
                <AgentCard
                  key={agent.id}
                  id={agent.id ?? ''}
                  name={agent.name ?? 'Unknown'}
                  description={agent.description ?? ''}
                  status={(agent.status ?? 'pending') as AgentStatus}
                  icon={Icon}
                  accentColor={agent.accentColor ?? '#8FB5AA'}
                  dailySpent={agent.budget?.daily_spent || '0'}
                  dailyCap={agent.budget?.daily_cap || '0'}
                />
              )
            })}
          </div>
        )}
      </div>

      {/* Bottom Nav */}
      <BottomNav activeTab="agents" />
    </div>
  )
}
