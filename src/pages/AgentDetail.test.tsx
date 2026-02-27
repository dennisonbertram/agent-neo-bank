import { describe, it, expect } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Routes, Route } from "react-router-dom";
import { renderWithRouter } from "../test/render";
import {
  mockInvoke,
  createMockAgent,
  createMockSpendingPolicy,
  createMockTransaction,
} from "../test/helpers";
import { AgentDetail } from "./AgentDetail";

function renderAgentDetail(agentId = "test-agent-id") {
  return renderWithRouter(
    <Routes>
      <Route path="/agents/:id" element={<AgentDetail />} />
    </Routes>,
    { route: `/agents/${agentId}` },
  );
}

describe("AgentDetail", () => {
  const agent = createMockAgent({
    name: "Claude Agent",
    purpose: "AI tasks",
    agent_type: "autonomous",
    status: "active",
  });
  const policy = createMockSpendingPolicy({
    per_tx_max: "50",
    daily_cap: "500",
    weekly_cap: "2000",
    monthly_cap: "8000",
  });

  it("renders agent detail with profile info", async () => {
    await mockInvoke({
      get_agent: agent,
      get_agent_spending_policy: policy,
      get_agent_transactions: [],
    });

    renderAgentDetail();

    await waitFor(() => {
      expect(screen.getByText("Claude Agent")).toBeInTheDocument();
    });
    expect(screen.getByText("AI tasks")).toBeInTheDocument();
    expect(screen.getByText("autonomous")).toBeInTheDocument();
    expect(screen.getByText("active")).toBeInTheDocument();
  });

  it("renders spending limits", async () => {
    await mockInvoke({
      get_agent: agent,
      get_agent_spending_policy: policy,
      get_agent_transactions: [],
    });

    renderAgentDetail();

    await waitFor(() => {
      expect(screen.getByText("Spending Limits")).toBeInTheDocument();
    });
    expect(screen.getByText("50")).toBeInTheDocument();
    expect(screen.getByText("500")).toBeInTheDocument();
    expect(screen.getByText("2000")).toBeInTheDocument();
    expect(screen.getByText("8000")).toBeInTheDocument();
  });

  it("edit limits saves updated policy", async () => {
    const invokeMock = await mockInvoke({
      get_agent: agent,
      get_agent_spending_policy: policy,
      get_agent_transactions: [],
      update_agent_spending_policy: undefined,
    });

    renderAgentDetail();
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("Spending Limits")).toBeInTheDocument();
    });

    // Click Edit button
    await user.click(screen.getByRole("button", { name: "Edit" }));

    // Change per_tx_max value
    const perTxInput = screen.getByLabelText("Per Transaction Max");
    await user.clear(perTxInput);
    await user.type(perTxInput, "75");

    // Click Save
    await user.click(screen.getByRole("button", { name: "Save Limits" }));

    await waitFor(() => {
      const updateCalls = invokeMock.mock.calls.filter(
        (call) => call[0] === "update_agent_spending_policy",
      );
      expect(updateCalls.length).toBeGreaterThan(0);
    });
  });

  it("suspend button calls suspend command", async () => {
    const invokeMock = await mockInvoke({
      get_agent: agent,
      get_agent_spending_policy: policy,
      get_agent_transactions: [],
      suspend_agent: undefined,
    });

    renderAgentDetail();
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("Claude Agent")).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "Suspend" }));

    await waitFor(() => {
      const suspendCalls = invokeMock.mock.calls.filter(
        (call) => call[0] === "suspend_agent",
      );
      expect(suspendCalls.length).toBeGreaterThan(0);
    });
  });

  it("renders recent transactions", async () => {
    const transactions = [
      createMockTransaction({
        id: "tx-1",
        amount: "25.00",
        description: "API call payment",
        status: "confirmed",
      }),
      createMockTransaction({
        id: "tx-2",
        amount: "10.50",
        description: "Service subscription",
        status: "pending",
      }),
    ];

    await mockInvoke({
      get_agent: agent,
      get_agent_spending_policy: policy,
      get_agent_transactions: transactions,
    });

    renderAgentDetail();

    await waitFor(() => {
      expect(screen.getByText("Recent Activity")).toBeInTheDocument();
    });

    expect(screen.getByText("25.00")).toBeInTheDocument();
    expect(screen.getByText("API call payment")).toBeInTheDocument();
    expect(screen.getByText("10.50")).toBeInTheDocument();
    expect(screen.getByText("Service subscription")).toBeInTheDocument();
  });
});
