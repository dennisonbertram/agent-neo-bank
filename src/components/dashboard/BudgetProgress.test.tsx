import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { BudgetProgress } from "./BudgetProgress";
import { AgentBudgets } from "./AgentBudgets";
import { GlobalBudget } from "./GlobalBudget";
import type { AgentBudgetSummary, GlobalBudgetSummary } from "@/types";

describe("BudgetProgress", () => {
  it("renders progress bar with correct percentage", () => {
    render(<BudgetProgress label="Daily" spent="50" cap="100" />);

    expect(screen.getByText("Daily")).toBeInTheDocument();
    expect(screen.getByText("50 / 100 USDC")).toBeInTheDocument();

    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "50%" });
  });

  it("shows warning color above 80%", () => {
    render(<BudgetProgress label="Daily" spent="85" cap="100" />);

    const bar = screen.getByTestId("progress-bar");
    expect(bar.className).toContain("bg-yellow-500");
    expect(bar).toHaveStyle({ width: "85%" });
  });

  it("shows danger color above 95%", () => {
    render(<BudgetProgress label="Daily" spent="98" cap="100" />);

    const bar = screen.getByTestId("progress-bar");
    expect(bar.className).toContain("bg-red-500");
    expect(bar).toHaveStyle({ width: "98%" });
  });

  it("handles zero data", () => {
    render(<BudgetProgress label="Daily" spent="0" cap="0" />);

    expect(screen.getByText("Daily")).toBeInTheDocument();
    expect(screen.getByText("0 / 0 USDC")).toBeInTheDocument();

    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "0%" });
  });

  it("caps at 100% when spent exceeds cap", () => {
    render(<BudgetProgress label="Daily" spent="150" cap="100" />);

    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "100%" });
  });

  it("shows green color at low utilization", () => {
    render(<BudgetProgress label="Daily" spent="30" cap="100" />);

    const bar = screen.getByTestId("progress-bar");
    expect(bar.className).toContain("bg-green-500");
  });
});

describe("AgentBudgets", () => {
  it("renders agent budgets list", () => {
    const summaries: AgentBudgetSummary[] = [
      {
        agent_id: "agent-1",
        agent_name: "Claude",
        daily_spent: "100",
        daily_cap: "1000",
        weekly_spent: "500",
        weekly_cap: "5000",
        monthly_spent: "2000",
        monthly_cap: "20000",
      },
      {
        agent_id: "agent-2",
        agent_name: "GPT",
        daily_spent: "200",
        daily_cap: "1000",
        weekly_spent: "800",
        weekly_cap: "5000",
        monthly_spent: "3000",
        monthly_cap: "20000",
      },
    ];

    render(<AgentBudgets summaries={summaries} />);

    expect(screen.getByText("Claude")).toBeInTheDocument();
    expect(screen.getByText("GPT")).toBeInTheDocument();
  });

  it("shows empty message when no agents", () => {
    render(<AgentBudgets summaries={[]} />);

    expect(screen.getByText("No agents registered")).toBeInTheDocument();
  });
});

describe("GlobalBudget", () => {
  it("renders global budget progress bars", () => {
    const summary: GlobalBudgetSummary = {
      daily_spent: "500",
      daily_cap: "10000",
      weekly_spent: "2000",
      weekly_cap: "50000",
      monthly_spent: "8000",
      monthly_cap: "200000",
      kill_switch_active: false,
    };

    render(<GlobalBudget summary={summary} />);

    expect(screen.getByText("Daily Global")).toBeInTheDocument();
    expect(screen.getByText("Weekly Global")).toBeInTheDocument();
    expect(screen.getByText("Monthly Global")).toBeInTheDocument();
  });

  it("shows kill switch warning when active", () => {
    const summary: GlobalBudgetSummary = {
      daily_spent: "0",
      daily_cap: "10000",
      weekly_spent: "0",
      weekly_cap: "50000",
      monthly_spent: "0",
      monthly_cap: "200000",
      kill_switch_active: true,
    };

    render(<GlobalBudget summary={summary} />);

    expect(
      screen.getByText("Kill Switch Active — All transactions blocked"),
    ).toBeInTheDocument();
  });

  it("renders nothing when summary is null", () => {
    const { container } = render(<GlobalBudget summary={null} />);

    expect(container.innerHTML).toBe("");
  });
});
