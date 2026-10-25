// The "Temporanei dei programmi" setting (D-90) and the disk
// facts of the Il kit group (D-90).
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SAMPLE_INFO, SAMPLE_SETTINGS, type AppInfo } from "@/ipc";
import { KitInfo } from "@/settings/about";
import { GeneralSettings } from "@/settings/sections";

afterEach(cleanup);

// The browser preview's settings: «Automatico».
const SETTINGS = SAMPLE_SETTINGS.settings;

describe("program temp setting", () => {
  it("shows the saved choice with its help and sends the new one", async () => {
    const set = vi.fn();
    render(<GeneralSettings s={SETTINGS} set={set} />);
    expect(screen.getByText(/^Automatico usa la cartella temporanea di Windows/)).toBeTruthy();
    const select = screen.getByLabelText<HTMLElementTagNameMap["vscode-single-select"]>("Temporanei dei programmi");
    await waitFor(() => {
      expect(select.value).toBe("auto");
    });
    expect(select.querySelectorAll("vscode-option")).toHaveLength(3);
    fireEvent.change(select, { target: { value: "system" } });
    expect(set).toHaveBeenCalledWith({ programTemp: "system" });
  });
});

describe("KitInfo disk facts", () => {
  // The browser preview's kit, on a removable exFAT disk.
  const info: AppInfo = {
    ...SAMPLE_INFO,
    diskFs: "exFAT",
    diskKind: "removable",
    diskNoexec: null,
    diskNote: null,
    systemTemp: "C:\\Users\\me\\AppData\\Local\\Temp\\Atollo\\ab12",
  };

  it("names the file system, the kind of disk and the temp folder in use", () => {
    render(<KitInfo info={info} />);
    expect(screen.getByText("exFAT, rimovibile")).toBeTruthy();
    expect(screen.getByText(info.systemTemp ?? "")).toBeTruthy();
  });

  it("adds noexec and the app's note on the kit's disk (D-105)", () => {
    const note = "Il disco del kit è exFAT: niente link simbolici.";
    render(<KitInfo info={{ ...info, diskFs: "POSIX", diskKind: "fixed", diskNoexec: true, diskNote: note }} />);
    expect(screen.getByText("POSIX, fisso, noexec")).toBeTruthy();
    expect(screen.getByText(note)).toBeTruthy();
  });

  it("falls back to data\\tmp and hides an unreadable disk", () => {
    render(<KitInfo info={{ ...info, diskFs: null, diskKind: null, systemTemp: null }} />);
    expect(screen.queryByText("Disco del kit")).toBeNull();
    expect(screen.getByText("data\\tmp")).toBeTruthy();
  });
});
