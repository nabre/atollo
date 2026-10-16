// The Server Web page (R3-4): two web actions of one project in the same
// pool are two VirtualHosts, each with its own row (key project:action).
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, screen } from "@testing-library/react";
import type { WebServersView } from "../../ui/src/api/servers";
import { ServersPage } from "../../ui/src/servers";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

const site = (action: string, folder: string) => ({
  project: "shop",
  action,
  folder,
  serverName: "shop.localhost",
  domains: [],
  aliases: {},
  url: "http://shop.localhost:8090",
});

const VIEW: WebServersView = {
  pools: [
    {
      id: "legacy",
      name: "Legacy",
      engine: "nginx",
      serverLine: "1.28",
      phpLine: null,
      port: 8090,
      running: false,
      pid: null,
      manual: true,
      sites: [site("site", "public"), site("docs", "docs")],
      confDir: "data\\servers\\legacy",
      orphan: false,
    },
  ],
  apacheLines: [],
  nginxLines: ["1.28"],
  phpLines: [],
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  invoke.mockReset();
  vi.restoreAllMocks();
});

describe("ServersPage", () => {
  it("shows every site of a project, one row per action, without duplicate keys", async () => {
    const errors = vi.spyOn(console, "error").mockImplementation(() => undefined);
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve(VIEW);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByText("public")).toBeTruthy();
    expect(screen.getByText("docs")).toBeTruthy();
    const keyWarnings = errors.mock.calls.filter((c) => c.some((a) => String(a).includes("same key")));
    expect(keyWarnings).toEqual([]);
  });

  it("marks an orphan pool, with no reload, still stoppable", async () => {
    const [legacy] = VIEW.pools;
    if (!legacy) throw new Error("fixture");
    const orphan = { ...legacy, id: "nginx-1.25", running: true, pid: 42, orphan: true };
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve({ ...VIEW, pools: [orphan] });
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByText(/non corrisponde più ai progetti/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Ricarica configurazione" })).toBeNull();
    expect(screen.getAllByRole("button", { name: "Ferma" }).length).toBeGreaterThan(0);
  });
});
