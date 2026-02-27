import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EmailStep } from "./EmailStep";

describe("EmailStep", () => {
  it("renders email input", () => {
    render(<EmailStep onNext={vi.fn()} onBack={vi.fn()} />);
    expect(screen.getByPlaceholderText(/you@example\.com/i)).toBeInTheDocument();
  });

  it("shows validation error for invalid email", async () => {
    render(<EmailStep onNext={vi.fn()} onBack={vi.fn()} />);
    const input = screen.getByPlaceholderText(/you@example\.com/i);
    await userEvent.type(input, "notanemail");
    await userEvent.click(screen.getByRole("button", { name: /send verification code/i }));
    await waitFor(() => {
      expect(screen.getByText(/valid email/i)).toBeInTheDocument();
    });
  });

  it("calls onNext with email when form submitted", async () => {
    const onNext = vi.fn();
    render(<EmailStep onNext={onNext} onBack={vi.fn()} />);
    const input = screen.getByPlaceholderText(/you@example\.com/i);
    await userEvent.type(input, "test@example.com");
    await userEvent.click(screen.getByRole("button", { name: /send verification code/i }));
    await waitFor(() => {
      expect(onNext).toHaveBeenCalledWith("test@example.com");
    });
  });
});
