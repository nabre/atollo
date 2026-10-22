// The log of a project's action read again (the Activity panel's selection):
// the lines that come as events while it is read are not lost, nor doubled.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { mergeLog, useRuns } from "../../../ui/src/projects/runs";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
const handlers = new Map<string, (e: { payload: unknown }) => void>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, h: (e: { payload: unknown }) => void) => {
    handlers.set(event, h);
    return Promise.resolve(() => handlers.delete(event));
  },
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  handlers.clear();
});

describe("mergeLog", () => {
  it("adds the lines that came after the snapshot, once", () => {
    expect(mergeLog(["a", "b"], ["c"])).toEqual(["a", "b", "c"]);
    expect(mergeLog(["a", "b", "c"], ["b", "c", "d"])).toEqual(["a", "b", "c", "d"]);
    expect(mergeLog(["a", "b"], [])).toEqual(["a", "b"]);
    expect(mergeLog([], ["a"])).toEqual(["a"]);
  });
});

describe("useRuns", () => {
  it("keeps a line that comes while the log is read", async () => {
    let answer: (lines: string[]) => void = () => undefined;
    invoke.mockImplementation((cmd) => {
      if (cmd === "run_log")
        return new Promise((resolve) => {
          answer = resolve;
        });
      return Promise.resolve([]);
    });
    const { result } = renderHook(useRuns);
    await act(async () => {
      await Promise.resolve();
    });
    act(() => {
      result.current.loadLog("blog|dev");
    });
    // A line after the snapshot, which the answer does not hold.
    act(() => {
      handlers.get("run")?.({ payload: { kind: "log", key: "blog|dev", lines: ["ready"] } });
    });
    await act(async () => {
      answer(["start"]);
      await Promise.resolve();
    });
    expect(result.current.logs["blog|dev"]).toEqual(["start", "ready"]);
  });
});
