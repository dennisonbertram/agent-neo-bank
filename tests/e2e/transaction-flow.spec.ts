import { test, expect, testTransactions, testAgents } from "./fixtures";

test.describe("Transaction flow", () => {
  test("renders transaction list with mock data", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_transactions: testTransactions,
      list_agents: testAgents,
    });
    await page.goto("/transactions");

    await expect(page.getByRole("heading", { name: "Transaction History" })).toBeVisible();
    await expect(page.getByText("Monthly subscription")).toBeVisible();
    await expect(page.getByText("DCA purchase")).toBeVisible();
  });

  test("shows empty state when no transactions", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_transactions: { transactions: [], total: 0 },
      list_agents: [],
    });
    await page.goto("/transactions");

    await expect(page.getByText("No transactions yet")).toBeVisible();
  });

  test("status badges render correctly", async ({ page, mockTauri }) => {
    await mockTauri({
      list_transactions: testTransactions,
      list_agents: testAgents,
    });
    await page.goto("/transactions");

    // Check status badges by test id
    await expect(page.locator("[data-testid='status-badge-tx-1']")).toContainText("Confirmed");
    await expect(page.locator("[data-testid='status-badge-tx-2']")).toContainText("Pending");
  });

  test("status filter dropdown is present", async ({ page, mockTauri }) => {
    await mockTauri({
      list_transactions: testTransactions,
      list_agents: testAgents,
    });
    await page.goto("/transactions");

    const statusFilter = page.locator("[data-testid='status-filter']");
    await expect(statusFilter).toBeVisible();

    // Verify filter options
    await expect(statusFilter.locator("option")).toHaveCount(7);
  });

  test("agent filter renders when agents exist", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_transactions: testTransactions,
      list_agents: testAgents,
    });
    await page.goto("/transactions");

    const agentFilter = page.locator("[data-testid='agent-filter']");
    await expect(agentFilter).toBeVisible();
  });
});
