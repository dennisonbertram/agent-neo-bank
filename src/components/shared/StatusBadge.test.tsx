import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusBadge } from "./StatusBadge";

describe("StatusBadge", () => {
  it("renders active status with correct label", () => {
    render(<StatusBadge status="active" />);
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it("renders pending status with correct label", () => {
    render(<StatusBadge status="pending" />);
    expect(screen.getByText("Pending")).toBeInTheDocument();
  });

  it("renders suspended status with correct label", () => {
    render(<StatusBadge status="suspended" />);
    expect(screen.getByText("Suspended")).toBeInTheDocument();
  });

  it("renders revoked status with correct label", () => {
    render(<StatusBadge status="revoked" />);
    expect(screen.getByText("Revoked")).toBeInTheDocument();
  });
});
