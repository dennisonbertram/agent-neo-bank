import { describe, it, expect } from "vitest";
import { screen } from "@testing-library/react";
import { Sidebar } from "./Sidebar";
import { renderWithRouter } from "@/test/render";

describe("Sidebar", () => {
  it("renders all 6 nav links", () => {
    renderWithRouter(<Sidebar />);
    expect(screen.getByRole("link", { name: /dashboard/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /agents/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /transactions/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /approvals/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /fund/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /settings/i })).toBeInTheDocument();
  });

  it("renders logo text", () => {
    renderWithRouter(<Sidebar />);
    expect(screen.getByText("Agent Neo Bank")).toBeInTheDocument();
  });

  it("renders user profile section", () => {
    renderWithRouter(<Sidebar />);
    expect(screen.getByText("Wallet")).toBeInTheDocument();
    expect(screen.getByText("Connected")).toBeInTheDocument();
  });

  it("highlights active route", () => {
    renderWithRouter(<Sidebar />, { route: "/" });
    const dashboardLink = screen.getByRole("link", { name: /dashboard/i });
    expect(dashboardLink.className).toMatch(/bg-\[#EEF2FF\]/);
  });
});
