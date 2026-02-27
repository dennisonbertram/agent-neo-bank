import { test, expect, testGlobalPolicy } from "./fixtures";

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

test.describe("Kill switch", () => {
  test("renders settings page with kill switch section", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_global_policy: testGlobalPolicy,
      ...settingsMocks,
    });
    await page.goto("/settings");

    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Kill Switch", exact: false }).or(page.locator("h3:has-text('Kill Switch')"))).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Activate Kill Switch" })
    ).toBeVisible();
  });

  test("clicking Activate shows confirmation dialog", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_global_policy: testGlobalPolicy,
      ...settingsMocks,
    });
    await page.goto("/settings");

    await page.getByRole("button", { name: "Activate Kill Switch" }).click();

    // Confirmation dialog should appear
    await expect(
      page.getByText("Are you sure? This will immediately block ALL agent transactions.")
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Confirm Activation" })
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible();
  });

  test("confirming activation calls toggle_kill_switch with correct args", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({
      get_global_policy: testGlobalPolicy,
      toggle_kill_switch: null,
      ...settingsMocks,
    });
    await page.goto("/settings");

    await page.getByRole("button", { name: "Activate Kill Switch" }).click();
    await page.getByRole("button", { name: "Confirm Activation" }).click();

    // After confirming, loadPolicy is called again; still on settings page
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();

    // Verify toggle_kill_switch was called with activate=true
    const calls = await page.evaluate(() => (window as any).__TAURI_INVOKE_CALLS__);
    const toggleCall = calls.find(
      (c: { cmd: string; args: unknown }) => c.cmd === "toggle_kill_switch"
    );
    expect(toggleCall).toBeTruthy();
    expect(toggleCall.args).toMatchObject({ active: true });
  });

  test("deactivate button shows when kill switch is active and calls with active=false", async ({
    page,
    mockTauri,
  }) => {
    const activePolicy = {
      ...testGlobalPolicy,
      kill_switch_active: true,
      kill_switch_reason: "Manual activation",
    };
    await mockTauri({
      get_global_policy: activePolicy,
      toggle_kill_switch: null,
      ...settingsMocks,
    });
    await page.goto("/settings");

    const deactivateBtn = page.getByRole("button", { name: "Deactivate Kill Switch" });
    await expect(deactivateBtn).toBeVisible();
    await deactivateBtn.click();

    // Verify toggle_kill_switch was called with active=false
    const calls = await page.evaluate(() => (window as any).__TAURI_INVOKE_CALLS__);
    const toggleCall = calls.find(
      (c: { cmd: string; args: unknown }) => c.cmd === "toggle_kill_switch"
    );
    expect(toggleCall).toBeTruthy();
    expect(toggleCall.args).toMatchObject({ active: false });
  });
});
