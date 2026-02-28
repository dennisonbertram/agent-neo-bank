import { test, expect } from "./fixtures";

/**
 * Auth guard tests.
 *
 * The current app does not have web-layer route guards — authentication is
 * managed on the Tauri side. The Shell layout renders for all routes under it
 * regardless of auth state. These tests verify that the app renders
 * reasonable content even when the auth store has isAuthenticated=false
 * (the default state).
 */
test.describe("Auth and routing guards", () => {
  test("navigating to /agents without auth still renders the page", async ({
    page,
    mockTauri,
  }) => {
    // No auth setup — isAuthenticated defaults to false in the zustand store
    await mockTauri({ list_agents: [] });
    await page.goto("/agents");

    // The app currently renders the agents page regardless of auth state
    await expect(page.getByRole("heading", { name: "Agents" })).toBeVisible();
  });

  test("navigating to /settings without auth still renders the page", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_global_policy: {
        id: "global-1",
        daily_cap: "1000",
        weekly_cap: "5000",
        monthly_cap: "15000",
        min_reserve_balance: "100",
        kill_switch_active: false,
        kill_switch_reason: "",
        updated_at: 1700000000,
      },
      get_notification_preferences: {
        id: "notif-1",
        enabled: true,
        on_all_tx: false,
        on_large_tx: true,
        large_tx_threshold: "1000",
        on_errors: true,
        on_limit_requests: true,
        on_agent_registration: true,
      },
      list_invitation_codes: [],
    });
    await page.goto("/settings");

    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  });

  test("navigating to /onboarding renders the welcome screen", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({});
    await page.goto("/onboarding");

    await expect(
      page.getByRole("heading", { name: "Welcome to Tally Agentic Wallet" })
    ).toBeVisible();
  });

  test("navigating to / renders the dashboard", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_balance: { balance: "0.00", asset: "USDC" },
      get_global_budget_summary: {
        daily_spent: "0",
        daily_cap: "1000",
        weekly_spent: "0",
        weekly_cap: "5000",
        monthly_spent: "0",
        monthly_cap: "15000",
        kill_switch_active: false,
      },
      get_agent_budget_summaries: [],
    });
    await page.goto("/");

    await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  });
});
