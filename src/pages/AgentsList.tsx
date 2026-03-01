import { useState, useEffect } from 'react'
import { Search, Rocket, Landmark, Database } from 'lucide-react'
import { TopBar } from '../components/layout/TopBar'
import { SegmentControl } from '../components/ui/SegmentControl'
import { AgentCard } from '../components/agent/AgentCard'
import { safeTauriCall, tauriApi, placeholderData } from '../lib/tauri'
import type { Agent, AgentBudgetSummary, AgentStatus } from '../types'

const iconMap: Record<string, typeof Search> = {
  Search, Rocket, Landmark, Database,
}

/** Maps agent_type to a display icon name */
const agentTypeIconMap: Record<string, string> = {
  research: 'Search',
  deployment: 'Rocket',
  treasury: 'Landmark',
}

/** Maps agent_type to a default accent color */
const agentTypeColorMap: Record<string, string> = {
  research: '#8FB5AA',
  deployment: '#F2D48C',
  treasury: '#D9A58B',
}

interface AgentDisplayData {
  id: string
  name: string
  description: string
  status: AgentStatus
  agent_type: string
  icon: string
  accentColor: string
  budget?: AgentBudgetSummary
}

function mergeAgentData(agents: Agent[], budgets: AgentBudgetSummary[]): AgentDisplayData[] {
  const budgetMap = new Map(budgets.map((b) => [b.agent_id, b]))
  return agents.map((agent) => ({
    id: agent.id,
    name: agent.name,
    description: agent.description,
    status: agent.status,
    agent_type: agent.agent_type,
    icon: agentTypeIconMap[agent.agent_type] ?? 'Search',
    accentColor: agentTypeColorMap[agent.agent_type] ?? '#8FB5AA',
    budget: budgetMap.get(agent.id),
  }))
}

export default function AgentsList() {
  const [segment, setSegment] = useState('All Agents')
  const [agentDisplayData, setAgentDisplayData] = useState<AgentDisplayData[]>([])
  const [loading, setLoading] = useState(true)
  useEffect(() => {
    async function loadData() {
      const [agents, budgets] = await Promise.all([
        safeTauriCall(
          () => tauriApi.agents.list(),
          placeholderData.agents.samples as unknown as Agent[],
        ),
        safeTauriCall(
          () => tauriApi.budget.getAgentSummaries(),
          placeholderData.budgetSummaries.samples as AgentBudgetSummary[],
        ),
      ])
      setAgentDisplayData(mergeAgentData(agents, budgets))
      setLoading(false)
    }
    loadData()
  }, [])

  const filteredAgents = agentDisplayData.filter((agent) => {
    if (segment === 'Active') return agent.status === 'active'
    if (segment === 'Archived') return agent.status === 'revoked'
    return true
  })

  return (
    <div className="flex flex-col h-full relative">
      <TopBar />

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 pb-6 scrollbar-hide">
        <SegmentControl
          options={['Active', 'All Agents', 'Archived']}
          value={segment}
          onChange={setSegment}
          className="mb-6 mt-4"
        />

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="w-6 h-6 border-2 border-[var(--text-tertiary)] border-t-[var(--text-primary)] rounded-full animate-spin" />
          </div>
        ) : filteredAgents.length === 0 ? (
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
    </div>
  )
}
