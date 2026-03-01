import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, cleanup, act } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import Home from "./Home";
import * as tauriLib from "../lib/tauri";

// Spy on safeTauriCall to track calls and return controlled values
const safeTauriCallSpy = vi.spyOn(tauriLib, "safeTauriCall");

const mockBalance = {
  balance: "$100.00",
  asset: "USD",
  balances: {
    ETH: { raw: "50000000000000000", formatted: "0.05", decimals: 18 },
    USDC: { raw: "10000000", formatted: "10.00", decimals: 6 },
  },
  balance_visible: true,
  cached: false,
};

function renderHome() {
  return render(
    <MemoryRouter>
      <Home />
    </MemoryRouter>,
  );
}

function setupSafeTauriCallMock() {
  safeTauriCallSpy.mockImplementation(async (fn, fallback) => {
    const fnStr = fn.toString();
    if (fnStr.includes("getBalance")) {
      return mockBalance;
    }
    if (fnStr.includes("getAddress")) {
      return { address: "0x1234567890abcdef1234567890abcdef12345678" };
    }
    if (fnStr.includes("transactions")) {
      return { transactions: [], total: 0 };
    }
    if (fnStr.includes("list") || fnStr.includes("agents")) {
      return [];
    }
    if (fnStr.includes("budget") || fnStr.includes("Summaries")) {
      return [];
    }
    return fallback;
  });
}

function countGetBalanceCalls() {
  return safeTauriCallSpy.mock.calls.filter(
    (c) => c[0].toString().includes("getBalance"),
  ).length;
}

/** Advance timers enough to flush the initial load without triggering infinite loop from setInterval */
async function flushInitialLoad() {
  // Advance by a small amount to flush pending promises/microtasks, not enough to trigger the 15s poll
  await act(async () => {
    await vi.advanceTimersByTimeAsync(100);
  });
}

describe("Home page", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    setupSafeTauriCallMock();
  });

  afterEach(() => {
    cleanup();
    vi.useRealTimers();
    safeTauriCallSpy.mockReset();
  });

  it("renders without errors", async () => {
    renderHome();
    await flushInitialLoad();
    expect(screen.getByText("Base Network Balance")).toBeInTheDocument();
  });

  it("balance polls at interval", async () => {
    renderHome();
    await flushInitialLoad();

    const initialCallCount = countGetBalanceCalls();
    expect(initialCallCount).toBe(1);

    // Advance by 15 seconds to trigger the first poll
    await act(async () => {
      await vi.advanceTimersByTimeAsync(15_000);
    });

    expect(countGetBalanceCalls()).toBe(2);

    // Advance by another 15 seconds
    await act(async () => {
      await vi.advanceTimersByTimeAsync(15_000);
    });

    expect(countGetBalanceCalls()).toBe(3);
  });

  it("balance polling cleanup on unmount", async () => {
    const clearIntervalSpy = vi.spyOn(globalThis, "clearInterval");

    const { unmount } = renderHome();
    await flushInitialLoad();

    const callsBefore = clearIntervalSpy.mock.calls.length;
    unmount();
    const callsAfter = clearIntervalSpy.mock.calls.length;

    // At least one clearInterval should have been called (for the polling useEffect)
    expect(callsAfter).toBeGreaterThan(callsBefore);

    clearIntervalSpy.mockRestore();
  });
});
