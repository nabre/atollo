// The Activity panel (UX-V5-01): the versions of a database server that are
// not stopped (the main one or another one) and the web server pools on are
// listed, and "Ferma tutto", once confirmed, stops actions, lines and pools.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { RunInfo, ServiceInfo, WebServerPool } from "@/api";
import { ActivityPanel, liveLines, useRunning } from "@/shell/activity";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

const MYSQL: ServiceInfo = {
  id: "mysql",
  name: "MySQL",
  installed: true,
  state: "stopped",
  port: 3306,
  autostart: false,
  dataDir: "E:\\Atollo\\data\\db\\mysql\\9.7",
  log: "E:\\Atollo\\data\\app\\logs\\mysql.log",
  url: "mysql://root@127.0.0.1:3306/",
  user: "root",
  password: null,
  message: null,
  line: "9.7",
  lines: ["9.7", "8.4"],
  others: [
    {
      line: "8.4",
      state: "running",
      port: 3307,
      dataDir: "E:\\Atollo\\data\\db\\mysql\\8.4",
      log: "E:\\Atollo\\data\\app\\logs\\mysql-8.4.log",
      url: "mysql://root@127.0.0.1:3307/",
      message: null,
    },
  ],
};

const POOL: WebServerPool = {
  id: "caddy-2.11",
  name: "Caddy 2.11",
  engine: "caddy",
  serverLine: "2.11",
  phpLine: null,
  port: 8090,
  running: true,
  pid: 42,
  manual: false,
  sites: [],
  confDir: "data\\servers\\caddy-2.11",
  orphan: false,
  profile: null,
};

const RUN: RunInfo = {
  key: "blog|dev",
  project: "blog",
  action: "dev",
  label: "dev",
  type: "web",
  command: "npm run dev",
  port: 5173,
  url: null,
  state: "running",
  start: 1,
  end: null,
  code: null,
  pid: 7,
  skippedAliases: [],
};

function Harness({ runs, onStop }: { runs: Record<string, RunInfo>; onStop: (key: string) => Promise<void> }) {
  const running = useRunning(runs);
  return (
    <>
      <output aria-label="count">{running.count}</output>
      <ActivityPanel
        height={240}
        runs={runs}
        running={running}
        logs={{}}
        loadLog={() => undefined}
        onStop={onStop}
        onForget={() => undefined}
        setup={null}
        onClose={() => undefined}
      />
    </>
  );
}

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

describe("liveLines", () => {
  it("lists another version running while the main one is stopped", () => {
    expect(liveLines([MYSQL]).map((l) => [l.key, l.name, l.line, l.port])).toEqual([
      ["mysql:8.4", "MySQL 8.4", "8.4", 3307],
    ]);
    expect(liveLines([{ ...MYSQL, state: "running", others: [] }]).map((l) => [l.key, l.line])).toEqual([
      ["mysql", undefined],
    ]);
  });
});

describe("ActivityPanel", () => {
  it("lists a secondary database line and a pool, and stops everything once confirmed", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") return Promise.resolve([MYSQL]);
      if (cmd === "servers_list")
        return Promise.resolve({ pools: [POOL], apacheLines: [], caddyLines: [], phpLines: [] });
      if (cmd === "service_stop" || cmd === "server_stop") return Promise.resolve(null);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    const onStop = vi.fn<(key: string) => Promise<void>>(() => Promise.resolve());
    render(<Harness runs={{ [RUN.key]: RUN }} onStop={onStop} />);
    const list = within(screen.getByRole("list", { name: "Attività" }));
    expect(await list.findByText("MySQL 8.4")).toBeTruthy();
    expect(await list.findByText("Caddy 2.11")).toBeTruthy();
    expect(screen.getByLabelText("count").textContent).toBe("3");

    // The pool's details
    fireEvent.click(list.getByText("Caddy 2.11"));
    expect(screen.getByText("Caddy 2.11 · 127.0.0.1:8090")).toBeTruthy();

    // Nothing stops before the confirmation; Esc answers no
    fireEvent.click(screen.getByRole("button", { name: "Ferma tutto" }));
    expect(onStop).not.toHaveBeenCalled();
    fireEvent.keyDown(screen.getByRole("alertdialog"), { key: "Escape" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("service_stop", expect.anything());

    fireEvent.click(screen.getByRole("button", { name: "Ferma tutto" }));
    fireEvent.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Ferma tutto" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("service_stop", { id: "mysql", line: "8.4" });
    });
    expect(onStop).toHaveBeenCalledWith(RUN.key);
    expect(invoke).toHaveBeenCalledWith("server_stop", { id: "caddy-2.11" });
    expect(invoke).not.toHaveBeenCalledWith("service_stop", { id: "mysql" });
  });

  it("says why a stop failed, alone or among «Ferma tutto»", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") return Promise.resolve([]);
      if (cmd === "servers_list")
        return Promise.resolve({ pools: [POOL], apacheLines: [], caddyLines: [], phpLines: [] });
      if (cmd === "server_stop") return Promise.reject(new Error("port 8090 held"));
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    const onStop = vi.fn<(key: string) => Promise<void>>(() => Promise.reject(new Error("process in use")));
    render(<Harness runs={{ [RUN.key]: RUN }} onStop={onStop} />);
    const list = within(screen.getByRole("list", { name: "Attività" }));
    fireEvent.click(await list.findByText("blog: dev"));
    fireEvent.click(screen.getByRole("button", { name: "Ferma" }));
    expect(await screen.findByText(/process in use/)).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Ferma tutto" }));
    fireEvent.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Ferma tutto" }));
    expect(await screen.findByText(/port 8090 held/)).toBeTruthy();
    expect(screen.getByText(/process in use/)).toBeTruthy();
  });
});
