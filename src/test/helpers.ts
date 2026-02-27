import { vi } from "vitest";
import type { Agent, Transaction, SpendingPolicy } from "../types";

export async function mockInvoke(responses: Record<string, unknown>) {
  const mod = await import("@tauri-apps/api/core");
  const mocked = vi.mocked(mod);
  mocked.invoke.mockImplementation(async (cmd: string) => {
    if (cmd in responses) {
      return responses[cmd];
    }
    throw new Error(`Unmocked invoke: ${cmd}`);
  });
  return mocked.invoke;
}

export function createMockAgent(overrides: Partial<Agent> = {}): Agent {
  return {
    id: "test-agent-id",
    name: "Test Agent",
    description: "A test agent",
    purpose: "Testing",
    agent_type: "test",
    capabilities: ["send"],
    status: "active",
    api_token_hash: null,
    token_prefix: null,
    balance_visible: true,
    invitation_code: "INV-test",
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    last_active_at: null,
    metadata: "{}",
    ...overrides,
  };
}

export function createMockTransaction(overrides: Partial<Transaction> = {}): Transaction {
  return {
    id: "test-tx-id",
    agent_id: "test-agent-id",
    tx_type: "send",
    amount: "10.00",
    asset: "USDC",
    recipient: "0xTestRecipient",
    sender: null,
    chain_tx_hash: null,
    status: "pending",
    category: "test",
    memo: "Test transaction",
    description: "Test transaction",
    service_name: "Test Service",
    service_url: "https://test.example.com",
    reason: "Testing",
    webhook_url: null,
    error_message: null,
    period_daily: "daily:2026-02-27",
    period_weekly: "weekly:2026-W09",
    period_monthly: "monthly:2026-02",
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    ...overrides,
  };
}

export function createMockSpendingPolicy(overrides: Partial<SpendingPolicy> = {}): SpendingPolicy {
  return {
    agent_id: "test-agent-id",
    per_tx_max: "100",
    daily_cap: "1000",
    weekly_cap: "5000",
    monthly_cap: "20000",
    auto_approve_max: "10",
    allowlist: [],
    updated_at: Date.now() / 1000,
    ...overrides,
  };
}
