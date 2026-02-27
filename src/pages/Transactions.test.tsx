import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Transactions } from "./Transactions";
import { mockInvoke, createMockTransaction, createMockAgent } from "@/test/helpers";

describe("Transactions", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders transaction table with data", async () => {
    const tx1 = createMockTransaction({
      id: "tx-1",
      agent_id: "agent-1",
      amount: "5.00",
      recipient: "0x1234567890abcdef1234567890abcdef12345678",
      status: "confirmed",
      description: "API payment",
      created_at: 1740700800,
    });
    const tx2 = createMockTransaction({
      id: "tx-2",
      agent_id: "agent-2",
      amount: "10.00",
      recipient: "0xabcdef1234567890abcdef1234567890abcdef12",
      status: "executing",
      description: "Service fee",
      created_at: 1740614400,
    });

    const agent1 = createMockAgent({ id: "agent-1", name: "Claude" });
    const agent2 = createMockAgent({ id: "agent-2", name: "GPT" });

    await mockInvoke({
      list_transactions: { transactions: [tx1, tx2], total: 2 },
      list_agents: [agent1, agent2],
    });

    render(<Transactions />);

    // Table column headers
    expect(await screen.findByText("Date")).toBeInTheDocument();
    expect(screen.getByText("Agent")).toBeInTheDocument();
    expect(screen.getByText("Amount")).toBeInTheDocument();
    expect(screen.getByText("Recipient")).toBeInTheDocument();
    expect(screen.getByText("Status")).toBeInTheDocument();
    expect(screen.getByText("Description")).toBeInTheDocument();

    // Transaction data (amounts now have +/- prefix)
    expect(screen.getByText(/5\.00 USDC/)).toBeInTheDocument();
    expect(screen.getByText(/10\.00 USDC/)).toBeInTheDocument();
    expect(screen.getByText("API payment")).toBeInTheDocument();
    expect(screen.getByText("Service fee")).toBeInTheDocument();

    // Agent names resolved — appears in both table and dropdown, so use getAllByText
    const claudeElements = screen.getAllByText("Claude");
    expect(claudeElements.length).toBeGreaterThanOrEqual(1);
    const gptElements = screen.getAllByText("GPT");
    expect(gptElements.length).toBeGreaterThanOrEqual(1);

    // Truncated addresses
    expect(screen.getByText("0x1234...5678")).toBeInTheDocument();
    expect(screen.getByText("0xabcd...ef12")).toBeInTheDocument();
  });

  it("shows empty state when no transactions", async () => {
    await mockInvoke({
      list_transactions: { transactions: [], total: 0 },
      list_agents: [],
    });

    render(<Transactions />);

    expect(await screen.findByText("No transactions yet")).toBeInTheDocument();
  });

  it("pagination controls work", async () => {
    const transactions = Array.from({ length: 20 }, (_, i) =>
      createMockTransaction({ id: `tx-${i}`, description: `Tx ${i}` })
    );

    const invoker = await mockInvoke({
      list_transactions: { transactions, total: 47 },
      list_agents: [],
    });

    render(<Transactions />);

    // Wait for data
    expect(await screen.findByText("Showing 1-20 of 47")).toBeInTheDocument();

    // Click Next
    const user = userEvent.setup();
    const nextButton = screen.getByRole("button", { name: /next/i });
    await user.click(nextButton);

    // Verify invoke was called with offset 20
    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith(
        "list_transactions",
        expect.objectContaining({ offset: 20 })
      );
    });
  });

  it("status filter changes displayed transactions", async () => {
    const tx = createMockTransaction({ id: "tx-1", status: "confirmed" });

    const invoker = await mockInvoke({
      list_transactions: { transactions: [tx], total: 1 },
      list_agents: [],
    });

    render(<Transactions />);

    // Wait for initial render
    await screen.findByText("Confirmed");

    const user = userEvent.setup();

    // Use selectOptions to change the native select element
    const statusSelect = screen.getByTestId("status-filter");
    await user.selectOptions(statusSelect, "confirmed");

    // Verify invoke was called with status filter
    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith(
        "list_transactions",
        expect.objectContaining({ status: "confirmed" })
      );
    });
  });

  it("displays correct status badges", async () => {
    const transactions = [
      createMockTransaction({ id: "tx-1", status: "confirmed", description: "tx-confirmed" }),
      createMockTransaction({ id: "tx-2", status: "executing", description: "tx-executing" }),
      createMockTransaction({ id: "tx-3", status: "pending", description: "tx-pending" }),
      createMockTransaction({ id: "tx-4", status: "failed", description: "tx-failed" }),
      createMockTransaction({ id: "tx-5", status: "denied", description: "tx-denied" }),
    ];

    await mockInvoke({
      list_transactions: { transactions, total: 5 },
      list_agents: [],
    });

    render(<Transactions />);

    // Wait for data to load
    await screen.findByText("tx-confirmed");

    // Check badge elements exist with correct color classes
    const confirmedBadge = screen.getByTestId("status-badge-tx-1");
    expect(confirmedBadge).toHaveAttribute("data-variant", "default");
    expect(confirmedBadge).toHaveClass("bg-green-100");

    const executingBadge = screen.getByTestId("status-badge-tx-2");
    expect(executingBadge).toHaveClass("bg-blue-100");

    const pendingBadge = screen.getByTestId("status-badge-tx-3");
    expect(pendingBadge).toHaveClass("bg-yellow-100");

    const failedBadge = screen.getByTestId("status-badge-tx-4");
    expect(failedBadge).toHaveClass("bg-red-100");

    const deniedBadge = screen.getByTestId("status-badge-tx-5");
    expect(deniedBadge).toHaveClass("bg-red-100");
  });
});
