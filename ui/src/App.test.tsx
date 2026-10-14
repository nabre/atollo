import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { AgentInfo, AppInfo, OpenNow, ProjectInfo, RunEvent, RunInfo, SetupSnapshot } from "./api";

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
  otherKits: [],
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
    run_list: [],
    run_ports: { ports: {}, locked: [] },
    url_open: null,
    settings_get: {
      settings: {
        lang: null,
        network: { httpsProxy: null, httpProxy: null, noProxy: null, extraCaCerts: null, allowLan: false },
        geminiApiKey: null,
        autoUpdate: true,
        pythonVersion: null,
        phpVersion: null,
        rustToolchain: null,
        terminal: null,
      },
      geminiKeySet: true,
      problem: null,
    },
    settings_set: null,
    update_status: { current: "0.1.0", latest: null, staged: null, checking: false, error: null },
    app_quit: null,
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
let runListener: ((e: { payload: RunEvent }) => void) | null = null;
let closeListener: ((e: { payload: OpenNow }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (e: { payload: unknown }) => void) => {
    if (name === "setup") setupListener = handler;
    if (name === "run") runListener = handler;
    if (name === "close-requested") closeListener = handler;
    return Promise.resolve(() => undefined);
  },
}));

function project(name: string): ProjectInfo {
  return {
    id: name,
    name,
    modified: 1_791_900_000,
    empty: false,
    missing: [],
    platform: "Sito statico",
    type: "web",
    reason: "index.html",
    languages: ["HTML"],
    requires: [],
    actions: [
      {
        id: "static",
        label: "sito statico",
        command: "",
        type: "web",
        folder: null,
        url: null,
        env: {},
        static: ".",
        custom: false,
      },
    ],
    warnings: [],
    settings: {},
    custom: false,
    default: "static",
  };
}

const running: RunInfo = {
  key: "sito (1) & co|static",
  project: "sito (1) & co",
  action: "static",
  label: "sito statico",
  type: "web",
  command: "Dev-Kit.exe serve . 7100",
  port: 7100,
  url: "http://localhost:7100/",
  state: "running",
  start: 1,
  end: null,
  code: null,
  pid: 7,
};

const { App } = await import("./App");
const { setLang } = await import("./i18n");

