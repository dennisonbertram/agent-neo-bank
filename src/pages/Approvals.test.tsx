import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Approvals } from "./Approvals";
import { mockInvoke, createMockAgent } from "@/test/helpers";
import { renderWithRouter } from "@/test/render";
import type { ApprovalRequest } from "@/types";

function createMockApproval(
  overrides: Partial<ApprovalRequest> = {}
): ApprovalRequest {
  return {
    id: "approval-1",
    agent_id: "agent-1",
    request_type: "transaction",
    payload: JSON.stringify({
      tx_id: "tx-1",
      to: "0xRecipient",
      amount: "50",
      asset: "USDC",
    }),
    status: "pending",
    tx_id: "tx-1",
    expires_at: Math.floor(Date.now() / 1000) + 86400,
    created_at: Math.floor(Date.now() / 1000) - 3600,
    resolved_at: null,
    resolved_by: null,
    ...overrides,
  };
}

describe("Approvals", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders pending approvals with agent context", async () => {
    const approval1 = createMockApproval({
      id: "approval-1",
      agent_id: "agent-1",
      payload: JSON.stringify({
        tx_id: "tx-1",
        to: "0xAlice",
        amount: "100",
        asset: "USDC",
      }),
    });
    const approval2 = createMockApproval({
      id: "approval-2",
      agent_id: "agent-2",
      request_type: "limit_increase",
      payload: JSON.stringify({
        proposed_daily: "5000",
        proposed_monthly: "50000",
      }),
      tx_id: null,
    });

    const agent1 = createMockAgent({ id: "agent-1", name: "Payment Bot" });
    const agent2 = createMockAgent({ id: "agent-2", name: "Treasury Agent" });

    await mockInvoke({
      list_approvals: [approval1, approval2],
      list_agents: [agent1, agent2],
    });

    renderWithRouter(<Approvals />);

    // Verify agent names appear
    expect(await screen.findByText("Payment Bot")).toBeInTheDocument();
    expect(screen.getByText("Treasury Agent")).toBeInTheDocument();

    // Verify transaction details from payload (CurrencyDisplay renders USDC as $100.00)
    expect(screen.getByText(/\$100\.00/)).toBeInTheDocument();

    // Verify approval type badges (shown as rounded pills)
    expect(screen.getByText("transaction")).toBeInTheDocument();
    expect(screen.getByText("limit_increase")).toBeInTheDocument();
  });

  it("shows empty state when no pending approvals", async () => {
    await mockInvoke({
      list_approvals: [],
      list_agents: [],
    });

    renderWithRouter(<Approvals />);

    expect(
      await screen.findByText("No pending approvals right now.")
    ).toBeInTheDocument();
  });

  it("approve button calls resolve_approval with correct args", async () => {
    const approval = createMockApproval({ id: "approval-xyz" });
    const agent = createMockAgent({ id: "agent-1", name: "Test Agent" });

    const invoker = await mockInvoke({
      list_approvals: [approval],
      list_agents: [agent],
      resolve_approval: undefined,
    });

    renderWithRouter(<Approvals />);

    await screen.findByText("Test Agent");

    const user = userEvent.setup();
    // First click shows confirmation
    const approveButton = screen.getByRole("button", { name: /approve/i });
    await user.click(approveButton);
    // Second click confirms
    await user.click(screen.getByRole("button", { name: /confirm/i }));

    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith("resolve_approval", {
        approval_id: "approval-xyz",
        decision: "approved",
      });
    });
  });

  it("deny button calls resolve_approval with correct args", async () => {
    const approval = createMockApproval({ id: "approval-deny-me" });
    const agent = createMockAgent({ id: "agent-1", name: "Test Agent" });

    const invoker = await mockInvoke({
      list_approvals: [approval],
      list_agents: [agent],
      resolve_approval: undefined,
    });

    renderWithRouter(<Approvals />);

    await screen.findByText("Test Agent");

    const user = userEvent.setup();
    // First click shows confirmation
    const denyButton = screen.getByRole("button", { name: /deny/i });
    await user.click(denyButton);
    // Second click confirms
    await user.click(screen.getByRole("button", { name: /confirm/i }));

    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith("resolve_approval", {
        approval_id: "approval-deny-me",
        decision: "denied",
      });
    });
  });

  it("filter tabs switch between pending and all", async () => {
    const pendingApproval = createMockApproval({ id: "approval-pending" });
    const agent = createMockAgent({ id: "agent-1", name: "Test Agent" });

    const invoker = await mockInvoke({
      list_approvals: [pendingApproval],
      list_agents: [agent],
    });

    renderWithRouter(<Approvals />);

    // Wait for initial load
    await screen.findByText("Test Agent");

    const user = userEvent.setup();

    // Click "All" filter button
    const allButton = screen.getByRole("button", { name: /^all$/i });
    await user.click(allButton);

    // Verify list_approvals was called again (for "all" filter, no status arg)
    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith("list_approvals", {});
    });
  });
});
