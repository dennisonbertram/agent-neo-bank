import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InvitationCodes } from "./InvitationCodes";
import { mockInvoke } from "@/test/helpers";
import { renderWithRouter } from "@/test/render";
import type { InvitationCode } from "@/types";

function createMockInvitationCode(
  overrides: Partial<InvitationCode> = {}
): InvitationCode {
  return {
    code: "INV-abc123",
    created_at: 1740700800,
    expires_at: null,
    used_by: null,
    used_at: null,
    max_uses: 1,
    use_count: 0,
    label: "Test Code",
    ...overrides,
  };
}

describe("InvitationCodes", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders invitation codes table with data", async () => {
    const code1 = createMockInvitationCode({
      code: "INV-001",
      label: "Alpha Invite",
      use_count: 0,
      max_uses: 1,
    });
    const code2 = createMockInvitationCode({
      code: "INV-002",
      label: "Beta Invite",
      use_count: 0,
      max_uses: 5,
    });

    await mockInvoke({
      list_invitation_codes: [code1, code2],
    });

    renderWithRouter(<InvitationCodes />);

    // Table headers
    expect(await screen.findByText("Code")).toBeInTheDocument();
    expect(screen.getByText("Label")).toBeInTheDocument();
    expect(screen.getByText("Status")).toBeInTheDocument();
    expect(screen.getByText("Uses")).toBeInTheDocument();
    expect(screen.getByText("Created")).toBeInTheDocument();

    // Code data
    expect(screen.getByText("INV-001")).toBeInTheDocument();
    expect(screen.getByText("INV-002")).toBeInTheDocument();
    expect(screen.getByText("Alpha Invite")).toBeInTheDocument();
    expect(screen.getByText("Beta Invite")).toBeInTheDocument();
    expect(screen.getByText("0 / 1")).toBeInTheDocument();
    expect(screen.getByText("0 / 5")).toBeInTheDocument();
  });

  it("shows empty state when no codes", async () => {
    await mockInvoke({
      list_invitation_codes: [],
    });

    renderWithRouter(<InvitationCodes />);

    expect(
      await screen.findByText("No invitation codes")
    ).toBeInTheDocument();
  });

  it("generate code dialog opens and submits", async () => {
    const newCode = createMockInvitationCode({
      code: "INV-new",
      label: "New Code",
    });

    const invoker = await mockInvoke({
      list_invitation_codes: [],
      generate_invitation_code: newCode,
    });

    renderWithRouter(<InvitationCodes />);

    // Wait for initial load
    await screen.findByText("No invitation codes");

    const user = userEvent.setup();

    // Open dialog
    const generateButton = screen.getByRole("button", {
      name: /generate code/i,
    });
    await user.click(generateButton);

    // Fill in label
    const labelInput = screen.getByPlaceholderText(/label/i);
    await user.type(labelInput, "New Code");

    // Submit
    const submitButton = screen.getByRole("button", { name: /^generate$/i });
    await user.click(submitButton);

    // Verify invoke was called
    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith(
        "generate_invitation_code",
        expect.objectContaining({ label: "New Code" })
      );
    });
  });

  it("shows correct status badges", async () => {
    const now = Date.now() / 1000;

    const activeCode = createMockInvitationCode({
      code: "INV-active",
      label: "Active",
      use_count: 0,
      max_uses: 1,
      expires_at: null,
    });

    const usedCode = createMockInvitationCode({
      code: "INV-used",
      label: "Used",
      use_count: 1,
      max_uses: 1,
      used_by: "agent-1",
      used_at: now - 3600,
    });

    const expiredCode = createMockInvitationCode({
      code: "INV-expired",
      label: "Expired",
      use_count: 0,
      max_uses: 1,
      expires_at: now - 86400, // expired yesterday
    });

    await mockInvoke({
      list_invitation_codes: [activeCode, usedCode, expiredCode],
    });

    renderWithRouter(<InvitationCodes />);

    // Wait for data
    await screen.findByText("INV-active");

    // Check status badges
    const activeBadge = screen.getByTestId("status-badge-INV-active");
    expect(activeBadge).toHaveTextContent("Active");
    expect(activeBadge).toHaveClass("bg-green-100");

    const usedBadge = screen.getByTestId("status-badge-INV-used");
    expect(usedBadge).toHaveTextContent("Used");
    expect(usedBadge).toHaveClass("bg-gray-100");

    const expiredBadge = screen.getByTestId("status-badge-INV-expired");
    expect(expiredBadge).toHaveTextContent("Expired");
    expect(expiredBadge).toHaveClass("bg-red-100");
  });

  it("revoke button calls revoke command", async () => {
    const activeCode = createMockInvitationCode({
      code: "INV-revokeme",
      label: "Revoke Me",
      use_count: 0,
      max_uses: 1,
    });

    const invoker = await mockInvoke({
      list_invitation_codes: [activeCode],
      revoke_invitation_code: undefined,
    });

    renderWithRouter(<InvitationCodes />);

    // Wait for data
    await screen.findByText("INV-revokeme");

    const user = userEvent.setup();
    // First click shows confirmation
    const revokeButton = screen.getByRole("button", { name: /revoke/i });
    await user.click(revokeButton);
    // Second click confirms
    await user.click(screen.getByRole("button", { name: /confirm/i }));

    await waitFor(() => {
      expect(invoker).toHaveBeenCalledWith("revoke_invitation_code", {
        code: "INV-revokeme",
      });
    });
  });
});