afterEach(() => {
  cleanup();
  setLang("it");
  invoke.mockReset();
  setupListener = null;
  runListener = null;
  closeListener = null;
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

  it("signs in to GitHub with the one-time code", async () => {
    let status = {
      installed: true,
      signedIn: false,
      login: null,
      signingIn: false,
      code: null as string | null,
      error: null,
    };
    invoke.mockImplementation((cmd) => {
      if (cmd === "github_status") return Promise.resolve(status);
      if (cmd === "github_sign_in") {
        status = { ...status, signingIn: true, code: "1A2B-3C4D" };
        return Promise.resolve(status);
      }
      return answers()(cmd);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "GitHub" }));
    fireEvent.click(await screen.findByRole("button", { name: "Accedi a GitHub" }));
    expect(await screen.findByText("1A2B-3C4D")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("github_sign_in", { delete: false });
    expect(invoke).not.toHaveBeenCalledWith("github_repos", undefined);
  });

  it("lists the GitHub repositories, clones one and asks for the delete permission", async () => {
    const repo = (owner: string, name: string, extra: object = {}) => ({
      fullName: `${owner}/${name}`,
      name,
      owner,
      private: false,
      fork: false,
      archived: false,
      description: "",
      language: "",
      stars: 0,
      pushedAt: null,
      htmlUrl: `https://github.com/${owner}/${name}`,
      admin: true,
      local: null,
      ...extra,
    });
    invoke.mockImplementation(
      answers({
        github_status: { installed: true, signedIn: true, login: "me", signingIn: false, code: null, error: null },
        github_repos: {
          login: "me",
          canDelete: false,
          repos: [repo("me", "sito", { local: "sito" }), repo("me", "app10"), repo("me", "app2"), repo("acme", "erp")],
        },
        github_clone: "app2",
      }),
    );
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "GitHub" }));
    expect(await screen.findByText("Accesso fatto come @me")).toBeTruthy();
    const mine = await screen.findByRole("list", { name: "me" });
    // app2 before app10; the organization in its own group
    expect([...mine.querySelectorAll("li")].map((li) => li.querySelector(".font-semibold")?.textContent)).toEqual([
      "app2",
      "app10",
      "sito",
    ]);
    expect(screen.getByRole("list", { name: "acme" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Clona", description: "Copia me/app2 in projects" }));
    expect(await screen.findByText("me/app2 clonato in projects\\app2.")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("github_clone", { fullName: "me/app2" });
    fireEvent.click(screen.getByRole("button", { name: /^sito/ }));
    fireEvent.click(screen.getByRole("button", { name: "Elimina da GitHub" }));
    fireEvent.click(screen.getByRole("button", { name: "Elimina" }));
    expect(await screen.findByRole("button", { name: "Concedi il permesso" })).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("github_delete", expect.anything());
  });

  it("opens editor, console and agent only in a project from Home", async () => {
    const agents: AgentInfo[] = [
      {
        id: "claude",
        name: "Claude Code",
        installed: true,
        version: "2",
        signedIn: true,
        signInKind: "account",
        running: false,
      },
      { id: "codex", name: "Codex", installed: true, version: "1", signedIn: false, signInKind: null, running: false },
    ];
    invoke.mockImplementation(
      answers({
        projects_list: [{ ...project("app & co"), languages: ["TypeScript", "CSS"], settings: { agent: "codex" } }],
        agents_status: agents,
        agent_launch: null,
      }),
    );
    render(<App />);
    expect(await screen.findByText("app & co")).toBeTruthy();
    expect(screen.getByText("TypeScript")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Terminale" }));
    expect(invoke).toHaveBeenCalledWith("terminal_open", { project: "app & co" });
    // the agent last used in the project is the default
    await waitFor(() => {
      expect(screen.getByLabelText<HTMLSelectElement>("Agente per app & co").value).toBe("codex");
    });
    fireEvent.click(screen.getByRole("button", { name: "Agente" }));
    expect(invoke).toHaveBeenCalledWith("agent_launch", { agent: "codex", project: "app & co" });
    expect(invoke).not.toHaveBeenCalledWith("terminal_open", { project: null });
  });

  it("reports a failed command", async () => {
    invoke.mockImplementation((cmd) => (cmd === "app_info" ? Promise.reject(new Error("forbidden")) : answers()(cmd)));
    render(<App />);
    expect((await screen.findByRole("alert")).textContent).toContain("forbidden");
  });

  it("opens a project in the editor and starts its main action", async () => {
    invoke.mockImplementation(
      answers({
        projects_list: [project("sito (1) & co")],
        run_start: { ...running, state: "starting" },
        run_log: ["> Dev-Kit.exe serve . 7100"],
      }),
    );
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Progetti" }));
    expect(await screen.findByText("sito (1) & co")).toBeTruthy();
    fireEvent.click(screen.getByTitle("Apre sito (1) & co nell'editor"));
    expect(invoke).toHaveBeenCalledWith("editor_open", { project: "sito (1) & co" });
    fireEvent.click(screen.getByRole("button", { name: "Avvia" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("run_start", { project: "sito (1) & co", action: "static" });
    });
    act(() => {
      runListener?.({ payload: { kind: "state", ...running } });
      runListener?.({ payload: { kind: "log", key: running.key, lines: ["pronto"] } });
    });
    expect(await screen.findByText("in ascolto")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "localhost:7100" }));
    expect(invoke).toHaveBeenCalledWith("url_open", { url: "http://localhost:7100/" });
  });

  it("saves the settings without sending the hidden Gemini key", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    const proxy = await screen.findByLabelText("Proxy HTTPS");
    fireEvent.change(proxy, { target: { value: " http://proxy:8080 " } });
    fireEvent.click(screen.getByRole("button", { name: "Salva" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("settings_set", expect.objectContaining({ gemini: null }));
    });
    const call = invoke.mock.calls.find((c) => c[0] === "settings_set");
    expect(JSON.stringify(call?.[1])).toContain('"httpsProxy":"http://proxy:8080"');
    expect(await screen.findByText("Salvato.")).toBeTruthy();
  });

  it("asks before closing with something open, and switches language", async () => {
    invoke.mockImplementation(answers({ app_info: { ...info, lang: "en" } }));
    render(<App />);
    expect(await screen.findByRole("button", { name: "Projects" })).toBeTruthy();
    act(() => {
      closeListener?.({ payload: { processes: [], editor: true, terminals: 1, actions: 0, setup: false } });
    });
    const dialog = await screen.findByRole("alertdialog");
    expect(dialog.textContent).toContain("Kit consoles: 1");
    fireEvent.click(screen.getByRole("button", { name: "Close everything" }));
    expect(invoke).toHaveBeenCalledWith("app_quit", undefined);
  });

  it("prepares the drive for removal", async () => {
    const open = {
      processes: [{ pid: 42, name: "node.exe", program: "data\apps\win32-x64\node\node.exe" }],
      editor: false,
      terminals: 0,
      actions: 0,
      setup: false,
    };
    invoke.mockImplementation(answers({ eject_list: open, eject_run: { ...open, processes: [] }, app_quit: null }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Rimuovi l'unità" }));
    expect(await screen.findByText("node.exe")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Chiudi tutto e prepara la rimozione" }));
    fireEvent.click(await screen.findByRole("button", { name: "Chiudi Dev-Kit" }));
    expect(invoke).toHaveBeenCalledWith("eject_run", undefined);
    expect(invoke).toHaveBeenCalledWith("app_quit", undefined);
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
        present: false,
        locked: true,
        version: null,
        available: true,
        x64Only: false,
        noArm64: false,
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
        present: false,
        locked: false,
        version: null,
        available: true,
        x64Only: true,
        noArm64: false,
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
        present: false,
        locked: false,
        version: null,
        available: true,
        x64Only: true,
        noArm64: false,
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
        present: false,
        locked: false,
        version: null,
        available: true,
        x64Only: false,
        noArm64: false,
        needs: [],
      },
      {
        id: "r",
        name: "R",
        group: "languages",
        desc: "statistica",
        mb: 1100,
        chosen: false,
        installed: false,
        present: false,
        locked: false,
        version: null,
        available: false,
        x64Only: false,
        noArm64: true,
        needs: [],
      },
    ];
    invoke.mockImplementation(
      answers({ app_info: { ...info, firstRun: true }, components_list: comps, components_apply: null }),
    );
    render(<App />);
    expect(await screen.findByText("Benvenuto in Dev-Kit")).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>(/^Git(?!Hub)/).disabled).toBe(true);
    expect(screen.getByLabelText<HTMLInputElement>(/^Rstatistica/).disabled).toBe(true);
    expect(screen.getByText("non disponibile su PC ARM64")).toBeTruthy();
    // Dependencies: Apache shows what it needs, PHP what it is needed by.
    expect(screen.getByTitle("Richiede PHP: si installa insieme.")).toBeTruthy();
    expect(screen.getByText("per Apache")).toBeTruthy();
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
    // only the installed agents, and no launch outside a project
    expect(screen.queryByText("Codex")).toBeNull();
    expect(screen.queryByRole("button", { name: "Apri" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Esci" }));
    expect(screen.getByText("Togliere l'accesso salvato nel kit?")).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("agent_sign_out", expect.anything());
  });
});
