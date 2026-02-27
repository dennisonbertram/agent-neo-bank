import { describe, it, expect } from "vitest";
import { screen } from "@testing-library/react";
import { Sidebar } from "./Sidebar";
import { renderWithRouter } from "@/test/render";

describe("Sidebar", () => {
  it("renders nav links for Dashboard, Agents, Transactions, Settings", () => {
    renderWithRouter(<Sidebar />);
    expect(screen.getByRole("link", { name: /dashboard/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /agents/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /transactions/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /settings/i })).toBeInTheDocument();
  });

  it("highlights active route", () => {
    renderWithRouter(<Sidebar />, { route: "/" });
    const dashboardLink = screen.getByRole("link", { name: /dashboard/i });
    expect(dashboardLink.className).toMatch(/bg-accent/);
  });
});
