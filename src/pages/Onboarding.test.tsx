import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Onboarding } from "./Onboarding";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve({ address: "0xTestAddress" })),
}));

describe("Onboarding", () => {
  it("renders WelcomeStep initially", () => {
    render(<Onboarding />);
    expect(screen.getByText(/welcome to agent neo bank/i)).toBeInTheDocument();
  });

  it("advances through steps", async () => {
    render(<Onboarding />);
    // Step 0: Welcome
    expect(screen.getByText(/welcome to agent neo bank/i)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /get started/i }));
    // Step 1: Email
    expect(screen.getByPlaceholderText(/email/i)).toBeInTheDocument();
  });
});
