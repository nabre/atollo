// The table of every tool during an install: each percent of a download
// renders the row of the tool downloading, not the other rows.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { SAMPLE_SETUP, type ComponentInfo, type SetupSnapshot } from "@/ipc";
import { setupOf, ToolsTable } from "@/components/table";
import type { ComponentsState } from "@/components/state";
import type { KitVersions } from "@/versions";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

/** How many times each row drew its logo: once per render of the row. */
const drawn = vi.hoisted(() => new Map<string, number>());
vi.mock("@/shared/logos", async (original) => ({
  ...(await original<typeof import("@/shared/logos")>()),
  ComponentLogo: ({ id }: { id: string }) => {
    drawn.set(id, (drawn.get(id) ?? 0) + 1);
    return null;
  },
}));

afterEach(() => {
  cleanup();
  drawn.clear();
});

function tool(id: string, name: string): ComponentInfo {
  return {
    id,
    name,
    group: "languages",
    desc: name,
    mb: 100,
    chosen: false,
    installed: false,
    present: false,
    locked: false,
    version: null,
    available: true,
    unavailableReason: null,
    notNeeded: null,
    needs: [],
    versioned: false,
    home: "https://example.org",
    agent: null,
    leftover: false,
  };
}

const LIST = [tool("node", "Node.js"), tool("python", "Python"), tool("go", "Go")];

const STATE = {
  list: LIST,
  chosen: new Set<string>(),
  choose: () => undefined,
  neededBy: new Map<string, string[]>(),
  target: undefined,
  setTarget: () => undefined,
  checks: new Map(),
  checking: false,
  verifyTarget: () => undefined,
  grid: null,
  host: "win32-x64",
} as unknown as ComponentsState;

const KIT = {
  byId: new Map(),
  setTools: () => undefined,
  busy: null,
  error: null,
  last: null,
  run: () => undefined,
  offers: { checking: new Set<string>(), failed: {}, refresh: () => undefined },
  target: undefined,
} as unknown as KitVersions;

/** Node.js downloading at `done` bytes of 100, Python waiting. */
function installing(done: number): SetupSnapshot {
  return {
    ...SAMPLE_SETUP,
    running: true,
    total: 2,
    steps: [{ id: "node", name: "Node.js", note: null, done, size: 100, outcome: null }],
    queue: [{ id: "python", name: "Python" }],
  };
}

function table(setup: SetupSnapshot | null) {
  return <ToolsTable state={STATE} kit={KIT} setup={setup} onOpen={() => undefined} />;
}

describe("ToolsTable", () => {
  it("renders at each percent only the rows the install touches", () => {
    const { rerender } = render(table(installing(10)));
    expect(screen.getByText("download 10%")).toBeTruthy();
    const before = new Map(drawn);
    rerender(table(installing(20)));
    expect(screen.getByText("download 20%")).toBeTruthy();
    expect(drawn.get("node")).toBe((before.get("node") ?? 0) + 1);
    // Python waits in the queue: the snapshot is its too.
    expect(drawn.get("python")).toBe((before.get("python") ?? 0) + 1);
    expect(drawn.get("go")).toBe(before.get("go"));
  });

  it("gives a row the preparation only when it touches its tool", () => {
    const s = installing(10);
    expect(setupOf(s, "node")).toBe(s);
    expect(setupOf(s, "python")).toBe(s);
    expect(setupOf({ ...s, queue: [], pending: [{ job: "j", title: "Go", items: ["go"] }] }, "go")).not.toBeNull();
    expect(setupOf(s, "go")).toBeNull();
    expect(setupOf(null, "node")).toBeNull();
  });
});
