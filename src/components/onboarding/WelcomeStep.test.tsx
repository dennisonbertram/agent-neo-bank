import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WelcomeStep } from "./WelcomeStep";

describe("WelcomeStep", () => {
  it("renders welcome text", () => {
    render(<WelcomeStep onNext={vi.fn()} />);
    expect(screen.getByText(/give your ai agents spending power/i)).toBeInTheDocument();
  });

  it("renders continue button", () => {
    render(<WelcomeStep onNext={vi.fn()} />);
    expect(screen.getByRole("button", { name: /get started/i })).toBeInTheDocument();
  });

  it("calls onNext when button clicked", async () => {
    const onNext = vi.fn();
    render(<WelcomeStep onNext={onNext} />);
    await userEvent.click(screen.getByRole("button", { name: /get started/i }));
    expect(onNext).toHaveBeenCalledOnce();
  });
});
