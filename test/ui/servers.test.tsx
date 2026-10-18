// The Server Web page (R3-4): two web actions of one project in the same
// pool are two VirtualHosts, each with its own row (key project:action).
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
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
      engine: "caddy",
      serverLine: "2.11",
      phpLine: null,
      port: 8090,
      running: false,
      pid: null,
      manual: true,
      sites: [site("site", "public"), site("docs", "docs")],
      confDir: "data\\servers\\legacy",
      orphan: false,
      profile: null,
    },
  ],
  apacheLines: ["2.4"],
  caddyLines: ["2.11", "2.10"],
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

  it("saves a profile's start options with the profile as it was saved", async () => {
    const [legacy] = VIEW.pools;
    if (!legacy) throw new Error("fixture");
    const profile = {
      id: "legacy",
      name: "Legacy",
      engine: "caddy" as const,
      serverLine: null,
      phpLine: null,
      port: 8090,
    };
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve({ ...VIEW, pools: [{ ...legacy, profile }] });
      if (cmd === "server_save") return Promise.resolve(null);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    const keep = screen.getAllByRole("checkbox", { name: "Mantieni attivo anche senza progetti" });
    // The profile's (the creation form is closed): checked, as for an older
    // servers.json with no option saved.
    expect(keep.map((c) => (c as HTMLInputElement).checked)).toEqual([true]);
    const auto = screen.getAllByRole("checkbox", { name: "Avvia con Atollo" });
    const last = auto[auto.length - 1];
    if (!last) throw new Error("no checkbox");
    await act(async () => {
      last.click();
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(invoke).toHaveBeenCalledWith("server_save", {
      profile: { ...profile, keepAlive: true, autostart: true },
    });
  });

  it("explains how an automatic pool starts and stops", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve(VIEW);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByText(/si ferma con l'ultimo/)).toBeTruthy();
  });

  it("marks a detached site, apart from the attached ones", async () => {
    const [legacy] = VIEW.pools;
    if (!legacy) throw new Error("fixture");
    const sites = [site("site", "public"), { ...site("docs", "docs"), detached: true }];
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve({ ...VIEW, pools: [{ ...legacy, sites }] });
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    const { container } = render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getAllByText("Sganciato: il progetto è fermo")).toHaveLength(1);
    const marked = container.querySelectorAll("[data-detached]");
    expect(marked).toHaveLength(1);
    expect(marked[0]?.textContent).toContain("docs");
  });

  it("marks an orphan pool, with no reload, still stoppable", async () => {
    const [legacy] = VIEW.pools;
    if (!legacy) throw new Error("fixture");
    const orphan = { ...legacy, id: "caddy-2.10", running: true, pid: 42, orphan: true };
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

  it("shows a Caddy pool as Caddy", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve(VIEW);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByText("Caddy")).toBeTruthy();
    expect(screen.queryByText("Apache httpd")).toBeNull();
  });

  it("creates a Caddy profile by default, with the lines of Caddy, and offers only Caddy and Apache", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve(VIEW);
      if (cmd === "server_save") return Promise.resolve(null);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Nuovo profilo server", expanded: false }));
      await vi.advanceTimersByTimeAsync(0);
    });
    const engine = document.getElementById("srv-new-engine") as HTMLElementTagNameMap["vscode-single-select"];
    const engines = [...engine.querySelectorAll("vscode-option")].map((o) => o.value);
    expect(engines).toEqual(["caddy", "apache"]);
    expect(engine.value).toBe("caddy");
    const line = document.getElementById("srv-new-sline") as HTMLElementTagNameMap["vscode-single-select"];
    const lines = [...line.querySelectorAll("vscode-option")].map((o) => o.value);
    expect(lines).toEqual(["", "2.11", "2.10"]);
    fireEvent.change(line, { target: { value: "2.11" } });
    fireEvent.change(document.getElementById("srv-new-name") as HTMLElement, { target: { value: "Shop" } });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Crea profilo" }));
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(invoke).toHaveBeenCalledWith("server_save", {
      profile: {
        id: "",
        name: "Shop",
        engine: "caddy",
        serverLine: "2.11",
        phpLine: null,
        port: null,
        keepAlive: true,
        autostart: false,
      },
    });
  });

  it("lists the lines of Apache when Apache is chosen", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve(VIEW);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ServersPage />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Nuovo profilo server", expanded: false }));
      await vi.advanceTimersByTimeAsync(0);
    });
    const engine = document.getElementById("srv-new-engine") as HTMLElementTagNameMap["vscode-single-select"];
    await act(async () => {
      fireEvent.change(engine, { target: { value: "apache" } });
      await vi.advanceTimersByTimeAsync(0);
    });
    const line = document.getElementById("srv-new-sline") as HTMLElementTagNameMap["vscode-single-select"];
    const lines = [...line.querySelectorAll("vscode-option")].map((o) => o.value);
    expect(lines).toEqual(["", "2.4"]);
  });
});
