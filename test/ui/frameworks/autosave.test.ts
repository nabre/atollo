// The frameworks of the new-project wizard saved as they change (D-137):
// a moment after the last edit, or when Settings closes before that.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import type { FrameworkPiece, FrameworksView } from "@/api";
import { useFrameworks, useFrameworksAutosave } from "@/frameworks";
import { useAction } from "@/shared/hooks";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  vi.useRealTimers();
});

const piece = (name: string): FrameworkPiece => ({ id: "vite", kind: "base", name, desc: "", home: "" });
const VIEW: FrameworksView = { pieces: [piece("Vite")], official: [piece("Vite")], texts: {}, problem: null };

/** The names in each frameworks_set, in order. */
function namesSent(): string[] {
  return invoke.mock.calls
    .filter(([cmd]) => cmd === "frameworks_set")
    .map(([, a]) => (a as { pieces: FrameworkPiece[] }).pieces.map((p) => p.name).join());
}

function useSettingsFrameworks() {
  const fw = useFrameworks();
  const { run } = useAction();
  useFrameworksAutosave(fw, run);
  return fw;
}

async function mounted() {
  invoke.mockImplementation((cmd) => Promise.resolve(cmd === "frameworks_get" ? VIEW : undefined));
  const hook = renderHook(useSettingsFrameworks);
  act(() => {
    hook.result.current.load();
  });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(0);
  });
  return hook;
}

describe("useFrameworksAutosave", () => {
  it("saves the last edit a moment after it", async () => {
    vi.useFakeTimers();
    const { result } = await mounted();
    act(() => {
      result.current.change([piece("A")]);
    });
    act(() => {
      result.current.change([piece("AB")]);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(namesSent()).toEqual(["AB"]);
  });

  it("saves the latest edit when the page closes within the wait", async () => {
    vi.useFakeTimers();
    const { result, unmount } = await mounted();
    act(() => {
      result.current.change([piece("A")]);
    });
    act(() => {
      result.current.change([piece("AB")]);
    });
    unmount();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(namesSent()).toEqual(["AB"]);
  });

  it("runs one save at a time, the latest edit last", async () => {
    vi.useFakeTimers();
    const { result } = await mounted();
    let finish: (v?: unknown) => void = () => undefined;
    invoke.mockImplementation((cmd) => {
      if (cmd === "frameworks_get") return Promise.resolve(VIEW);
      if (namesSent().length === 1)
        return new Promise((resolve) => {
          finish = resolve;
        });
      return Promise.resolve();
    });
    act(() => {
      result.current.change([piece("A")]);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    act(() => {
      result.current.change([piece("AB")]);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    // The second save waits for the first, still running.
    expect(namesSent()).toEqual(["A"]);
    await act(async () => {
      finish(undefined);
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(namesSent()).toEqual(["A", "AB"]);
  });

  it("keeps an edit made while the reload after a save runs", async () => {
    vi.useFakeTimers();
    const { result } = await mounted();
    let answer: (v: FrameworksView) => void = () => undefined;
    invoke.mockImplementation((cmd) => {
      if (cmd === "frameworks_get")
        return new Promise((resolve) => {
          answer = resolve;
        });
      return Promise.resolve();
    });
    act(() => {
      result.current.change([piece("A")]);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    // The save is done and its reload waits: the user types on.
    act(() => {
      result.current.change([piece("AB")]);
    });
    await act(async () => {
      answer({ ...VIEW, pieces: [piece("A")] });
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(result.current.pieces.map((p) => p.name)).toEqual(["AB"]);
    expect(result.current.dirty).toBe(true);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(namesSent()).toEqual(["A", "AB"]);
  });

  it("saves nothing when the page closes with nothing changed", async () => {
    vi.useFakeTimers();
    const { unmount } = await mounted();
    unmount();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(namesSent()).toEqual([]);
  });
});
