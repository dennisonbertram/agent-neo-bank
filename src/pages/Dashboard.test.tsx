import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import { renderWithRouter } from "@/test/render";
import { Dashboard } from "./Dashboard";

vi.mock("@/hooks/useBalance", () => ({
  useBalance: () => ({
    balance: "1247.83",
    balances: null,
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
  it("renders balance in the hero card", () => {
    renderWithRouter(<Dashboard />);
    expect(screen.getByText("$1,247.83")).toBeInTheDocument();
  });

  it("shows empty state for agents when none registered", async () => {
    renderWithRouter(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("No agents yet")).toBeInTheDocument();
    });
  });

  it("shows empty state for transactions", () => {
    renderWithRouter(<Dashboard />);
    expect(screen.getByText("No transactions yet")).toBeInTheDocument();
  });

  it("renders global budget utilization section", async () => {
    renderWithRouter(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("Global Budget Utilization")).toBeInTheDocument();
    });
  });

  it("renders global budget progress bars after loading", async () => {
    renderWithRouter(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("Daily Global")).toBeInTheDocument();
      expect(screen.getByText("Weekly Global")).toBeInTheDocument();
      expect(screen.getByText("Monthly Global")).toBeInTheDocument();
    });
  });

  it("renders quick action links", () => {
    renderWithRouter(<Dashboard />);
    expect(screen.getByText("Send")).toBeInTheDocument();
    expect(screen.getByText("Fund")).toBeInTheDocument();
    expect(screen.getByText("Invite Agent")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });

  it("renders page header", () => {
    renderWithRouter(<Dashboard />);
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
  });

  it("renders USDC label", () => {
    renderWithRouter(<Dashboard />);
    expect(screen.getByText("USDC")).toBeInTheDocument();
  });

  it("renders Fund Wallet link in hero card", () => {
    renderWithRouter(<Dashboard />);
    expect(screen.getByText("Fund Wallet")).toBeInTheDocument();
  });

  it("calls invoke for budget data on mount", async () => {
    renderWithRouter(<Dashboard />);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_agent_budget_summaries");
      expect(mockInvoke).toHaveBeenCalledWith("get_global_budget_summary");
    });
  });

  it("renders agent cards when agent budgets exist", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_agent_budget_summaries") {
        return [
          {
            agent_id: "a1",
            agent_name: "Trading Bot",
            daily_spent: "50",
            daily_cap: "200",
            weekly_spent: "150",
            weekly_cap: "1000",
            monthly_spent: "400",
            monthly_cap: "5000",
          },
        ];
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
      throw new Error(`Unmocked invoke: ${cmd}`);
    });

    renderWithRouter(<Dashboard />);
    await waitFor(() => {
      expect(screen.getAllByText("Trading Bot").length).toBeGreaterThanOrEqual(1);
    });
    // Agent Budgets section should also appear for detailed view
    await waitFor(() => {
      expect(screen.getByText("Agent Budgets")).toBeInTheDocument();
    });
  });
});
