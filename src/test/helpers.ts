import type { Agent, Transaction, SpendingPolicy } from "../types";

export function createMockAgent(overrides: Partial<Agent> = {}): Agent {
  return {
    id: "test-agent-id",
    name: "Test Agent",
    description: "A test agent",
    status: "active",
    created_at: Date.now(),
    updated_at: Date.now(),
    ...overrides,
  };
}

export function createMockTransaction(
  overrides: Partial<Transaction> = {}
): Transaction {
  return {
    id: "test-tx-id",
    agent_id: "test-agent-id",
    tx_type: "send",
    amount: "10.00",
    asset: "USDC",
    status: "pending",
    created_at: Date.now(),
    updated_at: Date.now(),
    ...overrides,
  };
}

export function createMockSpendingPolicy(
  overrides: Partial<SpendingPolicy> = {}
): SpendingPolicy {
  return {
    agent_id: "test-agent-id",
    per_tx_max: "100",
    daily_cap: "500",
    weekly_cap: "2000",
    monthly_cap: "5000",
    auto_approve_max: "10",
    ...overrides,
  };
}
