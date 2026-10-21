// The frameworks of the new-project wizard saved as they change (D-137):
// a moment after the last edit, or when Settings closes before that.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import type { FrameworkPiece, FrameworksView } from "../../../ui/src/api";
import { useFrameworks, useFrameworksAutosave } from "../../../ui/src/frameworks";
import { useAction } from "../../../ui/src/hooks";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

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
