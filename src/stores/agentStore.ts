import { create } from 'zustand'
import { tauriApi } from '../lib/tauri'
import type { Agent, AgentBudgetSummary, SpendingPolicy } from '../types'

interface AgentState {
  agents: Agent[]
  budgetSummaries: AgentBudgetSummary[]
  isLoading: boolean
  lastFetched: number | null

  fetchAgents: () => Promise<void>
  suspendAgent: (agentId: string) => Promise<void>
  revokeAgent: (agentId: string) => Promise<void>
  updatePolicy: (policy: SpendingPolicy) => Promise<void>
}

export const useAgentStore = create<AgentState>((set) => ({
  agents: [],
  budgetSummaries: [],
  isLoading: false,
  lastFetched: null,

  fetchAgents: async () => {
    set({ isLoading: true })
    try {
      const [agents, budgetSummaries] = await Promise.all([
        tauriApi.agents.list(),
        tauriApi.budget.getAgentSummaries(),
      ])
      set({
        agents,
        budgetSummaries,
        isLoading: false,
        lastFetched: Date.now(),
      })
    } catch {
      set({ isLoading: false })
    }
  },

  suspendAgent: async (agentId) => {
    await tauriApi.agents.suspend(agentId)
    set((state) => ({
      agents: state.agents.map((a) =>
        a.id === agentId ? { ...a, status: 'suspended' as const } : a
      ),
    }))
  },

  revokeAgent: async (agentId) => {
    await tauriApi.agents.revoke(agentId)
    set((state) => ({
      agents: state.agents.map((a) =>
        a.id === agentId ? { ...a, status: 'revoked' as const } : a
      ),
    }))
  },

  updatePolicy: async (policy) => {
    await tauriApi.agents.updatePolicy(policy)
  },
}))
