// The log of a project's action read again (the Activity panel's selection):
// the lines that come as events while it is read are not lost, nor doubled.
// The output is kept apart from the app: the lines of a frame come in
// together, the last 3000 are kept, and the view redraws its last block.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, renderHook, screen, waitFor } from "@testing-library/react";
import { createRunLogs, mergeLog, useRuns } from "@/projects/runs";
import { RunOutputView } from "@/projects/output";
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
    expect(result.current.logs.get("blog|dev").lines).toEqual(["start", "", "", "ready"]);
    // The lines that waited for the frame were in the read: not added again.
    // The frame they would have come at, not a guess of how long it takes.
    await new Promise((resolve) => requestAnimationFrame(resolve));
    expect(result.current.logs.get("blog|dev").lines).toEqual(["start", "", "", "ready"]);
  });

  it("adds the lines of the events at the next frame, together", async () => {
    invoke.mockResolvedValue([]);
    const { result } = renderHook(useRuns);
    await act(async () => {
      await Promise.resolve();
    });
    const told = vi.fn();
    result.current.logs.subscribe("blog|dev", told);
    act(() => {
      const emit = (lines: string[], seq: number) =>
        handlers.get("run")?.({ payload: { kind: "log", key: "blog|dev", lines, seq } });
      emit(["a"], 1);
      emit(["b", "c"], 3);
    });
    expect(result.current.logs.get("blog|dev").lines).toEqual([]);
    await waitFor(() => {
      expect(result.current.logs.get("blog|dev").lines).toEqual(["a", "b", "c"]);
    });
    expect(told).toHaveBeenCalledTimes(1);
  });
});

describe("createRunLogs", () => {
  it("keeps the last 3000 lines and counts the ones dropped", async () => {
    const logs = createRunLogs({ k: ["0"] });
    logs.append(
      "k",
      Array.from({ length: 3001 }, (_, i) => String(i + 1)),
    );
    await waitFor(() => {
      expect(logs.get("k").first).toBe(2);
    });
    expect(logs.get("k").lines.length).toBe(3000);
    expect(logs.get("k").lines[0]).toBe("2");
    expect(logs.get("k").lines.at(-1)).toBe("3001");
  });
});

describe("RunOutputView", () => {
  it("shows the lines as one text, redrawing only the last block when one is added", async () => {
    const logs = createRunLogs({ k: Array.from({ length: 150 }, (_, i) => `l${String(i)}`) });
    render(<RunOutputView logs={logs} runKey="k" label="Output di k" className="" />);
    const pre = screen.getByLabelText("Output di k");
    expect(pre.textContent).toBe(Array.from({ length: 150 }, (_, i) => `l${String(i)}`).join("\n"));
    const firstBlock = pre.firstChild;
    act(() => {
      logs.append("k", ["new"]);
    });
    await waitFor(() => {
      expect(pre.textContent.endsWith("l149\nnew")).toBe(true);
    });
    expect(pre.firstChild).toBe(firstBlock);
  });
});
