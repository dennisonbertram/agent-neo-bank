import { test, expect, testApprovals, testAgents } from "./fixtures";

test.describe("Approval queue", () => {
  test("renders approval queue with pending items", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_approvals: testApprovals,
      list_agents: testAgents,
    });
    await page.goto("/approvals");

    await expect(page.getByRole("heading", { name: "Approvals" })).toBeVisible();
    // The approval shows the agent name (resolved from agent-1 -> Payment Bot)
    await expect(page.getByText("Payment Bot")).toBeVisible();
    // Action buttons for pending approval
    await expect(page.getByRole("button", { name: "Approve" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Deny" })).toBeVisible();
  });

  test("shows empty state when no approvals", async ({ page, mockTauri }) => {
    await mockTauri({
      list_approvals: [],
      list_agents: [],
    });
    await page.goto("/approvals");

    await expect(page.getByText("No pending approvals")).toBeVisible();
  });

  test("filter toggle between Pending and All", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_approvals: testApprovals,
      list_agents: testAgents,
    });
    await page.goto("/approvals");

    // Both filter buttons should be visible
    const pendingBtn = page.getByRole("button", { name: "Pending", exact: true });
    const allBtn = page.getByRole("button", { name: "All" });

    await expect(pendingBtn).toBeVisible();
    await expect(allBtn).toBeVisible();

    // Click All filter
    await allBtn.click();
    // The page re-renders with the new filter
    await expect(page.getByRole("heading", { name: "Approvals" })).toBeVisible();
  });

  test("approve button triggers resolve_approval with correct args", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_approvals: testApprovals,
      list_agents: testAgents,
      resolve_approval: null,
    });
    await page.goto("/approvals");

    await page.getByRole("button", { name: "Approve" }).click();
    // After resolving, the page reloads approvals
    await expect(page.getByRole("heading", { name: "Approvals" })).toBeVisible();

    // Verify resolve_approval was called with the correct approval ID and decision
    const calls = await page.evaluate(() => (window as any).__TAURI_INVOKE_CALLS__);
    const resolveCall = calls.find(
      (c: { cmd: string; args: unknown }) => c.cmd === "resolve_approval"
    );
    expect(resolveCall).toBeTruthy();
    expect(resolveCall.args).toMatchObject({
      approvalId: "approval-1",
      decision: "approved",
    });
  });

  test("deny button triggers resolve_approval with denied decision", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      list_approvals: testApprovals,
      list_agents: testAgents,
      resolve_approval: null,
    });
    await page.goto("/approvals");

    await page.getByRole("button", { name: "Deny" }).click();
    await expect(page.getByRole("heading", { name: "Approvals" })).toBeVisible();

    const calls = await page.evaluate(() => (window as any).__TAURI_INVOKE_CALLS__);
    const resolveCall = calls.find(
      (c: { cmd: string; args: unknown }) => c.cmd === "resolve_approval"
    );
    expect(resolveCall).toBeTruthy();
    expect(resolveCall.args).toMatchObject({
      approvalId: "approval-1",
      decision: "denied",
    });
  });
});
