import { test, expect, testAgents } from "./fixtures";

test.describe("Agent management", () => {
  test("renders agent list with mock data", async ({ page, mockTauri }) => {
    await mockTauri({ list_agents: testAgents });
    await page.goto("/agents");

    await expect(page.getByRole("heading", { name: "Agents" })).toBeVisible();
    await expect(page.getByText("Payment Bot")).toBeVisible();
    await expect(page.getByText("Trading Bot")).toBeVisible();
    await expect(page.locator("[data-testid='status-badge-agent-1']")).toContainText("active");
    await expect(page.locator("[data-testid='status-badge-agent-2']")).toContainText("suspended");
  });

  test("shows empty state when no agents", async ({ page, mockTauri }) => {
    await mockTauri({ list_agents: [] });
    await page.goto("/agents");

    await expect(page.getByText("No agents registered")).toBeVisible();
  });

  test("clicking an agent navigates to detail page", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_agents: testAgents,
      get_agent: testAgents[0],
      get_agent_spending_policy: {
        agent_id: "agent-1",
        per_tx_max: "100",
        daily_cap: "500",
        weekly_cap: "2500",
        monthly_cap: "8000",
        auto_approve_max: "50",
        allowlist: [],
        updated_at: 1700000000,
      },
      get_agent_transactions: [],
    });
    await page.goto("/agents");

    await page.getByText("Payment Bot").click();
    await page.waitForURL("**/agents/agent-1");

    await expect(page.getByText("Agent Profile")).toBeVisible();
  });
});
