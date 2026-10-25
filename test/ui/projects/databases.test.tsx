// The Database section of a project's page: a server version chosen for
// the project and then uninstalled is told, with "Install" and the other
// versions to move to; the section is read again when the install ends.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SAMPLE_SETUP, type ProjectDatabases, type ProjectInfo, type SetupSnapshot } from "@/ipc";
import { Databases } from "@/projects/databases";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

const listeners: ((e: { payload: SetupSnapshot }) => void)[] = [];
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_event: string, handler: (e: { payload: SetupSnapshot }) => void) => {
    listeners.push(handler);
    return Promise.resolve(() => undefined);
  },
}));

/** A `setup` event, to every section listening. */
function emit(running: boolean) {
  const payload: SetupSnapshot = { ...SAMPLE_SETUP, running, total: 1 };
  act(() => {
    for (const l of listeners) l({ payload });
  });
}

afterEach(() => {
  cleanup();
  invoke.mockReset();
  listeners.length = 0;
});

const P = { id: "shop" } as ProjectInfo;

function dbs(chosen: string | null, lines: string[]): ProjectDatabases {
  return {
    dbs: [],
    detected: ["mysql"],
    envExists: false,
    envIgnored: true,
    versions: {
      mysql: { chosen, server: lines[0] ?? null, lines },
      mongodb: { chosen: null, server: null, lines: [] },
    },
  };
}

describe("project databases", () => {
  it("tells a version uninstalled and offers to install it", async () => {
    invoke.mockResolvedValue(null);
    const changed = vi.fn();
    render(<Databases p={P} dbs={dbs("10.6", ["11.4"])} onChanged={changed} />);
    expect(screen.getByText(/MySQL 10\.6 non è installata/)).toBeTruthy();
    const version = screen.getByLabelText<HTMLElementTagNameMap["vscode-single-select"]>("Versione del server");
    expect(version.querySelectorAll("vscode-option")).toHaveLength(3);
    fireEvent.click(screen.getByRole("button", { name: "Installa MySQL 10.6" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("versions_add", { id: "mysql", line: "10.6" });
    });
    changed.mockClear();
    emit(true);
    expect(changed).not.toHaveBeenCalled();
    emit(false);
    expect(changed).toHaveBeenCalledTimes(1);
    // another version: the project moves to it
    fireEvent.change(version, { target: { value: "11.4" } });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_db_line", { project: "shop", engine: "mysql", line: "11.4" });
    });
  });

  it("copies the connection as .env lines, read only when asked", async () => {
    const block = "DB_CONNECTION=mysql\nDB_PASSWORD=pw1\n";
    invoke.mockImplementation((c) => Promise.resolve(c === "project_env_block" ? block : null));
    const writeText = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    const withDb: ProjectDatabases = {
      ...dbs(null, ["8.4"]),
      dbs: [
        {
          engine: "mysql",
          database: "shop",
          user: "shop",
          password: "pw1",
          host: "127.0.0.1",
          port: 3306,
          url: "mysql://shop:pw1@127.0.0.1:3306/shop",
        },
      ],
    };
    render(<Databases p={P} dbs={withDb} onChanged={() => undefined} />);
    expect(invoke).not.toHaveBeenCalledWith("project_env_block", expect.anything());
    fireEvent.click(screen.getByRole("button", { name: "Copia blocco .env" }));
    await waitFor(() => {
      expect(writeText).toHaveBeenCalledWith(block);
    });
    expect(invoke).toHaveBeenCalledWith("project_env_block", { project: "shop", engine: "mysql" });
    expect(await screen.findByText("Copiato")).toBeTruthy();
  });

  it("says nothing while the version is installed", () => {
    render(<Databases p={P} dbs={dbs("10.6", ["11.4", "10.6"])} onChanged={() => undefined} />);
    expect(screen.queryByText(/non è installata/)).toBeNull();
  });
});
