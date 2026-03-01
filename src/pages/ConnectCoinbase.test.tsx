import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import ConnectCoinbase from "./ConnectCoinbase";
import * as tauriLib from "../lib/tauri";
import { useAuthStore } from "../stores/authStore";

// Mock navigate
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

// Spy on isTauri and tauriApi.auth.login
const isTauriSpy = vi.spyOn(tauriLib, "isTauri");
const loginSpy = vi.spyOn(tauriLib.tauriApi.auth, "login");

function renderConnectCoinbase() {
  return render(
    <MemoryRouter>
      <ConnectCoinbase />
    </MemoryRouter>,
  );
}

describe("ConnectCoinbase", () => {
  beforeEach(() => {
    isTauriSpy.mockReturnValue(true);
    mockNavigate.mockClear();
    loginSpy.mockReset();
    // Reset auth store
    useAuthStore.setState({
      isAuthenticated: false,
      email: null,
      flowId: null,
    });
  });

  afterEach(() => {
    cleanup();
  });

  it("already_authenticated skips OTP and navigates to /home", async () => {
    const user = userEvent.setup();
    loginSpy.mockResolvedValue({ status: "already_authenticated" });

    renderConnectCoinbase();

    const emailInput = screen.getByPlaceholderText("name@email.com");
    await user.type(emailInput, "test@example.com");

    const sendButton = screen.getByRole("button", { name: /send code/i });
    await user.click(sendButton);

    // Wait for async handler to complete
    await vi.waitFor(() => {
      expect(loginSpy).toHaveBeenCalledWith("test@example.com");
    });

    // Should have set authenticated in store
    const state = useAuthStore.getState();
    expect(state.isAuthenticated).toBe(true);
    expect(state.email).toBe("test@example.com");

    // Should navigate to /home, NOT /setup/verify
    expect(mockNavigate).toHaveBeenCalledWith("/home");
    expect(mockNavigate).not.toHaveBeenCalledWith("/setup/verify", expect.anything());
  });

  it("normal login navigates to /setup/verify and sets flowId", async () => {
    const user = userEvent.setup();
    loginSpy.mockResolvedValue({ status: "otp_sent", flow_id: "flow-123" });

    renderConnectCoinbase();

    const emailInput = screen.getByPlaceholderText("name@email.com");
    await user.type(emailInput, "user@example.com");

    const sendButton = screen.getByRole("button", { name: /send code/i });
    await user.click(sendButton);

    await vi.waitFor(() => {
      expect(loginSpy).toHaveBeenCalledWith("user@example.com");
    });

    // Should have set flowId in store
    const state = useAuthStore.getState();
    expect(state.flowId).toBe("flow-123");

    // Should navigate to /setup/verify
    expect(mockNavigate).toHaveBeenCalledWith("/setup/verify", {
      state: { email: "user@example.com" },
    });

    // Should NOT navigate to /home
    expect(mockNavigate).not.toHaveBeenCalledWith("/home");
  });
});
