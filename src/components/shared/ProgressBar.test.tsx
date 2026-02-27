import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { ProgressBar } from "./ProgressBar";

describe("ProgressBar", () => {
  function getInnerBar(container: HTMLElement) {
    // The outer div is the track, the inner div is the fill bar
    const outer = container.firstElementChild as HTMLElement;
    return outer.firstElementChild as HTMLElement;
  }

  it("renders with value=50 and max=100", () => {
    const { container } = render(<ProgressBar value={50} max={100} />);
    const inner = getInnerBar(container);
    expect(inner.style.width).toBe("50%");
  });

  it("does not crash when max=0", () => {
    const { container } = render(<ProgressBar value={0} max={0} />);
    const inner = getInnerBar(container);
    expect(inner.style.width).toBe("0%");
  });

  it("caps at 100% when value exceeds max", () => {
    const { container } = render(<ProgressBar value={150} max={100} />);
    const inner = getInnerBar(container);
    expect(inner.style.width).toBe("100%");
  });
});
