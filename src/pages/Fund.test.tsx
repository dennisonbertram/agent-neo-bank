import { render, screen, fireEvent } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { describe, it, expect } from "vitest";
import { Fund } from "./Fund";

function renderFund() {
  return render(
    <BrowserRouter>
      <Fund />
    </BrowserRouter>
  );
}

describe("Fund", () => {
  it("renders page title", () => {
    renderFund();
    expect(screen.getByText("Fund Wallet")).toBeInTheDocument();
  });

  it("shows Buy Crypto tab by default", () => {
    renderFund();
    expect(screen.getByText("Buy crypto with Coinbase")).toBeInTheDocument();
  });

  it("switches to Deposit tab", () => {
    renderFund();
    fireEvent.click(screen.getByText("Deposit"));
    expect(screen.getByText("Your Wallet Address")).toBeInTheDocument();
  });
});
