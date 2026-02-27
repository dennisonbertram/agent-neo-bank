import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { Dashboard } from "./Dashboard";

vi.mock("@/hooks/useBalance", () => ({
  useBalance: () => ({
    balance: "1247.83",
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
}));

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
});
