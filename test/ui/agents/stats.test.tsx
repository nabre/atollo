// The agents' usage, read again every two minutes: never while the window
// is hidden, and at once when it shows again.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { useStats } from "@/agents/stats";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

/** The window minimized (true) or shown again, as the WebView tells it. */
function setHidden(hidden: boolean) {
  Object.defineProperty(document, "hidden", { configurable: true, get: () => hidden });
  act(() => {
    document.dispatchEvent(new Event("visibilitychange"));
  });
}

afterEach(() => {
  cleanup();
  invoke.mockReset();
  vi.useRealTimers();
  setHidden(false);
});

describe("useStats", () => {
  it("reads nothing while the window is hidden, and reads at once when it shows", () => {
    vi.useFakeTimers();
    invoke.mockReturnValue(new Promise(() => undefined));
    const reads = () => invoke.mock.calls.filter(([c]) => c === "agents_stats").length;
    const { unmount } = renderHook(useStats);
    expect(reads()).toBe(1);
    setHidden(true);
    act(() => {
      vi.advanceTimersByTime(10 * 60 * 1000);
    });
    expect(reads()).toBe(1);
    setHidden(false);
    expect(reads()).toBe(2);
    act(() => {
      vi.advanceTimersByTime(2 * 60 * 1000);
    });
    expect(reads()).toBe(3);
    unmount();
  });
});
