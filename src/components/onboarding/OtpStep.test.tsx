import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OtpStep } from "./OtpStep";

describe("OtpStep", () => {
  it("renders 6-digit input", () => {
    render(<OtpStep onNext={vi.fn()} onBack={vi.fn()} />);
    expect(screen.getByPlaceholderText(/000000/i)).toBeInTheDocument();
  });

  it("shows error on invalid OTP", async () => {
    render(<OtpStep onNext={vi.fn()} onBack={vi.fn()} />);
    const input = screen.getByPlaceholderText(/000000/i);
    await userEvent.type(input, "123");
    await userEvent.click(screen.getByRole("button", { name: /verify/i }));
    await waitFor(() => {
      expect(screen.getByText(/6 digits/i)).toBeInTheDocument();
    });
  });
});
