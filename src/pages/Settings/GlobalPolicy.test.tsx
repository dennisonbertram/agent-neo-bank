import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { GlobalPolicySettings } from "./GlobalPolicy";
import { mockInvoke } from "@/test/helpers";
import { renderWithRouter } from "@/test/render";
import type { GlobalPolicy } from "@/types";

function createMockPolicy(
  overrides: Partial<GlobalPolicy> = {}
): GlobalPolicy {
  return {
    id: "default",
    daily_cap: "1000",
    weekly_cap: "5000",
    monthly_cap: "20000",
    min_reserve_balance: "100",
    kill_switch_active: false,
    kill_switch_reason: "",
    updated_at: Math.floor(Date.now() / 1000),
    ...overrides,
  };
}

describe("GlobalPolicySettings", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders global policy caps", async () => {
    const policy = createMockPolicy();

    await mockInvoke({
      get_global_policy: policy,
    });

    renderWithRouter(<GlobalPolicySettings />);

    // Card title and description
    expect(await screen.findByText("Global Policy")).toBeInTheDocument();
    expect(
      screen.getByText("Wallet-level spending controls and kill switch")
    ).toBeInTheDocument();

    // Cap inputs should show values
    const dailyInput = screen.getByLabelText(/daily cap/i);
    expect(dailyInput).toHaveValue(1000);

    const weeklyInput = screen.getByLabelText(/weekly cap/i);
    expect(weeklyInput).toHaveValue(5000);

    const monthlyInput = screen.getByLabelText(/monthly cap/i);
    expect(monthlyInput).toHaveValue(20000);

    const reserveInput = screen.getByLabelText(/minimum reserve balance/i);
    expect(reserveInput).toHaveValue(100);

    // Save button
    expect(
      screen.getByRole("button", { name: /save caps/i })
    ).toBeInTheDocument();

    // Kill switch button
    expect(
      screen.getByRole("button", { name: /activate kill switch/i })
    ).toBeInTheDocument();
  });

  it("kill switch toggle shows confirmation", async () => {
    const policy = createMockPolicy({ kill_switch_active: false });

    await mockInvoke({
      get_global_policy: policy,
    });

    renderWithRouter(<GlobalPolicySettings />);

    await screen.findByText("Global Policy");

    const user = userEvent.setup();

    // Click kill switch button
    const killButton = screen.getByRole("button", {
      name: /activate kill switch/i,
    });
    await user.click(killButton);

    // Confirmation dialog should appear
    expect(
      screen.getByText(/are you sure\? this will immediately block all/i)
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /confirm activation/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /cancel/i })
    ).toBeInTheDocument();
  });

  it("kill switch confirmation calls toggle command", async () => {
    const policy = createMockPolicy({ kill_switch_active: false });

    const invoker = await mockInvoke({
      get_global_policy: policy,
      toggle_kill_switch: undefined,
    });

    renderWithRouter(<GlobalPolicySettings />);

    await screen.findByText("Global Policy");

    const user = userEvent.setup();

    // Click kill switch to show confirmation
    const killButton = screen.getByRole("button", {
      name: /activate kill switch/i,
    });
    await user.click(killButton);

    // Click confirm
    const confirmButton = screen.getByRole("button", {
      name: /confirm activation/i,
    });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith(
        "toggle_kill_switch",
        expect.objectContaining({
          active: true,
          reason: "Manual activation",
        })
      );
    });
  });

  it("save caps calls update command", async () => {
    const policy = createMockPolicy({ daily_cap: "1000" });

    const invoker = await mockInvoke({
      get_global_policy: policy,
      update_global_policy: undefined,
    });

    renderWithRouter(<GlobalPolicySettings />);

    await screen.findByText("Global Policy");

    const user = userEvent.setup();

    // Change daily cap
    const dailyInput = screen.getByLabelText(/daily cap/i);
    await user.clear(dailyInput);
    await user.type(dailyInput, "2000");

    // Click save
    const saveButton = screen.getByRole("button", { name: /save caps/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith(
        "update_global_policy",
        expect.objectContaining({
          policy: expect.objectContaining({
            daily_cap: "2000",
          }),
        })
      );
    });
  });

  it("deactivate kill switch calls without confirmation", async () => {
    const policy = createMockPolicy({ kill_switch_active: true });

    const invoker = await mockInvoke({
      get_global_policy: policy,
      toggle_kill_switch: undefined,
    });

    renderWithRouter(<GlobalPolicySettings />);

    await screen.findByText("Global Policy");

    const user = userEvent.setup();

    // When active, button says "Deactivate Kill Switch"
    const deactivateButton = screen.getByRole("button", {
      name: /deactivate kill switch/i,
    });
    await user.click(deactivateButton);

    // Should call directly without confirmation dialog
    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith(
        "toggle_kill_switch",
        expect.objectContaining({
          active: false,
        })
      );
    });

    // Confirmation dialog should NOT appear
    expect(
      screen.queryByText(/are you sure\? this will immediately block all/i)
    ).not.toBeInTheDocument();
  });
});
