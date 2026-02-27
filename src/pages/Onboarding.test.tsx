import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Onboarding } from "./Onboarding";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("Onboarding", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders WelcomeStep initially", () => {
    render(<Onboarding />);
    expect(
      screen.getByText(/give your ai agents spending power/i)
    ).toBeInTheDocument();
  });

  it("advances through steps", async () => {
    render(<Onboarding />);
    // Step 0: Welcome
    expect(
      screen.getByText(/give your ai agents spending power/i)
    ).toBeInTheDocument();
    await userEvent.click(
      screen.getByRole("button", { name: /get started/i })
    );
    // Step 1: Email
    expect(
      screen.getByPlaceholderText(/you@example\.com/i)
    ).toBeInTheDocument();
  });

  it("calls invoke('auth_login') when email is submitted", async () => {
    mockInvoke.mockResolvedValueOnce({ status: "otp_sent", flow_id: "abc" });

    render(<Onboarding />);
    // Go to email step
    await userEvent.click(
      screen.getByRole("button", { name: /get started/i })
    );

    // Fill email and submit
    const emailInput = screen.getByPlaceholderText(/you@example\.com/i);
    await userEvent.type(emailInput, "test@example.com");
    await userEvent.click(
      screen.getByRole("button", { name: /send verification code/i })
    );

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("auth_login", {
        email: "test@example.com",
      });
    });

    // Should advance to OTP step
    await waitFor(() => {
      expect(screen.getByText(/check your email/i)).toBeInTheDocument();
    });
  });

  it("calls invoke('auth_verify') when OTP is submitted", async () => {
    // First call: auth_login
    mockInvoke.mockResolvedValueOnce({ status: "otp_sent", flow_id: "abc" });
    // Second call: auth_verify
    mockInvoke.mockResolvedValueOnce({ status: "verified" });
    // Third call: auth_status (optional, may fail)
    mockInvoke.mockRejectedValueOnce(new Error("not implemented"));

    render(<Onboarding />);

    // Go to email step
    await userEvent.click(
      screen.getByRole("button", { name: /get started/i })
    );

    // Submit email
    const emailInput = screen.getByPlaceholderText(/you@example\.com/i);
    await userEvent.type(emailInput, "test@example.com");
    await userEvent.click(
      screen.getByRole("button", { name: /send verification code/i })
    );

    // Wait for OTP step
    await waitFor(() => {
      expect(screen.getByText(/check your email/i)).toBeInTheDocument();
    });

    // Fill OTP digits
    const digitInputs = screen.getAllByRole("textbox");
    for (let i = 0; i < 6; i++) {
      await userEvent.type(digitInputs[i], String(i + 1));
    }

    // Click verify
    await userEvent.click(screen.getByRole("button", { name: /verify/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("auth_verify", {
        otp: "123456",
      });
    });

    // Should advance to Fund step
    await waitFor(() => {
      expect(screen.getByText(/fund your wallet/i)).toBeInTheDocument();
    });
  });

  it("shows error when auth_login fails", async () => {
    mockInvoke.mockRejectedValueOnce("Invalid email domain");

    render(<Onboarding />);

    // Go to email step
    await userEvent.click(
      screen.getByRole("button", { name: /get started/i })
    );

    // Submit email
    const emailInput = screen.getByPlaceholderText(/you@example\.com/i);
    await userEvent.type(emailInput, "test@example.com");
    await userEvent.click(
      screen.getByRole("button", { name: /send verification code/i })
    );

    // Should show error
    await waitFor(() => {
      expect(screen.getByText("Invalid email domain")).toBeInTheDocument();
    });
  });

  it("shows error when auth_verify fails", async () => {
    // auth_login succeeds
    mockInvoke.mockResolvedValueOnce({ status: "otp_sent", flow_id: "abc" });
    // auth_verify fails
    mockInvoke.mockRejectedValueOnce("Invalid OTP");

    render(<Onboarding />);

    // Navigate to OTP step
    await userEvent.click(
      screen.getByRole("button", { name: /get started/i })
    );
    const emailInput = screen.getByPlaceholderText(/you@example\.com/i);
    await userEvent.type(emailInput, "test@example.com");
    await userEvent.click(
      screen.getByRole("button", { name: /send verification code/i })
    );
    await waitFor(() => {
      expect(screen.getByText(/check your email/i)).toBeInTheDocument();
    });

    // Fill OTP and verify
    const digitInputs = screen.getAllByRole("textbox");
    for (let i = 0; i < 6; i++) {
      await userEvent.type(digitInputs[i], String(i + 1));
    }
    await userEvent.click(screen.getByRole("button", { name: /verify/i }));

    // Should show error
    await waitFor(() => {
      expect(screen.getByText("Invalid OTP")).toBeInTheDocument();
    });
  });
});
