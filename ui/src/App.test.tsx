import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { AppInfo } from "./api";

const info: AppInfo = {
  version: "0.1.0",
  root: "E:\\Dev Kit",
  data: "E:\\Dev Kit\\data",
  projects: "E:\\Dev Kit\\projects",
  platform: "win32-x64",
  lang: "it",
  movedFrom: null,
  webview: "system 140.0",
  pathEntries: 5,
};

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

const { App } = await import("./App");

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

describe("App", () => {
  it("shows where the kit lives", async () => {
    invoke.mockResolvedValue(info);
    render(<App />);
    expect(await screen.findByText("E:\\Dev Kit")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("app_info", undefined);
  });

  it("warns when the kit was moved", async () => {
    invoke.mockResolvedValue({ ...info, movedFrom: "F:\\Old" });
    render(<App />);
    expect(await screen.findByText(/prima era in F:\\Old/)).toBeTruthy();
  });

  it("switches pages from the activity bar", () => {
    invoke.mockResolvedValue(info);
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Agenti" }));
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Agenti");
    expect(screen.getByRole("button", { name: "Agenti" }).getAttribute("aria-current")).toBe("page");
  });

  it("reports a failed command", async () => {
    invoke.mockRejectedValue("forbidden");
    render(<App />);
    expect((await screen.findByRole("alert")).textContent).toContain("forbidden");
  });

  it("opens a project in the editor from the Projects page", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "projects_list") return Promise.resolve([{ name: "sito (1) & co", modified: 1_791_900_000 }]);
      if (cmd === "editor_status")
        return Promise.resolve({ installed: true, running: false, port: null, emulated: false });
      if (cmd === "editor_open") return Promise.resolve(null);
      return Promise.resolve(info);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Progetti" }));
    expect(await screen.findByText("sito (1) & co")).toBeTruthy();
    fireEvent.click(screen.getByTitle("Apre sito (1) & co nell'editor"));
    expect(invoke).toHaveBeenCalledWith("editor_open", { project: "sito (1) & co" });
  });
});
