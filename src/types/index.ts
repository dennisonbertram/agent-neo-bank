export interface Agent {
  id: string;
  name: string;
  description: string;
  status: "pending" | "active" | "suspended" | "revoked";
  created_at: number;
  updated_at: number;
}

export interface Transaction {
  id: string;
  agent_id: string | null;
  tx_type: "send" | "receive" | "earn";
  amount: string;
  asset: string;
  status: "pending" | "approved" | "executing" | "confirmed" | "failed" | "denied";
  created_at: number;
  updated_at: number;
}

export interface SpendingPolicy {
  agent_id: string;
  per_tx_max: string;
  daily_cap: string;
  weekly_cap: string;
  monthly_cap: string;
  auto_approve_max: string;
}

export interface BalanceResponse {
  balance: string;
  asset: string;
}

export interface AuthStatusResponse {
  authenticated: boolean;
  email: string | null;
}
