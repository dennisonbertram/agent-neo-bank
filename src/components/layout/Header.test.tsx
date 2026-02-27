import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { Header } from "./Header";

const mockUseBalance = vi.fn();

vi.mock("@/hooks/useBalance", () => ({
  useBalance: () => mockUseBalance(),
}));

describe("Header", () => {
  beforeEach(() => {
    mockUseBalance.mockReset();
  });

  it("displays Agent Neo Bank title", () => {
    mockUseBalance.mockReturnValue({ balance: null, isLoading: false, error: null, refetch: vi.fn() });
    render(<Header />);
    expect(screen.getByText("Agent Neo Bank")).toBeInTheDocument();
  });

  it("shows balance when loaded", () => {
    mockUseBalance.mockReturnValue({ balance: "1247.83", isLoading: false, error: null, refetch: vi.fn() });
    render(<Header />);
    expect(screen.getByText("$1,247.83")).toBeInTheDocument();
  });

  it("shows loading state when balance loading", () => {
    mockUseBalance.mockReturnValue({ balance: null, isLoading: true, error: null, refetch: vi.fn() });
    render(<Header />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });
});
