import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Notifications } from "./Notifications";
import { mockInvoke } from "@/test/helpers";
import { renderWithRouter } from "@/test/render";
import type { NotificationPreferences } from "@/types";

function createMockPrefs(
  overrides: Partial<NotificationPreferences> = {}
): NotificationPreferences {
  return {
    id: "default",
    enabled: true,
    on_all_tx: true,
    on_large_tx: false,
    large_tx_threshold: "100",
    on_errors: true,
    on_limit_requests: true,
    on_agent_registration: true,
    ...overrides,
  };
}

describe("Notifications", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders notification toggles with loaded preferences", async () => {
    const prefs = createMockPrefs();

    await mockInvoke({
      get_notification_preferences: prefs,
    });

    renderWithRouter(<Notifications />);

    // All toggle labels should render
    expect(
      await screen.findByText("Enable Notifications")
    ).toBeInTheDocument();
    expect(screen.getByText("All Transactions")).toBeInTheDocument();
    expect(screen.getByText("Large Transactions Only")).toBeInTheDocument();
    expect(screen.getByText("Errors")).toBeInTheDocument();
    expect(screen.getByText("Limit Requests")).toBeInTheDocument();
    expect(screen.getByText("Agent Registration")).toBeInTheDocument();

    // Card title and description
    expect(screen.getByText("Notification Preferences")).toBeInTheDocument();
    expect(
      screen.getByText("Choose which events trigger OS notifications")
    ).toBeInTheDocument();

    // Save button
    expect(
      screen.getByRole("button", { name: /save preferences/i })
    ).toBeInTheDocument();
  });

  it("toggle changes preference state", async () => {
    const prefs = createMockPrefs({ on_all_tx: true });

    await mockInvoke({
      get_notification_preferences: prefs,
    });

    renderWithRouter(<Notifications />);

    await screen.findByText("Enable Notifications");

    const user = userEvent.setup();

    // Find the "All Transactions" toggle (second switch)
    const switches = screen.getAllByRole("switch");
    // on_all_tx is the second toggle (index 1)
    const allTxSwitch = switches[1];
    expect(allTxSwitch).toHaveAttribute("aria-checked", "true");

    await user.click(allTxSwitch);

    expect(allTxSwitch).toHaveAttribute("aria-checked", "false");
  });

  it("save button calls update command", async () => {
    const prefs = createMockPrefs({ on_errors: true });

    const invoker = await mockInvoke({
      get_notification_preferences: prefs,
      update_notification_preferences: undefined,
    });

    renderWithRouter(<Notifications />);

    await screen.findByText("Enable Notifications");

    const user = userEvent.setup();

    // Toggle errors off
    const switches = screen.getAllByRole("switch");
    // on_errors is the 4th toggle (index 3)
    const errorsSwitch = switches[3];
    await user.click(errorsSwitch);

    // Click save
    const saveButton = screen.getByRole("button", {
      name: /save preferences/i,
    });
    await user.click(saveButton);

    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith(
        "update_notification_preferences",
        expect.objectContaining({
          prefs: expect.objectContaining({
            on_errors: false,
          }),
        })
      );
    });
  });

  it("threshold input shown when large tx enabled", async () => {
    const prefs = createMockPrefs({
      on_large_tx: true,
      large_tx_threshold: "500",
    });

    await mockInvoke({
      get_notification_preferences: prefs,
    });

    renderWithRouter(<Notifications />);

    await screen.findByText("Enable Notifications");

    // Threshold input should be visible
    const thresholdInput = screen.getByLabelText(
      /large transaction threshold/i
    );
    expect(thresholdInput).toBeInTheDocument();
    expect(thresholdInput).toHaveValue(500);
  });

  it("threshold input hidden when large tx disabled", async () => {
    const prefs = createMockPrefs({
      on_large_tx: false,
    });

    await mockInvoke({
      get_notification_preferences: prefs,
    });

    renderWithRouter(<Notifications />);

    await screen.findByText("Enable Notifications");

    // Threshold input should NOT be visible
    expect(
      screen.queryByLabelText(/large transaction threshold/i)
    ).not.toBeInTheDocument();
  });
});
