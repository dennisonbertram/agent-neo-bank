import { test as base, type Page } from "@playwright/test";

/**
 * Mock responses for Tauri invoke commands.
 * Each key is a Tauri command name, value is the response to return.
 */
export type InvokeMocks = Record<string, unknown>;

/**
 * Injects a mock for `window.__TAURI_INTERNALS__.invoke` so that
 * the React app's calls to `invoke(cmd, args)` resolve with test data
 * instead of hitting the real Tauri backend.
 */
async function injectTauriMock(page: Page, mocks: InvokeMocks) {
  await page.addInitScript((mockData) => {
    // Track all invoke calls for assertion in tests
    const invokeCalls: Array<{ cmd: string; args: unknown }> = [];
    (window as any).__TAURI_INVOKE_CALLS__ = invokeCalls;

    // Tauri v2 invoke goes through __TAURI_INTERNALS__
    (window as any).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: unknown) => {
        invokeCalls.push({ cmd, args });
        if (cmd in mockData) {
          const value = mockData[cmd];
          // Support error simulation: if value is { __mock_error__: "msg" }, reject
          if (
            value &&
            typeof value === "object" &&
            "__mock_error__" in (value as Record<string, unknown>)
          ) {
            return Promise.reject(
              (value as Record<string, unknown>).__mock_error__
            );
          }
          return Promise.resolve(value);
        }
        // Return a sensible empty fallback so pages don't crash
        return Promise.resolve(null);
      },
      convertFileSrc: (src: string) => src,
      transformCallback: () => 0,
      metadata: { currentWebview: { label: "main" }, currentWindow: { label: "main" } },
    };
    // Also provide the event listener no-op
    (window as any).__TAURI_INTERNALS__.invoke.__isMock = true;
  }, mocks);
}

// Extend the base test with a helper that makes mocking easy
export const test = base.extend<{ mockTauri: (mocks: InvokeMocks) => Promise<void> }>({
  mockTauri: async ({ page }, use) => {
    const mock = async (mocks: InvokeMocks) => {
      await injectTauriMock(page, mocks);
    };
    await use(mock);
  },
});

export { expect } from "@playwright/test";

// ---- Shared test data factories ----

export const testAgents = [
  {
    id: "agent-1",
    name: "Payment Bot",
    description: "Handles payments",
    purpose: "Automate recurring payments",
    agent_type: "payment",
    capabilities: ["send"],
    status: "active",
    api_token_hash: null,
    token_prefix: null,
    balance_visible: true,
    invitation_code: null,
    created_at: 1700000000,
    updated_at: 1700000000,
    last_active_at: 1700001000,
    metadata: "{}",
  },
  {
    id: "agent-2",
    name: "Trading Bot",
    description: "Handles trades",
    purpose: "Execute DCA strategy",
    agent_type: "trading",
    capabilities: ["send", "receive"],
    status: "suspended",
    api_token_hash: null,
    token_prefix: null,
    balance_visible: false,
    invitation_code: null,
    created_at: 1700000000,
    updated_at: 1700000000,
    last_active_at: null,
    metadata: "{}",
  },
];

export const testApprovals = [
  {
    id: "approval-1",
    agent_id: "agent-1",
    request_type: "transaction",
    payload: JSON.stringify({ to: "0xabc", amount: "100", asset: "USDC" }),
    status: "pending",
    tx_id: "tx-1",
    expires_at: Math.floor(Date.now() / 1000) + 86400,
    created_at: Math.floor(Date.now() / 1000) - 3600,
    resolved_at: null,
    resolved_by: null,
  },
];

export const testTransactions = {
  transactions: [
    {
      id: "tx-1",
      agent_id: "agent-1",
      tx_type: "send",
      amount: "50.00",
      asset: "USDC",
      recipient: "0xabcdef1234567890abcdef1234567890abcdef12",
      sender: null,
      chain_tx_hash: null,
      status: "confirmed",
      category: "payment",
      memo: "",
      description: "Monthly subscription",
      service_name: "",
      service_url: "",
      reason: "",
      webhook_url: null,
      error_message: null,
      period_daily: "2024-01-01",
      period_weekly: "2024-W01",
      period_monthly: "2024-01",
      created_at: 1700000000,
      updated_at: 1700000000,
    },
    {
      id: "tx-2",
      agent_id: "agent-2",
      tx_type: "send",
      amount: "200.00",
      asset: "USDC",
      recipient: "0x1234567890abcdef1234567890abcdef12345678",
      sender: null,
      chain_tx_hash: null,
      status: "pending",
      category: "trade",
      memo: "",
      description: "DCA purchase",
      service_name: "",
      service_url: "",
      reason: "",
      webhook_url: null,
      error_message: null,
      period_daily: "2024-01-01",
      period_weekly: "2024-W01",
      period_monthly: "2024-01",
      created_at: 1700000500,
      updated_at: 1700000500,
    },
  ],
  total: 2,
};

export const testGlobalPolicy = {
  id: "global-1",
  daily_cap: "1000",
  weekly_cap: "5000",
  monthly_cap: "15000",
  min_reserve_balance: "100",
  kill_switch_active: false,
  kill_switch_reason: "",
  updated_at: 1700000000,
};

export const testGlobalBudgetSummary = {
  daily_spent: "250",
  daily_cap: "1000",
  weekly_spent: "1200",
  weekly_cap: "5000",
  monthly_spent: "3500",
  monthly_cap: "15000",
  kill_switch_active: false,
};

export const testAgentBudgetSummaries = [
  {
    agent_id: "agent-1",
    agent_name: "Payment Bot",
    daily_spent: "150",
    daily_cap: "500",
    weekly_spent: "800",
    weekly_cap: "2500",
    monthly_spent: "2000",
    monthly_cap: "8000",
  },
];
