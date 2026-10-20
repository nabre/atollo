// The packages kept to reinstall the conda environments without the
// network (D-118): the setting, the space and «Libera».
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { CondaKept, Settings } from "../../../ui/src/api";
import { CondaSettings } from "../../../ui/src/settings/conda";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const SETTINGS: Settings = {
  lang: null,
  network: { httpsProxy: null, httpProxy: null, noProxy: null, extraCaCerts: null, allowLan: false },
  geminiApiKey: null,
  autoUpdate: true,
  rustToolchain: null,
  programTemp: "auto",
  theme: "system",
  keepCondaPackages: true,
  toolsTarget: null,
};

const KEPT: CondaKept = { applies: true, bytes: 412 * 1024 * 1024, lists: ["r@4.5", "ruby@3.4"] };

describe("CondaSettings", () => {
  it("is not shown where every component follows the kit (Windows)", async () => {
    invoke.mockResolvedValue({ applies: false, bytes: 0, lists: [] });
    const { container } = render(<CondaSettings category="Generale" s={SETTINGS} set={vi.fn()} />);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("conda_kept", undefined);
    });
    expect(container.textContent).toBe("");
  });

  it("shows the space, frees it and turns the setting off", async () => {
    invoke.mockImplementation((cmd) =>
      Promise.resolve(cmd === "conda_kept_free" ? { ...KEPT, bytes: 0, lists: [] } : KEPT),
    );
    const set = vi.fn();
    render(<CondaSettings category="Generale" s={SETTINGS} set={set} />);
    expect(await screen.findByText("Copia per reinstallare senza rete: 412 MB")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Libera" }));
    await waitFor(() => {
      expect(screen.queryByText(/Copia per reinstallare senza rete/)).toBeNull();
    });
    expect(invoke).toHaveBeenCalledWith("conda_kept_free", undefined);
    fireEvent.click(screen.getByRole("checkbox", { name: "Conserva i pacchetti nel kit" }));
    expect(set).toHaveBeenCalledWith({ keepCondaPackages: false });
  });
});
