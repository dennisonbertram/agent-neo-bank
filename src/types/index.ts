// TypeScript types matching Rust models in src-tauri/src/db/models.rs

export type AgentStatus = "pending" | "active" | "suspended" | "revoked";
export type TxStatus = "pending" | "approved" | "awaiting_approval" | "executing" | "confirmed" | "failed" | "denied";
export type TxType = "send" | "receive" | "earn";
export type ApprovalRequestType = "transaction" | "limit_increase" | "registration";
export type ApprovalStatus = "pending" | "approved" | "denied" | "expired";

export interface Agent {
  id: string;
  name: string;
  description: string;
  purpose: string;
  agent_type: string;
  capabilities: string[];
  status: AgentStatus;
  api_token_hash: string | null;
  token_prefix: string | null;
  balance_visible: boolean;
  invitation_code: string | null;
  created_at: number;
  updated_at: number;
  last_active_at: number | null;
  metadata: string;
}

export interface SpendingPolicy {
  agent_id: string;
  per_tx_max: string;
  daily_cap: string;
  weekly_cap: string;
  monthly_cap: string;
  auto_approve_max: string;
  allowlist: string[];
  updated_at: number;
}

export interface GlobalPolicy {
  id: string;
  daily_cap: string;
  weekly_cap: string;
  monthly_cap: string;
  min_reserve_balance: string;
  kill_switch_active: boolean;
  kill_switch_reason: string;
  updated_at: number;
}

export interface Transaction {
  id: string;
  agent_id: string | null;
  tx_type: TxType;
  amount: string;
  asset: string;
  recipient: string | null;
  sender: string | null;
  chain_tx_hash: string | null;
  status: TxStatus;
  category: string;
  memo: string;
  description: string;
  service_name: string;
  service_url: string;
  reason: string;
  webhook_url: string | null;
  error_message: string | null;
  period_daily: string;
  period_weekly: string;
  period_monthly: string;
  created_at: number;
  updated_at: number;
}

export interface ApprovalRequest {
  id: string;
  agent_id: string;
  request_type: ApprovalRequestType;
  payload: string;
  status: ApprovalStatus;
  tx_id: string | null;
  expires_at: number;
  created_at: number;
  resolved_at: number | null;
  resolved_by: string | null;
}

export interface InvitationCode {
  code: string;
  created_at: number;
  expires_at: number | null;
  used_by: string | null;
  used_at: number | null;
  max_uses: number;
  use_count: number;
  label: string;
}

export interface TokenDelivery {
  agent_id: string;
  encrypted_token: string;
  created_at: number;
  expires_at: number;
  delivered: boolean;
}

export interface NotificationPreferences {
  id: string;
  enabled: boolean;
  on_all_tx: boolean;
  on_large_tx: boolean;
  large_tx_threshold: string;
  on_errors: boolean;
  on_limit_requests: boolean;
  on_agent_registration: boolean;
}

export interface SpendingLedger {
  agent_id: string;
  period: string;
  total: string;
  tx_count: number;
  updated_at: number;
}

export interface AgentBudgetSummary {
  agent_id: string;
  agent_name: string;
  daily_spent: string;
  daily_cap: string;
  weekly_spent: string;
  weekly_cap: string;
  monthly_spent: string;
  monthly_cap: string;
}

export interface GlobalBudgetSummary {
  daily_spent: string;
  daily_cap: string;
  weekly_spent: string;
  weekly_cap: string;
  monthly_spent: string;
  monthly_cap: string;
  kill_switch_active: boolean;
}

export interface BalanceResponse {
  balance: string;
  asset: string;
}

export interface AddressResponse {
  address: string;
}

export interface AuthStatusResponse {
  authenticated: boolean;
  email?: string;
}
