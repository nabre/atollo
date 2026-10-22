// «Copia del kit» (N-12): export to a zip, import of a copy waiting for the
// next start, cancelled or closed for.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { KitBackup } from "@/settings/backup";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

describe("KitBackup", () => {
  it("exports to a zip and says where", async () => {
    invoke.mockImplementation((c) =>
      Promise.resolve(
        c === "kit_import_pending"
          ? false
          : c === "kit_export"
            ? { path: "E:\\Copie\\Atollo-1.zip", files: 12, bytes: 3 * 1024 * 1024 }
            : undefined,
      ),
    );
    render(<KitBackup />);
    fireEvent.click(screen.getByRole("button", { name: "Esporta in uno zip…" }));
    expect(await screen.findByText("E:\\Copie\\Atollo-1.zip")).toBeTruthy();
    expect(screen.getByText(/Copia fatta: 12 file, 3 MB/)).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("kit_export", { zip: true });
  });

  it("nothing chosen: nothing said", async () => {
    invoke.mockImplementation((c) => Promise.resolve(c === "kit_import_pending" ? false : null));
    render(<KitBackup />);
    fireEvent.click(screen.getByRole("button", { name: "Importa da una cartella…" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("kit_import", { zip: false });
    });
    expect(screen.queryByRole("button", { name: "Chiudi Atollo per importarla" })).toBeNull();
  });

  it("a copy imported waits for the next start, or is cancelled", async () => {
    let pending = false;
    invoke.mockImplementation((c) => {
      if (c === "kit_import_pending") return Promise.resolve(pending);
      if (c === "kit_import") {
        pending = true;
        return Promise.resolve({ atollo: "0.5.1", created: 1_792_600_000, root: "E:\\A", files: 7, bytes: 2048 });
      }
      return Promise.resolve(undefined);
    });
    render(<KitBackup />);
    fireEvent.click(screen.getByRole("button", { name: "Importa da uno zip…" }));
    expect(await screen.findByText(/Copia pronta: 7 file/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Chiudi Atollo per importarla" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Annulla l'importazione" }));
    await waitFor(() => {
      expect(screen.queryByRole("button", { name: "Chiudi Atollo per importarla" })).toBeNull();
    });
    expect(invoke).toHaveBeenCalledWith("kit_import_cancel", undefined);
  });

  it("a copy left waiting from before is told at once", async () => {
    invoke.mockImplementation((c) => Promise.resolve(c === "kit_import_pending" ? true : undefined));
    render(<KitBackup />);
    expect(await screen.findByText(/Una copia aspetta di essere importata/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Chiudi Atollo per importarla" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("app_quit", undefined);
    });
  });
});
