// The Database section of a project's page: a server version chosen for
// the project and then uninstalled is told, with "Install" and the other
// versions to move to; the section is read again when the install ends.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ProjectDatabases, ProjectInfo, SetupSnapshot } from "../../../ui/src/api";
import { Databases } from "../../../ui/src/projects/databases";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

const listeners: ((e: { payload: SetupSnapshot }) => void)[] = [];
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_event: string, handler: (e: { payload: SetupSnapshot }) => void) => {
    listeners.push(handler);
    return Promise.resolve(() => undefined);
  },
}));

/** A `setup` event, to every section listening. */
function emit(running: boolean) {
  const payload: SetupSnapshot = {
    running,
    cancelling: false,
    automatic: false,
    total: 1,
    steps: [],
    queue: [],
    error: null,
  };
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

  it("says nothing while the version is installed", () => {
    render(<Databases p={P} dbs={dbs("10.6", ["11.4", "10.6"])} onChanged={() => undefined} />);
    expect(screen.queryByText(/non è installata/)).toBeNull();
  });
});
