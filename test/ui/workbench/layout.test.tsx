// The sizes dragged with a sash: clamped at once, written to the browser's
// storage once the drag pauses, not at every pointer move.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { useStoredSize } from "../../../ui/src/workbench/layout";

beforeEach(() => {
  vi.useFakeTimers();
  localStorage.clear();
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

describe("useStoredSize", () => {
  it("writes three quick changes once, after the pause", () => {
    const write = vi.spyOn(Storage.prototype, "setItem");
    const { result } = renderHook(() => useStoredSize("side", 300, 170, 640));
    act(() => {
      result.current[1](310);
    });
    act(() => {
      result.current[1]((v) => v + 5);
    });
    act(() => {
      result.current[1]((v) => v + 5);
    });
    expect(result.current[0]).toBe(320);
    expect(write).not.toHaveBeenCalled();
    act(() => {
      vi.advanceTimersByTime(150);
    });
    expect(write).toHaveBeenCalledTimes(1);
    expect(write).toHaveBeenCalledWith("size:side", "320");
  });

  it("clamps the size to its min and max", () => {
    const { result } = renderHook(() => useStoredSize("panel", 240, 120, 600));
    act(() => {
      result.current[1](10_000);
    });
    expect(result.current[0]).toBe(600);
    act(() => {
      result.current[1]((v) => v - 10_000);
    });
    expect(result.current[0]).toBe(120);
    act(() => {
      vi.advanceTimersByTime(150);
    });
    expect(localStorage.getItem("size:panel")).toBe("120");
  });

  it("clamps a stored size out of range", () => {
    localStorage.setItem("size:panel", "5000");
    const { result } = renderHook(() => useStoredSize("panel", 240, 120, 600));
    expect(result.current[0]).toBe(600);
  });

  it("writes the last size when it goes away before the pause", () => {
    const { result, unmount } = renderHook(() => useStoredSize("side", 300, 170, 640));
    act(() => {
      result.current[1](400);
    });
    unmount();
    expect(localStorage.getItem("size:side")).toBe("400");
  });

  it("still changes the size when the storage refuses it", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("full");
    });
    const { result } = renderHook(() => useStoredSize("side", 300, 170, 640));
    act(() => {
      result.current[1](350);
    });
    expect(() => {
      act(() => {
        vi.advanceTimersByTime(150);
      });
    }).not.toThrow();
    expect(result.current[0]).toBe(350);
  });
});
