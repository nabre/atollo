// The progress of a setup run: a component touched twice in one run (removed
// on another system then installed, two lines patched) is two steps.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { SAMPLE_SETUP, type SetupSnapshot, type SetupStep } from "@/ipc";
import { foldSetup, itemBusy, itemJob, itemState, SetupProgress, useStepsDone } from "@/components/setup";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const step = (name: string): SetupStep => ({ id: "mysql", name, note: null, done: null, size: null, outcome: null });

describe("SetupProgress", () => {
  it("lists two steps of the same component, each once", () => {
    const warn = vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(
      <SetupProgress
        snapshot={{ ...SAMPLE_SETUP, running: true, total: 2, steps: [step("MySQL 8.4"), step("MySQL 9.7")] }}
      />,
    );
    expect(screen.getByText("MySQL 8.4")).toBeTruthy();
    expect(screen.getByText("MySQL 9.7")).toBeTruthy();
    expect(warn.mock.calls.some((c) => String(c[0]).includes("same key"))).toBe(false);
  });
});

const done = (id: string): SetupStep => ({
  id,
  name: id,
  note: null,
  done: null,
  size: null,
  outcome: { kind: "installed", version: "1" },
});

describe("one item of the preparation (fluid installs, §8.6)", () => {
  const snapshot: SetupSnapshot = {
    ...SAMPLE_SETUP,
    running: true,
    job: "job-1",
    steps: [done("git"), { ...done("node"), outcome: null }],
    queue: [{ id: "php", name: "PHP" }],
    pending: [{ job: "job-2", title: "Installo Python", items: ["python"] }],
  };

  it("holds only the items being worked on, each with the job that cancels it", () => {
    expect(itemBusy(snapshot, "node")).toBe(true);
    expect(itemBusy(snapshot, "php")).toBe(true);
    expect(itemBusy(snapshot, "python")).toBe(true);
    expect(itemBusy(snapshot, "git")).toBe(false);
    expect(itemBusy(snapshot, "go")).toBe(false);
    expect(itemBusy({ ...snapshot, running: false }, "node")).toBe(false);
    expect(itemJob(snapshot, "node")).toBe("job-1");
    expect(itemJob(snapshot, "php")).toBe("job-1");
    expect(itemJob(snapshot, "python")).toBe("job-2");
    expect(itemJob(snapshot, "git")).toBeNull();
  });

  it("says «in coda» for an item asked again in a later run", () => {
    const again = { ...snapshot, pending: [{ job: "job-2", title: "Aggiorno git", items: ["git"] }] };
    expect(itemState(again, "git", "git")).toEqual({ kind: "queued" });
    expect(itemState(snapshot, "git", "git")).toBeNull();
  });

  it("tells each ended step once, and the end of the runs", () => {
    const seen: [string[], boolean][] = [];
    function Probe({ s }: { s: SetupSnapshot }) {
      useStepsDone(s, (ids, ended) => {
        seen.push([ids, ended]);
      });
      return null;
    }
    const first = { ...snapshot, steps: [{ ...done("git"), outcome: null }] };
    const { rerender } = render(<Probe s={first} />);
    rerender(<Probe s={{ ...first, steps: [done("git")] }} />);
    rerender(<Probe s={{ ...first, steps: [done("git"), { ...done("node"), outcome: null }] }} />);
    rerender(<Probe s={{ ...first, steps: [done("git"), done("node")] }} />);
    rerender(<Probe s={{ ...first, running: false, steps: [done("git"), done("node")] }} />);
    expect(seen).toEqual([
      [["git"], false],
      [["node"], false],
      [[], true],
    ]);
  });
});

describe("the preparation's snapshots folded in a frame", () => {
  it("counts each start and end of the runs, even those never seen running", () => {
    const on = { ...SAMPLE_SETUP, running: true };
    const off = { ...SAMPLE_SETUP, running: false };
    const fold = (...all: SetupSnapshot[]) => all.reduce(foldSetup, { snapshot: null, starts: 0, ends: 0 });
    expect(fold(off, on, off)).toMatchObject({ snapshot: off, starts: 1, ends: 1 });
    expect(fold(on, off, on, on)).toMatchObject({ snapshot: on, starts: 2, ends: 1 });
    expect(fold(off, off)).toMatchObject({ starts: 0, ends: 0 });
  });
});
