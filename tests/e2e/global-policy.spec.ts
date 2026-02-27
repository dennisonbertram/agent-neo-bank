import {
  test,
  expect,
  testGlobalPolicy,
  testGlobalBudgetSummary,
  testAgentBudgetSummaries,
} from "./fixtures";

const settingsMocks = {
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
};

test.describe("Global policy", () => {
  test("renders global policy section in settings", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_global_policy: testGlobalPolicy,
      ...settingsMocks,
    });
    await page.goto("/settings");

    await expect(page.locator("[data-slot='card-title']").filter({ hasText: "Global Policy" })).toBeVisible();
    await expect(page.getByText("Spending Caps")).toBeVisible();
    await expect(page.getByLabel("Daily Cap (USDC)")).toBeVisible();
    await expect(page.getByLabel("Weekly Cap (USDC)")).toBeVisible();
    await expect(page.getByLabel("Monthly Cap (USDC)")).toBeVisible();
  });

  test("spending cap inputs have correct values from mock", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_global_policy: testGlobalPolicy,
      ...settingsMocks,
    });
    await page.goto("/settings");

    await expect(page.getByLabel("Daily Cap (USDC)")).toHaveValue("1000");
    await expect(page.getByLabel("Weekly Cap (USDC)")).toHaveValue("5000");
    await expect(page.getByLabel("Monthly Cap (USDC)")).toHaveValue("15000");
  });

  test("Save Caps button is present and clickable", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_global_policy: testGlobalPolicy,
      update_global_policy: null,
      ...settingsMocks,
    });
    await page.goto("/settings");

    const saveBtn = page.getByRole("button", { name: "Save Caps" });
    await expect(saveBtn).toBeVisible();
    await saveBtn.click();
    // After save, should still be on settings page
    await expect(page.locator("[data-slot='card-title']").filter({ hasText: "Global Policy" })).toBeVisible();
  });

  test("dashboard shows global budget utilization", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_balance: { balance: "10000.00", asset: "USDC" },
      get_agent_budget_summaries: testAgentBudgetSummaries,
      get_global_budget_summary: testGlobalBudgetSummary,
    });
    await page.goto("/");

    await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
    await expect(page.getByText("Global Budget Utilization")).toBeVisible();
  });
});
