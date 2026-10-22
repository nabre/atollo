// The log of a project's action read again (the Activity panel's selection):
// the lines that come as events while it is read are not lost, nor doubled.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { mergeLog, useRuns } from "@/projects/runs";
import { invoke } from "@test/support/tauri";

const handlers = new Map<string, (e: { payload: unknown }) => void>();
vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));
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
    expect(mergeLog({ lines: ["a", "b"], seq: 2 }, [{ lines: ["c"], seq: 3 }])).toEqual(["a", "b", "c"]);
    // A batch half in the snapshot, half after it.
    expect(mergeLog({ lines: ["a", "b", "c"], seq: 3 }, [{ lines: ["b", "c", "d"], seq: 4 }])).toEqual([
      "a",
      "b",
      "c",
      "d",
    ]);
    expect(mergeLog({ lines: ["a", "b"], seq: 2 }, [])).toEqual(["a", "b"]);
    expect(mergeLog({ lines: [], seq: 0 }, [{ lines: ["a"], seq: 1 }])).toEqual(["a"]);
    // Numbered over all the actions: gaps are other actions' lines.
    expect(
      mergeLog({ lines: ["a"], seq: 40 }, [
        { lines: ["b"], seq: 37 },
        { lines: ["c"], seq: 52 },
      ]),
    ).toEqual(["a", "c"]);
  });

  it("keeps a new line equal to the last read (BUG-V7-30)", () => {
    expect(mergeLog({ lines: ["a", ""], seq: 2 }, [{ lines: [""], seq: 3 }])).toEqual(["a", "", ""]);
    expect(mergeLog({ lines: ["x", ""], seq: 2 }, [{ lines: [""], seq: 3 }])).toEqual(["x", "", ""]);
    expect(mergeLog({ lines: [".", "."], seq: 2 }, [{ lines: [".", "."], seq: 4 }])).toEqual([".", ".", ".", "."]);
    // The same line already read is not added twice.
    expect(mergeLog({ lines: ["a", ""], seq: 2 }, [{ lines: [""], seq: 2 }])).toEqual(["a", ""]);
  });

  it("puts the lines in their order when the events come out of it", () => {
    expect(
      mergeLog({ lines: ["a"], seq: 1 }, [
        { lines: ["c"], seq: 3 },
        { lines: ["b"], seq: 2 },
      ]),
    ).toEqual(["a", "b", "c"]);
  });
});

describe("useRuns", () => {
  it("keeps a line that comes while the log is read", async () => {
    let answer: (log: { lines: string[]; seq: number }) => void = () => undefined;
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
    // A line already in the snapshot, then two after it, one the same as
    // the snapshot's last.
    act(() => {
      const emit = (lines: string[], seq: number) =>
        handlers.get("run")?.({ payload: { kind: "log", key: "blog|dev", lines, seq } });
      emit([""], 2);
      emit(["", "ready"], 4);
    });
    await act(async () => {
      answer({ lines: ["start", ""], seq: 2 });
      await Promise.resolve();
    });
    expect(result.current.logs["blog|dev"]).toEqual(["start", "", "", "ready"]);
  });
});
