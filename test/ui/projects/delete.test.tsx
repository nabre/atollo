// The confirmation of a project's deletion (D-89): it lists
// the plan (folder, databases, copies), "Elimina" drops the databases with
// the folder, and only after an error "Elimina senza i database" appears.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { DeletePlan } from "@/api";
import { DeleteProjectDialog } from "@/projects/delete";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const PLAN: DeletePlan = {
  folderExists: true,
  folderEmpty: true,
  databases: [
    { engine: "mysql", database: "shop", installed: true },
    { engine: "mongodb", database: "shop", installed: false },
  ],
  backups: 2,
  backupsSize: 3 * 1024 * 1024,
};

function answer(plan: DeletePlan | Error, del: () => Promise<unknown>) {
  invoke.mockImplementation((cmd) => {
    if (cmd === "project_delete_plan") return plan instanceof Error ? Promise.reject(plan) : Promise.resolve(plan);
    if (cmd === "project_delete") return del();
    if (cmd === "folder_open") return Promise.resolve(null);
    return Promise.reject(new Error(`unexpected ${cmd}`));
  });
}

function show() {
  const onCancel = vi.fn();
  const onDeleted = vi.fn();
  render(<DeleteProjectDialog project="shop" onCancel={onCancel} onDeleted={onDeleted} />);
  return { onCancel, onDeleted };
}

describe("DeleteProjectDialog", () => {
  it("lists the plan and deletes with the databases", async () => {
    answer(PLAN, () => Promise.resolve(null));
    const { onDeleted } = show();
    expect(screen.getByRole("alertdialog", { name: "Eliminare shop?" })).toBeTruthy();
    await screen.findByText("Il database shop e il suo utente su MySQL");
    expect(screen.getByText("MongoDB non è installato: il database shop resta nei suoi dati")).toBeTruthy();
    expect(screen.getByText(/Copie dei database: 2 \(3 MB\)/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Elimina senza i database" })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Apri la cartella delle copie" }));
    expect(invoke).toHaveBeenCalledWith("folder_open", { which: "backup", project: "shop" });

    fireEvent.click(screen.getByRole("button", { name: "Elimina" }));
    await waitFor(() => {
      expect(onDeleted).toHaveBeenCalledTimes(1);
    });
    expect(invoke).toHaveBeenCalledWith("project_delete", { project: "shop", forceWithoutDb: false });
  });

  it("offers to delete without the databases after a failed drop", async () => {
    let calls = 0;
    answer(PLAN, () => (++calls === 1 ? Promise.reject(new Error("mysql: port 3306 in use")) : Promise.resolve(null)));
    const { onDeleted, onCancel } = show();
    await screen.findByText("Il database shop e il suo utente su MySQL");
    fireEvent.click(screen.getByRole("button", { name: "Elimina" }));
    await screen.findByText(/port 3306 in use/);
    expect(onDeleted).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Elimina senza i database" }));
    await waitFor(() => {
      expect(onDeleted).toHaveBeenCalledTimes(1);
    });
    expect(invoke).toHaveBeenLastCalledWith("project_delete", { project: "shop", forceWithoutDb: true });

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("without a plan only the forced delete is possible", async () => {
    answer(new Error("db.json: not valid JSON"), () => Promise.resolve(null));
    show();
    await screen.findByText(/not valid JSON/);
    expect(screen.getByRole("button", { name: "Elimina" }).hasAttribute("disabled")).toBe(true);
    expect(screen.getByRole("button", { name: "Elimina senza i database" })).toBeTruthy();
    expect(screen.queryByText(/Copie dei database/)).toBeNull();
  });
});
