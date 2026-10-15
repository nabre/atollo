// Data of the kit without a project (PLAN-DATA.md WP-7): a closed section
// with what each one holds, "Collega a…" and "Elimina".
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { Holdings, ProjectInfo } from "../api";
import { Orphans } from "./orphans";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const OLD: Holdings = { project: "vecchio", settings: true, ports: 2, databases: ["mariadb"], backups: 3 };
const PROJECTS = [{ id: "nuovo", name: "nuovo" }] as ProjectInfo[];

function answer(orphans: Holdings[], extra: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd) => {
    if (cmd === "projects_orphans") return Promise.resolve(orphans);
    const handler = extra[cmd];
    return handler ? handler() : Promise.reject(new Error(`unexpected ${cmd}`));
  });
}

describe("Orphans", () => {
  it("shows nothing without orphans", async () => {
    answer([]);
    const { container } = render(<Orphans projects={PROJECTS} onChanged={vi.fn()} />);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("projects_orphans", undefined);
    });
    expect(container.textContent).toBe("");
  });

  it("is closed, with the count, and lists what each one holds", async () => {
    answer([OLD]);
    render(<Orphans projects={PROJECTS} onChanged={vi.fn()} />);
    const header = await screen.findByRole("button", { name: /Dati senza progetto \(1\)/ });
    expect(header.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByText("vecchio")).toBeNull();
    fireEvent.click(header);
    expect(await screen.findByText("vecchio")).toBeTruthy();
    expect(screen.getByText("impostazioni, porte: 2, database: MariaDB, copie: 3")).toBeTruthy();
  });

  it("gives the data to a project", async () => {
    answer([OLD], { project_orphan_adopt: () => Promise.resolve(null) });
    const onChanged = vi.fn();
    render(<Orphans projects={PROJECTS} onChanged={onChanged} />);
    fireEvent.click(await screen.findByRole("button", { name: /Dati senza progetto/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Collega a…" }));
    expect(screen.getByRole("alertdialog", { name: "Collegare vecchio a un progetto?" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Collega" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_orphan_adopt", { orphan: "vecchio", project: "nuovo" });
    });
    await waitFor(() => {
      expect(onChanged).toHaveBeenCalled();
    });
  });

  it("deletes with the confirmation of a project's deletion", async () => {
    answer([OLD], {
      project_delete_plan: () =>
        Promise.resolve({ folderExists: false, folderEmpty: false, databases: [], backups: 0, backupsSize: 0 }),
      project_delete: () => Promise.resolve(null),
    });
    render(<Orphans projects={PROJECTS} onChanged={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /Dati senza progetto/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Elimina" }));
    expect(screen.getByRole("alertdialog", { name: "Eliminare vecchio?" })).toBeTruthy();
    await screen.findByText("Le impostazioni e i dati del kit di questo progetto");
    fireEvent.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Elimina" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_delete", { project: "vecchio", forceWithoutDb: false });
    });
  });
});
