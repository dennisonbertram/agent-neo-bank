import { describe, it, expect } from "vitest";
import { screen } from "@testing-library/react";
import { Routes, Route } from "react-router-dom";
import { Shell } from "./Shell";
import { renderWithRouter } from "@/test/render";

describe("Shell", () => {
  it("renders sidebar", () => {
    renderWithRouter(
      <Routes>
        <Route element={<Shell />}>
          <Route path="/" element={<div>Dashboard Content</div>} />
        </Route>
      </Routes>
    );
    expect(screen.getByRole("navigation")).toBeInTheDocument();
  });

  it("renders header", () => {
    renderWithRouter(
      <Routes>
        <Route element={<Shell />}>
          <Route path="/" element={<div>Dashboard Content</div>} />
        </Route>
      </Routes>
    );
    expect(screen.getByRole("banner")).toBeInTheDocument();
  });

  it("renders content outlet", () => {
    renderWithRouter(
      <Routes>
        <Route element={<Shell />}>
          <Route path="/" element={<div>Dashboard Content</div>} />
        </Route>
      </Routes>
    );
    expect(screen.getByText("Dashboard Content")).toBeInTheDocument();
  });
});
