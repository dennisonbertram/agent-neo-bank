import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { GradientCard } from "./GradientCard";

describe("GradientCard", () => {
  it("renders children inside the card", () => {
    render(
      <GradientCard>
        <span>Card Content</span>
      </GradientCard>
    );
    expect(screen.getByText("Card Content")).toBeInTheDocument();
  });
});
