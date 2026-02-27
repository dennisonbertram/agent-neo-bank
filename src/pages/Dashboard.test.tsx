import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { Dashboard } from "./Dashboard";

vi.mock("@/hooks/useBalance", () => ({
  useBalance: () => ({
    balance: "1247.83",
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
}));

// Mock invoke for budget commands
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

beforeEach(() => {
  mockInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "get_agent_budget_summaries") {
      return [];
    }
    if (cmd === "get_global_budget_summary") {
      return {
        daily_spent: "100",
        daily_cap: "10000",
        weekly_spent: "500",
        weekly_cap: "50000",
        monthly_spent: "2000",
        monthly_cap: "200000",
        kill_switch_active: false,
      };
    }
    if (cmd === "get_balance") {
      return { balance: "1247.83", asset: "USDC" };
    }
    throw new Error(`Unmocked invoke: ${cmd}`);
  });
});

describe("Dashboard", () => {
  it("renders balance card", () => {
    render(<Dashboard />);
    expect(screen.getByText("$1,247.83")).toBeInTheDocument();
  });

  it("shows empty state for agents grid", () => {
    render(<Dashboard />);
    expect(screen.getByText("No agents registered yet")).toBeInTheDocument();
  });

  it("shows empty state for transactions", () => {
    render(<Dashboard />);
    expect(screen.getByText("No transactions yet")).toBeInTheDocument();
  });

  it("renders global budget utilization section", async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("Global Budget Utilization")).toBeInTheDocument();
    });
  });

  it("renders agent budgets section", async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("Agent Budgets")).toBeInTheDocument();
    });
  });

  it("shows no agents message when no agent budgets", async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("No agents registered")).toBeInTheDocument();
    });
  });

  it("renders global budget progress bars after loading", async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("Daily Global")).toBeInTheDocument();
      expect(screen.getByText("Weekly Global")).toBeInTheDocument();
      expect(screen.getByText("Monthly Global")).toBeInTheDocument();
    });
  });
});
