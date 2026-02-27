import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Agents } from "./Agents";
import { mockInvoke, createMockAgent } from "@/test/helpers";
import { renderWithRouter } from "@/test/render";

const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe("Agents", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    mockNavigate.mockReset();
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
    expect(screen.getByText("Type: payment")).toBeInTheDocument();
    expect(screen.getByText("Type: research")).toBeInTheDocument();
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
    expect(activeBadge).toHaveTextContent("active");
    expect(activeBadge).toHaveClass("bg-green-100");

    const pendingBadge = screen.getByTestId("status-badge-a2");
    expect(pendingBadge).toHaveTextContent("pending");
    expect(pendingBadge).toHaveClass("bg-yellow-100");

    const suspendedBadge = screen.getByTestId("status-badge-a3");
    expect(suspendedBadge).toHaveTextContent("suspended");
    expect(suspendedBadge).toHaveClass("bg-orange-100");

    const revokedBadge = screen.getByTestId("status-badge-a4");
    expect(revokedBadge).toHaveTextContent("revoked");
    expect(revokedBadge).toHaveClass("bg-red-100");
  });

  it("shows empty state when no agents", async () => {
    await mockInvoke({ list_agents: [] });

    renderWithRouter(<Agents />);

    expect(
      await screen.findByText("No agents registered")
    ).toBeInTheDocument();
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

    const user = userEvent.setup();
    const card = screen.getByTestId("agent-card-agent-click-test");
    await user.click(card);

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith("/agents/agent-click-test");
    });
  });
});
