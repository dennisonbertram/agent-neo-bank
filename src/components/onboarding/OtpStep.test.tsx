import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OtpStep } from "./OtpStep";

describe("OtpStep", () => {
  it("renders 6 digit inputs", () => {
    render(<OtpStep onNext={vi.fn()} onBack={vi.fn()} />);
    const inputs = screen.getAllByRole("textbox");
    expect(inputs).toHaveLength(6);
  });

  it("shows error on invalid OTP", async () => {
    render(<OtpStep onNext={vi.fn()} onBack={vi.fn()} />);
    // Only type into first input, leaving rest empty
    const inputs = screen.getAllByRole("textbox");
    await userEvent.type(inputs[0]!, "1");
    await userEvent.click(screen.getByRole("button", { name: /verify/i }));
    await waitFor(() => {
      expect(screen.getByText(/6 digits/i)).toBeInTheDocument();
    });
  });
});
