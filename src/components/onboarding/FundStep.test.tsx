import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { FundStep } from "./FundStep";

describe("FundStep", () => {
  it("displays wallet address", () => {
    render(<FundStep address="0x1234567890abcdef1234567890abcdef12345678" onNext={() => {}} />);
    expect(screen.getByText(/0x1234/)).toBeInTheDocument();
  });

  it("has copy button", () => {
    render(<FundStep address="0x1234567890abcdef1234567890abcdef12345678" onNext={() => {}} />);
    expect(screen.getByRole("button", { name: /copy/i })).toBeInTheDocument();
  });
});
