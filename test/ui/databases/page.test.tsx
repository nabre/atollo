// The Database page (N-U1): the list read every 3 s and the actions keep
// their errors apart, and a read still running when an action answers does
// not bring back the old list.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { SAMPLE_SERVICES, type ServiceInfo } from "../../../ui/src/api";
import { DatabasesPage } from "../../../ui/src/databases";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

/** Both servers stopped: no database browser to read. */
const STOPPED: ServiceInfo[] = SAMPLE_SERVICES.map((s) => ({ ...s, state: "stopped" }));
const STARTED: ServiceInfo[] = STOPPED.map((s, i) => (i === 0 ? { ...s, state: "running" } : s));

function deferred<T>() {
  let resolve: (v: T) => void = () => undefined;
  let reject: (e: unknown) => void = () => undefined;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

async function flush(ms = 0) {
  await act(async () => {
    await vi.advanceTimersByTimeAsync(ms);
  });
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  invoke.mockReset();
});

describe("DatabasesPage", () => {
  it("keeps a failed action's error through the next reads of the list", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") return Promise.resolve(STOPPED);
      if (cmd === "service_start") return Promise.reject(new Error("porta occupata"));
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    const [start] = screen.getAllByRole("button", { name: "Avvia" });
    fireEvent.click(start as HTMLElement);
    await flush();
    expect(screen.getByRole("alert").textContent).toContain("porta occupata");
    await flush(3000);
    await flush(3000);
    expect(invoke.mock.calls.filter(([c]) => c === "services_list").length).toBeGreaterThanOrEqual(3);
    expect(screen.getByRole("alert").textContent).toContain("porta occupata");
  });

  it("shows a failed read apart, and clears it when a read works", async () => {
    let fail = true;
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") return fail ? Promise.reject(new Error("letto male")) : Promise.resolve(STOPPED);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    expect(screen.getByRole("alert").textContent).toContain("letto male");
    fail = false;
    await flush(3000);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("drops a read that was running when the action answered", async () => {
    const slow = deferred<ServiceInfo[]>();
    let reads = 0;
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") {
        reads += 1;
        // The second read (the poll at 3 s) answers late, with the old list.
        if (reads === 2) return slow.promise;
        return Promise.resolve(reads === 1 ? STOPPED : STARTED);
      }
      if (cmd === "service_start") return Promise.resolve(STARTED);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    await flush(3000);
    expect(reads).toBe(2);
    const [start] = screen.getAllByRole("button", { name: "Avvia" });
    fireEvent.click(start as HTMLElement);
    await flush();
    expect(screen.getAllByRole("button", { name: "Ferma" }).length).toBeGreaterThan(0);
    slow.resolve(STOPPED);
    await flush();
    expect(screen.getAllByRole("button", { name: "Ferma" }).length).toBeGreaterThan(0);
  });

  it("starts another version on its own port and sets that port (PLAN.md 13.2)", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") return Promise.resolve(STOPPED);
      if (cmd === "service_start" || cmd === "service_line_port") return Promise.resolve(STOPPED);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    expect(screen.getByText("Altre versioni")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Avvia 11.4" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("service_start", { id: "mariadb", line: "11.4" });
    const port = document.getElementById("port-mariadb-11.4") as HTMLInputElement;
    expect(port.value).toBe("3307");
    fireEvent.change(port, { target: { value: "3317" } });
    fireEvent.blur(port);
    await flush();
    expect(invoke).toHaveBeenCalledWith("service_line_port", { id: "mariadb", line: "11.4", port: 3317 });
  });

  it("opens Adminer on another version", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list" || cmd === "db_admin_open") return Promise.resolve(STOPPED);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    fireEvent.click(screen.getByRole("button", { name: "Adminer su 11.4" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_admin_open", { line: "11.4" });
    fireEvent.click(screen.getByRole("button", { name: "Apri Adminer" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_admin_open", { line: null });
  });

  it("browses the version chosen, each on its own port", async () => {
    const mongo: ServiceInfo = {
      ...(STOPPED[1] as ServiceInfo),
      installed: true,
      state: "running",
      line: "8.0",
      lines: ["8.0", "7.0"],
      others: [
        {
          line: "7.0",
          state: "running",
          port: 27018,
          dataDir: "E:\\Atollo\\data\\db\\mongodb\\7.0",
          log: "E:\\Atollo\\data\\app\\logs\\mongodb-7.0.log",
          url: "mongodb://admin@127.0.0.1:27018/?authSource=admin",
          message: null,
        },
      ],
    };
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") return Promise.resolve([STOPPED[0], mongo]);
      if (cmd === "db_browse_tree") return Promise.resolve([]);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    fireEvent.click(screen.getByRole("button", { name: /^MongoDB/ }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_browse_tree", { engine: "mongodb", line: null });
    const version = document.getElementById("browse-line-mongodb") as HTMLElementTagNameMap["vscode-single-select"];
    expect(version.querySelectorAll("vscode-option")).toHaveLength(2);
    fireEvent.change(version, { target: { value: "7.0" } });
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_browse_tree", { engine: "mongodb", line: "7.0" });
  });
});
