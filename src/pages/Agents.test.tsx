import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Agents } from "./Agents";
import { mockInvoke, createMockAgent } from "@/test/helpers";
import { renderWithRouter } from "@/test/render";

describe("Agents", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders agent cards with data", async () => {
    const agent1 = createMockAgent({
      id: "agent-1",
      name: "Payment Bot",
      purpose: "Handles payments",
      agent_type: "payment",
      status: "active",
    });
    const agent2 = createMockAgent({
      id: "agent-2",
      name: "Research Agent",
      purpose: "Performs research tasks",
      agent_type: "research",
      status: "pending",
    });

    await mockInvoke({ list_agents: [agent1, agent2] });

    renderWithRouter(<Agents />);

    expect(await screen.findByText("Payment Bot")).toBeInTheDocument();
    expect(screen.getByText("Research Agent")).toBeInTheDocument();
    expect(screen.getByText("Handles payments")).toBeInTheDocument();
    expect(screen.getByText("Performs research tasks")).toBeInTheDocument();
  });

  it("shows correct status badges", async () => {
    const activeAgent = createMockAgent({
      id: "a1",
      name: "Active Agent",
      status: "active",
    });
    const pendingAgent = createMockAgent({
      id: "a2",
      name: "Pending Agent",
      status: "pending",
    });
    const suspendedAgent = createMockAgent({
      id: "a3",
      name: "Suspended Agent",
      status: "suspended",
    });
    const revokedAgent = createMockAgent({
      id: "a4",
      name: "Revoked Agent",
      status: "revoked",
    });

    await mockInvoke({
      list_agents: [activeAgent, pendingAgent, suspendedAgent, revokedAgent],
    });

    renderWithRouter(<Agents />);

    await screen.findByText("Active Agent");

    const activeBadge = screen.getByTestId("status-badge-a1");
    expect(activeBadge).toHaveTextContent("Active");

    const pendingBadge = screen.getByTestId("status-badge-a2");
    expect(pendingBadge).toHaveTextContent("Pending");

    const suspendedBadge = screen.getByTestId("status-badge-a3");
    expect(suspendedBadge).toHaveTextContent("Suspended");

    const revokedBadge = screen.getByTestId("status-badge-a4");
    expect(revokedBadge).toHaveTextContent("Revoked");
  });

  it("shows empty state when no agents", async () => {
    await mockInvoke({ list_agents: [] });

    renderWithRouter(<Agents />);

    expect(await screen.findByText("No agents yet")).toBeInTheDocument();
  });

  it("shows loading state initially", async () => {
    // Use a never-resolving promise to keep loading state
    const mod = await import("@tauri-apps/api/core");
    vi.mocked(mod).invoke.mockImplementation(
      () => new Promise(() => {}) // never resolves
    );

    renderWithRouter(<Agents />);

    expect(screen.getByText("Loading agents...")).toBeInTheDocument();
  });

  it("clicking agent card navigates to detail", async () => {
    const agent = createMockAgent({
      id: "agent-click-test",
      name: "Clickable Agent",
      status: "active",
    });

    await mockInvoke({ list_agents: [agent] });

    renderWithRouter(<Agents />);

    await screen.findByText("Clickable Agent");

    const card = screen.getByTestId("agent-card-agent-click-test");
    expect(card.closest("a")).toHaveAttribute(
      "href",
      "/agents/agent-click-test"
    );
  });

  it("filters agents by search", async () => {
    const agent1 = createMockAgent({
      id: "a1",
      name: "Payment Bot",
      status: "active",
    });
    const agent2 = createMockAgent({
      id: "a2",
      name: "Research Agent",
      status: "active",
    });

    await mockInvoke({ list_agents: [agent1, agent2] });

    renderWithRouter(<Agents />);

    await screen.findByText("Payment Bot");

    const user = userEvent.setup();
    const searchInput = screen.getByPlaceholderText("Search agents...");
    await user.type(searchInput, "Payment");

    expect(screen.getByText("Payment Bot")).toBeInTheDocument();
    expect(screen.queryByText("Research Agent")).not.toBeInTheDocument();
  });

  it("filters agents by tab", async () => {
    const activeAgent = createMockAgent({
      id: "a1",
      name: "Active Agent",
      status: "active",
    });
    const pendingAgent = createMockAgent({
      id: "a2",
      name: "Pending Agent",
      status: "pending",
    });

    await mockInvoke({ list_agents: [activeAgent, pendingAgent] });

    renderWithRouter(<Agents />);

    await screen.findByText("Active Agent");

    const user = userEvent.setup();
    // Click the "Active" tab button
    const activeTabBtn = screen.getByRole("button", { name: /Active/i });
    await user.click(activeTabBtn);

    expect(screen.getByText("Active Agent")).toBeInTheDocument();
    expect(screen.queryByText("Pending Agent")).not.toBeInTheDocument();
  });
});
