// The sizes dragged with a sash: clamped at once, written to the browser's
// storage once the drag pauses, not at every pointer move. The panel's tabs,
// measured as VSCodium's.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { Panel, useStoredSize } from "../../../ui/src/workbench/layout";

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

describe("Panel", () => {
  it("draws its tabs as VSCodium's modern ones, with the size tokens", () => {
    const onSelect = vi.fn();
    render(
      <Panel
        tabs={[
          { id: "activity", label: "Attività" },
          { id: "output", label: "Output" },
        ]}
        active="activity"
        onSelect={onSelect}
        height={200}
      >
        <p>content</p>
      </Panel>,
    );
    const classes = (e: Element | null) => (e?.getAttribute("class") ?? "").split(" ");
    const list = screen.getByRole("tablist");
    expect(classes(list.parentElement)).toEqual(
      expect.arrayContaining(["h-vsc-title", "pl-vsc-size20", "pr-vsc-size40"]),
    );
    const on = screen.getByRole("tab", { name: "Attività" });
    const off = screen.getByRole("tab", { name: "Output" });
    expect(on.getAttribute("aria-selected")).toBe("true");
    // 10px beside the label; the fill a 24px pill 2px in from the sides.
    for (const tab of [on, off]) {
      expect(classes(tab)).toEqual(
        expect.arrayContaining([
          "px-vsc-size100",
          "before:inset-x-vsc-size20",
          "before:inset-y-vsc-size40",
          "before:rounded-vsc-small",
        ]),
      );
    }
    expect(classes(on)).toContain("before:bg-vsc-list-inactive");
    expect(classes(off)).toContain("hover:before:bg-vsc-list-hover");
    // No underline: the modern tabs drop the panelTitle.activeBorder.
    expect(classes(on).some((c) => c.startsWith("after:"))).toBe(false);
    fireEvent.click(off);
    expect(onSelect).toHaveBeenCalledWith("output");
  });
});
