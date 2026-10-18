// The move of MariaDB's data to MySQL on the Database page (D-117): the
// error with its dump and "Riprova", then the data kept and "Elimina".
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SAMPLE_MIGRATION, type DbMigration } from "../../../ui/src/api";
import { MigrationPanel } from "../../../ui/src/databases";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const FAILED: DbMigration = {
  state: "failed",
  lines: [
    { mariadb: "12.3", mysql: "9.7", port: 3306, done: true },
    { mariadb: "11.4", mysql: "8.4", port: 3307, done: false },
  ],
  error: "MariaDB 11.4: restore on MySQL: ERROR 1273",
  dump: "E:\\Dev-Kit\\data\\db\\mariadb\\dumps\\11.4.mysql.sql",
  kept: null,
  keptBytes: 0,
};

const DONE: DbMigration = {
  ...SAMPLE_MIGRATION,
  state: "done",
  kept: "E:\\Dev-Kit\\data\\db\\mariadb.pre-mysql",
  keptBytes: 250 * 1024 * 1024,
};

describe("MigrationPanel", () => {
  it("shows nothing on a kit without MariaDB", async () => {
    invoke.mockResolvedValue(SAMPLE_MIGRATION);
    const { container } = render(<MigrationPanel />);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("db_migration", undefined);
    });
    expect(container.textContent).toBe("");
  });

  it("tells a failed move with its dump and tries again", async () => {
    invoke.mockImplementation((cmd) => Promise.resolve(cmd === "db_migration_retry" ? DONE : FAILED));
    render(<MigrationPanel />);
    expect(await screen.findByText(/MariaDB resta attivo/)).toBeTruthy();
    expect(screen.getByText("MariaDB 11.4 → MySQL 8.4, porta 3307")).toBeTruthy();
    expect(screen.getByText(/ERROR 1273/)).toBeTruthy();
    expect(screen.getByText(FAILED.dump ?? "")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Riprova" }));
    expect(await screen.findByText("Dati MariaDB conservati (250 MB)")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("db_migration_retry", undefined);
  });

  it("deletes the data kept only after the question", async () => {
    invoke.mockImplementation((cmd) => Promise.resolve(cmd === "db_migration_drop_kept" ? SAMPLE_MIGRATION : DONE));
    render(<MigrationPanel />);
    fireEvent.click(await screen.findByRole("button", { name: "Elimina" }));
    expect(screen.getByText("Eliminare i dati MariaDB conservati?")).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("db_migration_drop_kept", undefined);
    fireEvent.click(screen.getByRole("button", { name: "Elimina" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("db_migration_drop_kept", undefined);
    });
  });
});
