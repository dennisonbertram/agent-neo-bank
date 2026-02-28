import { invoke } from '@tauri-apps/api/core'
import placeholderData from '../data/placeholder_data.json'

/** Returns true when running inside Tauri (not plain browser) */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
}

/**
 * Safely call a Tauri invoke. In browser mode (no Tauri runtime),
 * returns the provided fallback instead of throwing.
 * Includes a timeout to prevent hanging when CLI commands stall.
 */
export async function safeTauriCall<T>(fn: () => Promise<T>, fallback: T, timeoutMs = 10000): Promise<T> {
  if (!isTauri()) return fallback
  try {
    const timeout = new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error('Tauri call timed out')), timeoutMs)
    )
    return await Promise.race([fn(), timeout])
  } catch {
    return fallback
  }
}

/** Placeholder data re-exported for browser fallback */
export { placeholderData }
import type {
  Agent,
  SpendingPolicy,
  GlobalPolicy,
  Transaction,
  ApprovalRequest,
  InvitationCode,
  NotificationPreferences,
  AgentBudgetSummary,
  GlobalBudgetSummary,
  BalanceResponse,
  AddressResponse,
  AuthStatusResponse,
} from '../types'

interface ListTransactionsResponse {
  transactions: Transaction[]
  total: number
}

export const tauriApi = {
  auth: {
    login: (email: string) =>
      invoke<{ status: string; flow_id?: string }>('auth_login', { email }),
    verify: (otp: string) =>
      invoke<{ status: string }>('auth_verify', { otp }),
    status: () =>
      invoke<AuthStatusResponse>('auth_status'),
    logout: () =>
      invoke<void>('auth_logout'),
  },
  wallet: {
    getBalance: () =>
      invoke<BalanceResponse>('get_balance'),
    getAddress: () =>
      invoke<AddressResponse>('get_address'),
  },
  agents: {
    list: () =>
      invoke<Agent[]>('list_agents'),
    get: (agentId: string) =>
      invoke<Agent>('get_agent', { agentId }),
    getPolicy: (agentId: string) =>
      invoke<SpendingPolicy>('get_agent_spending_policy', { agentId }),
    updatePolicy: (policy: SpendingPolicy) =>
      invoke<void>('update_agent_spending_policy', { policy }),
    suspend: (agentId: string) =>
      invoke<void>('suspend_agent', { agentId }),
    revoke: (agentId: string) =>
      invoke<void>('revoke_agent', { agentId }),
    getTransactions: (agentId: string, limit?: number) =>
      invoke<Transaction[]>('get_agent_transactions', { agentId, limit }),
  },
  transactions: {
    list: (params: { limit: number; offset: number; agentId?: string; status?: string }) =>
      invoke<ListTransactionsResponse>('list_transactions', {
        limit: params.limit,
        offset: params.offset,
        agent_id: params.agentId ?? null,
        status: params.status ?? null,
      }),
    get: (txId: string) =>
      invoke<Transaction>('get_transaction', { txId }),
  },
  approvals: {
    list: (status?: string) =>
      invoke<ApprovalRequest[]>('list_approvals', { status: status ?? null }),
    get: (approvalId: string) =>
      invoke<ApprovalRequest>('get_approval', { approvalId }),
    resolve: (approvalId: string, decision: string) =>
      invoke<ApprovalRequest>('resolve_approval', { approvalId, decision }),
  },
  invitations: {
    list: () =>
      invoke<InvitationCode[]>('list_invitation_codes'),
    generate: (label: string, expiresAt?: number, maxUses?: number) =>
      invoke<InvitationCode>('generate_invitation_code', {
        label,
        expires_at: expiresAt ?? null,
        max_uses: maxUses ?? null,
      }),
    revoke: (code: string) =>
      invoke<void>('revoke_invitation_code', { code }),
  },
  notifications: {
    getPreferences: () =>
      invoke<NotificationPreferences>('get_notification_preferences'),
    updatePreferences: (prefs: NotificationPreferences) =>
      invoke<void>('update_notification_preferences', { prefs }),
  },
  budget: {
    getAgentSummaries: () =>
      invoke<AgentBudgetSummary[]>('get_agent_budget_summaries'),
    getGlobalSummary: () =>
      invoke<GlobalBudgetSummary>('get_global_budget_summary'),
  },
  settings: {
    getGlobalPolicy: () =>
      invoke<GlobalPolicy>('get_global_policy'),
    updateGlobalPolicy: (policy: GlobalPolicy) =>
      invoke<void>('update_global_policy', { policy }),
    toggleKillSwitch: (active: boolean, reason?: string) =>
      invoke<void>('toggle_kill_switch', { active, reason: reason ?? null }),
  },
}
