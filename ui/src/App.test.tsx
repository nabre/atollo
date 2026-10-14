import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { AppInfo, SetupSnapshot } from "./api";

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
  watcherUnsafeFolder: null,
  firstRun: false,
};

const idle: SetupSnapshot = { running: false, cancelling: false, automatic: false, total: 0, steps: [], error: null };

/** Default answer of every command; tests override single commands. */
function answers(overrides: Record<string, unknown> = {}): (cmd: string) => Promise<unknown> {
  const base: Record<string, unknown> = {
    app_info: info,
    editor_status: { installed: true, running: false, port: null, emulated: false },
    setup_state: idle,
    projects_list: [],
    components_list: [],
    agents_status: [],
    editor_open: null,
    terminal_open: null,
    ...overrides,
  };
  return (cmd) => (cmd in base ? Promise.resolve(base[cmd]) : Promise.reject(new Error(`unexpected ${cmd}`)));
}

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

let setupListener: ((e: { payload: SetupSnapshot }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (e: { payload: SetupSnapshot }) => void) => {
    if (name === "setup") setupListener = handler;
    return Promise.resolve(() => undefined);
  },
}));

const { App } = await import("./App");

afterEach(() => {
  cleanup();
  invoke.mockReset();
  setupListener = null;
});

describe("App", () => {
  it("shows where the kit lives", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    expect(await screen.findByText("E:\\Dev Kit")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("app_info", undefined);
  });

  it("warns when the kit was moved", async () => {
    invoke.mockImplementation(answers({ app_info: { ...info, movedFrom: "F:\\Old" } }));
    render(<App />);
    expect(await screen.findByText(/prima era in F:\\Old/)).toBeTruthy();
  });

  it("warns about kit folders that break the editor watcher", async () => {
    invoke.mockImplementation(answers({ app_info: { ...info, watcherUnsafeFolder: "2026" } }));
    render(<App />);
    expect(await screen.findByText(/La cartella «2026»/)).toBeTruthy();
  });

  it("switches pages from the activity bar", () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Agenti" }));
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Agenti");
    expect(screen.getByRole("button", { name: "Agenti" }).getAttribute("aria-current")).toBe("page");
  });

  it("reports a failed command", async () => {
    invoke.mockImplementation((cmd) => (cmd === "app_info" ? Promise.reject(new Error("forbidden")) : answers()(cmd)));
    render(<App />);
    expect((await screen.findByRole("alert")).textContent).toContain("forbidden");
  });

  it("opens a project in the editor from the Projects page", async () => {
    invoke.mockImplementation(answers({ projects_list: [{ name: "sito (1) & co", modified: 1_791_900_000 }] }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Progetti" }));
    expect(await screen.findByText("sito (1) & co")).toBeTruthy();
    fireEvent.click(screen.getByTitle("Apre sito (1) & co nell'editor"));
    expect(invoke).toHaveBeenCalledWith("editor_open", { project: "sito (1) & co" });
  });

  it("shows the progress of an automatic installation", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    await screen.findByText("E:\\Dev Kit");
    act(() => {
      setupListener?.({
        payload: {
          running: true,
          cancelling: false,
          automatic: true,
          total: 2,
          error: null,
          steps: [
            {
              id: "git",
              name: "Git",
              note: null,
              done: null,
              size: null,
              outcome: { kind: "installed", version: "2.56" },
            },
            { id: "pwsh", name: "PowerShell 7", note: "download", done: 50, size: 100, outcome: null },
          ],
        },
      });
    });
    expect(screen.getByText("Installazione dei componenti del kit")).toBeTruthy();
    expect(screen.getByText("Installazione 2/2: PowerShell 7 50%")).toBeTruthy();
    expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe("50");
    expect(screen.getByText("installato 2.56")).toBeTruthy();
  });

  it("shows the first-start choice and applies it", async () => {
    const comps = [
      {
        id: "git",
        name: "Git",
        group: "required",
        desc: "Git",
        mb: 350,
        chosen: true,
        installed: false,
        version: null,
        available: true,
        x64Only: false,
        needs: [],
      },
      {
        id: "php",
        name: "PHP",
        group: "languages",
        desc: "PHP",
        mb: 100,
        chosen: true,
        installed: false,
        version: null,
        available: true,
        x64Only: true,
        needs: [],
      },
      {
        id: "apache",
        name: "Apache",
        group: "languages",
        desc: "web",
        mb: 45,
        chosen: true,
        installed: false,
        version: null,
        available: true,
        x64Only: true,
        needs: ["php"],
      },
      {
        id: "go",
        name: "Go",
        group: "languages",
        desc: "Go",
        mb: 250,
        chosen: false,
        installed: false,
        version: null,
        available: true,
        x64Only: false,
        needs: [],
      },
    ];
    invoke.mockImplementation(
      answers({ app_info: { ...info, firstRun: true }, components_list: comps, components_apply: null }),
    );
    render(<App />);
    expect(await screen.findByText("Benvenuto in Dev-Kit")).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>(/^Git/).disabled).toBe(true);
    fireEvent.click(screen.getByLabelText(/^PHP/));
    expect(screen.getByLabelText<HTMLInputElement>(/^Apache/).checked).toBe(false);
    fireEvent.click(screen.getByLabelText(/^Go/));
    fireEvent.click(screen.getByRole("button", { name: "Installa" }));
    expect(invoke).toHaveBeenCalledWith("components_apply", {
      chosen: expect.arrayContaining(["git", "go"]) as unknown,
      updateAll: false,
    });
    const call = invoke.mock.calls.find((c) => c[0] === "components_apply");
    expect((call?.[1] as { chosen: string[] }).chosen).not.toContain("apache");
  });

  it("shows the agents and opens one", async () => {
    const agents = [
      {
        id: "claude",
        name: "Claude Code",
        installed: true,
        version: "2.1.299",
        signedIn: true,
        signInKind: "account",
        running: false,
      },
      {
        id: "codex",
        name: "Codex",
        installed: false,
        version: null,
        signedIn: false,
        signInKind: null,
        running: false,
      },
    ];
    invoke.mockImplementation(answers({ agents_status: agents, agent_launch: null, agent_sign_out: null }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Agenti" }));
    expect(await screen.findByText("2.1.299")).toBeTruthy();
    expect(screen.getByText("accesso fatto")).toBeTruthy();
    const open = screen.getAllByRole("button", { name: "Apri" });
    expect(open[1]?.hasAttribute("disabled")).toBe(true);
    fireEvent.click(open[0] as HTMLElement);
    expect(invoke).toHaveBeenCalledWith("agent_launch", { agent: "claude", project: null });
    fireEvent.click(screen.getByRole("button", { name: "Esci" }));
    expect(screen.getByText("Togliere l'accesso salvato nel kit?")).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("agent_sign_out", expect.anything());
  });
});
