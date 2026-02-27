import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { MonoAddress } from "./MonoAddress";

describe("MonoAddress", () => {
  const testAddress = "0x1234567890abcdef";

  it("truncates address by default", () => {
    render(<MonoAddress address={testAddress} />);
    expect(screen.getByText("0x1234...cdef")).toBeInTheDocument();
  });

  it("shows full address when full=true", () => {
    render(<MonoAddress address={testAddress} full />);
    expect(screen.getByText(testAddress)).toBeInTheDocument();
  });
});
