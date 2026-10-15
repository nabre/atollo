// The "Temporanei dei programmi" setting (PLAN-DATA.md T11.4) and the disk
// facts of the Il kit group (T8.5).
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { AppInfo, Settings } from "../api";
import { KitInfo } from "./about";
import { GeneralSettings } from "./sections";

afterEach(cleanup);

const SETTINGS: Settings = {
  lang: null,
  network: { httpsProxy: null, httpProxy: null, noProxy: null, extraCaCerts: null, allowLan: false },
  geminiApiKey: null,
  autoUpdate: true,
  pythonVersion: null,
  phpVersion: null,
  rustToolchain: null,
  terminal: null,
  programTemp: "auto",
};

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
  const info: AppInfo = {
    version: "0.1.0",
    root: "E:\\Kit",
    data: "E:\\Kit\\data",
    projects: "E:\\Kit\\projects",
    platform: "win32-x64",
    lang: "it",
    movedFrom: null,
    webview: "x",
    pathEntries: 0,
    watcherUnsafeFolder: null,
    firstRun: false,
    otherKits: [],
    diskFs: "exFAT",
    diskKind: "removable",
    systemTemp: "C:\\Users\\me\\AppData\\Local\\Temp\\Atollo\\ab12",
  };

  it("names the file system, the kind of disk and the temp folder in use", () => {
    render(<KitInfo info={info} />);
    expect(screen.getByText("exFAT, rimovibile")).toBeTruthy();
    expect(screen.getByText(info.systemTemp ?? "")).toBeTruthy();
  });

  it("falls back to data\\tmp and hides an unreadable disk", () => {
    render(<KitInfo info={{ ...info, diskFs: null, diskKind: null, systemTemp: null }} />);
    expect(screen.queryByText("Disco del kit")).toBeNull();
    expect(screen.getByText("data\\tmp")).toBeTruthy();
  });
});
