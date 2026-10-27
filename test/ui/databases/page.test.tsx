// The Database page (N-U1): the list read every 3 s and the actions keep
// their errors apart, and a read still running when an action answers does
// not bring back the old list.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { SAMPLE_SERVICES, type ServiceInfo } from "@/ipc";
import { DatabasesPage } from "@/databases/page";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

/** Both servers stopped. */
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

  it("starts another version on its own port and sets that port (D-102)", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list") return Promise.resolve(STOPPED);
      if (cmd === "service_start" || cmd === "service_line_port") return Promise.resolve(STOPPED);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    expect(screen.getByText("Altre versioni")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Avvia 8.4" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("service_start", { id: "mysql", line: "8.4" });
    const port = document.getElementById("port-mysql-8.4") as HTMLInputElement;
    expect(port.value).toBe("3307");
    fireEvent.change(port, { target: { value: "3317" } });
    fireEvent.blur(port);
    await flush();
    expect(invoke).toHaveBeenCalledWith("service_line_port", { id: "mysql", line: "8.4", port: 3317 });
  });

  it("opens phpMyAdmin on another version", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list" || cmd === "db_admin_open") return Promise.resolve(STOPPED);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    fireEvent.click(screen.getByRole("button", { name: "phpMyAdmin su 8.4" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_admin_open", { line: "8.4", engine: "mysql" });
    fireEvent.click(screen.getByRole("button", { name: "Apri phpMyAdmin" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_admin_open", { line: null, engine: "mysql" });
  });

  // D-147: MongoDB's page opens mongo-express, on its version or another.
  it("opens mongo-express on MongoDB", async () => {
    const mongo: ServiceInfo = {
      ...(STOPPED[1] as ServiceInfo),
      line: "8.0",
      lines: ["8.0", "7.0"],
      others: [
        {
          line: "7.0",
          state: "stopped",
          port: 27018,
          dataDir: "E:\\Atollo\\data\\db\\mongodb\\7.0",
          log: "E:\\Atollo\\data\\app\\logs\\mongodb-7.0.log",
          url: "mongodb://admin@127.0.0.1:27018/?authSource=admin",
          message: null,
        },
      ],
    };
    invoke.mockImplementation((cmd) => {
      if (cmd === "services_list" || cmd === "db_admin_open") return Promise.resolve([STOPPED[0], mongo]);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    fireEvent.click(screen.getByRole("button", { name: /^MongoDB/ }));
    await flush();
    expect(screen.queryByRole("button", { name: "Apri phpMyAdmin" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Apri mongo-express" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_admin_open", { line: null, engine: "mongodb" });
    // Stopped: the page shows its details, with the other versions.
    fireEvent.click(screen.getByRole("button", { name: "mongo-express su 7.0" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("db_admin_open", { line: "7.0", engine: "mongodb" });
  });

  it("shows a running MongoDB with its details only", async () => {
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
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<DatabasesPage />);
    await flush();
    fireEvent.click(screen.getByRole("button", { name: /^MongoDB/ }));
    await flush();
    // Its content is in mongo-express (D-150): only the details, no tabs.
    expect(screen.queryByRole("navigation", { name: "Sezioni del server" })).toBeNull();
    expect(screen.getByRole("button", { name: "Apri mongo-express" })).toBeTruthy();
    expect(screen.getByText("Connessione")).toBeTruthy();
  });
});
