import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type {
  AgentInfo,
  AppInfo,
  FrameworkPiece,
  OpenNow,
  ProjectInfo,
  RunEvent,
  RunInfo,
  SetupSnapshot,
} from "../../ui/src/api";

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
  diskFs: "NTFS",
  diskKind: "fixed",
  diskNoexec: null,
  diskNote: null,
  systemTemp: null,
};

const idle: SetupSnapshot = {
  running: false,
  cancelling: false,
  automatic: false,
  total: 0,
  steps: [],
  queue: [],
  error: null,
};

const PIECES: FrameworkPiece[] = [
  {
    id: "laravel",
    kind: "base",
    name: "Laravel",
    desc: "Framework PHP per applicazioni web",
    home: "https://laravel.com/docs/13.x/installation",
    needs: ["php", "node"],
    create: ["composer create-project laravel/laravel:^13.0 {name} --prefer-dist"],
    steps: ["npm install"],
  },
  {
    id: "vite",
    kind: "base",
    name: "Vite",
    desc: "Sito o applicazione web in TypeScript",
    home: "https://vite.dev/guide/",
    needs: ["node"],
    create: ["npm create vite@9 {name} -- --template vanilla-ts"],
    steps: ["npm install"],
  },
  {
    id: "react",
    kind: "addon",
    slot: "frontend",
    name: "React",
    desc: "Interfaccia in React con TypeScript",
    home: "https://react.dev/",
    on: { vite: { create: ["npm create vite@9 {name} -- --template react-ts"] } },
  },
];

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
        programTemp: "auto",
      },
      geminiKeySet: true,
      problem: null,
    },
    settings_set: null,
    frameworks_get: { pieces: PIECES, official: PIECES, texts: {}, problem: null },
    frameworks_set: null,
    frameworks_plan: {
      needs: ["node"],
      create: ["npm create vite@9 {name} -- --template react-ts"],
      steps: ["npm install"],
      actions: [{ name: "Avvia", command: "npm run dev" }],
      database: null,
      env: [{ key: "APP_NAME", value: "{name}" }],
      notes: [],
    },
    frameworks_create: "app",
    frameworks_targets: [
      { name: "vuoto", state: "empty" },
      { name: "appunti", state: "files" },
      { name: "repo", state: "git" },
    ],
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
        apache: null,
        custom: false,
      },
    ],
    warnings: [],
    settings: {},
    extensions: [],
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
  skippedAliases: [],
};

const { App } = await import("../../ui/src/App");
const { setLang } = await import("../../ui/src/i18n");

afterEach(() => {
  cleanup();
  setLang("it");
  invoke.mockReset();
  setupListener = null;
  runListener = null;
  closeListener = null;
});

describe("App", () => {
  it("shows where the kit lives, in the settings", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    fireEvent.click(await screen.findByRole("button", { name: "Il kit" }));
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
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    fireEvent.click(await screen.findByRole("button", { name: "Account GitHub" }));
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
    // New project › from your GitHub repositories (the account is in the settings)
    fireEvent.click(await screen.findByRole("button", { name: "Dai tuoi repository GitHub" }));
    const mine = await screen.findByRole("list", { name: "me" });
    // app2 before app10; the organization in its own group
    expect([...mine.querySelectorAll("li")].map((li) => li.querySelector("button .truncate")?.textContent)).toEqual([
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
    // the list shows no language logos (the project's page does)
    expect(screen.queryByRole("img", { name: "TypeScript" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Terminale" }));
    expect(invoke).toHaveBeenCalledWith("terminal_open", { project: "app & co" });
    // the row's agent is the one last used in the project
    fireEvent.click(await screen.findByTitle("Apre Codex nel progetto app & co"));
    expect(invoke).toHaveBeenCalledWith("agent_launch", { agent: "codex", project: "app & co" });
    expect(invoke).not.toHaveBeenCalledWith("terminal_open", { project: null });
    // on the project's page, the same agent is the one chosen
    fireEvent.click(screen.getByTitle("Apre la pagina di app & co"));
    await waitFor(() => {
      expect(screen.getByRole("radio", { name: "Codex" }).getAttribute("aria-checked")).toBe("true");
    });
  });

  it("opens a project's page with its sections and creates its database", async () => {
    invoke.mockImplementation(
      answers({
        projects_list: [{ ...project("negozio"), requires: ["php", "mariadb"], missing: ["mariadb"] }],
        project_status: { git: { branch: "main", upstream: null, synced: null, remote: null }, instructions: [] },
        project_databases: {
          dbs: [],
          detected: ["mariadb"],
          envExists: true,
          envIgnored: true,
          versions: { mariadb: { chosen: null, server: "11.4", lines: ["11.4", "10.6"] } },
        },
        project_db_line: null,
        project_db_create: {
          engine: "mariadb",
          database: "negozio",
          user: "negozio",
          password: "x",
          host: "127.0.0.1",
          port: 3306,
          url: "mysql://negozio:x@127.0.0.1:3306/negozio",
        },
      }),
    );
    render(<App />);
    fireEvent.click(await screen.findByTitle("Apre la pagina di negozio"));
    const nav = screen.getByRole("navigation", { name: "Sezioni del progetto" });
    expect(
      within(nav)
        .getAllByRole("button")
        .map((b) => b.textContent),
    ).toEqual(["Avvio", "Sito web", "Componenti", "Database", "Estensioni", "Editor", "Git"]);
    expect(await screen.findByText("main")).toBeTruthy();
    fireEvent.click(within(nav).getByRole("button", { name: /Componenti/ }));
    expect(screen.getByText("da installare")).toBeTruthy();
    fireEvent.click(within(nav).getByRole("button", { name: "Database" }));
    const creates = await screen.findAllByRole("button", { name: "Crea database e utente" });
    fireEvent.click(creates[0] as HTMLElement);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_db_create", { project: "negozio", engine: "mariadb" });
    });
    // The version of the server for its database, each on its own port.
    const version = screen.getByLabelText<HTMLElementTagNameMap["vscode-single-select"]>("Versione del server");
    expect(version.querySelectorAll("vscode-option")).toHaveLength(3);
    fireEvent.change(version, { target: { value: "10.6" } });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_db_line", { project: "negozio", engine: "mariadb", line: "10.6" });
    });
    // The activity bar's Projects again: back to the list
    fireEvent.click(screen.getAllByRole("button", { name: "Progetti" })[0] as HTMLElement);
    expect(await screen.findByRole("heading", { name: "Nuovo progetto" })).toBeTruthy();
  });

  it("reports a failed command", async () => {
    invoke.mockImplementation((cmd) => (cmd === "app_info" ? Promise.reject(new Error("forbidden")) : answers()(cmd)));
    render(<App />);
    expect((await screen.findByRole("alert")).textContent).toContain("forbidden");
  });

  it("reads the kit's state again from the banner", async () => {
    let fail = true;
    invoke.mockImplementation((cmd) =>
      cmd === "app_info" && fail ? Promise.reject(new Error("forbidden")) : answers()(cmd),
    );
    render(<App />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Impossibile leggere lo stato del kit: Error: forbidden");
    fail = false;
    fireEvent.click(within(alert).getByRole("button", { name: "Riprova" }));
    await waitFor(() => {
      expect(screen.queryByText(/Impossibile leggere lo stato del kit/)).toBeNull();
    });
    expect(invoke.mock.calls.filter(([c]) => c === "app_info")).toHaveLength(2);
  });

  it("opens a project in the editor and starts an action from its page", async () => {
    invoke.mockImplementation(
      answers({
        projects_list: [project("sito (1) & co")],
        run_start: { ...running, state: "starting" },
        run_log: ["> Dev-Kit.exe serve . 7100"],
      }),
    );
    render(<App />);
    expect(await screen.findByText("sito (1) & co")).toBeTruthy();
    fireEvent.click(screen.getByTitle("Apre sito (1) & co nell'editor"));
    expect(invoke).toHaveBeenCalledWith("editor_open", { project: "sito (1) & co" });
    // No Avvia in the list: the project's page starts it
    expect(screen.queryByRole("button", { name: "Avvia" })).toBeNull();
    fireEvent.click(screen.getByTitle("Apre la pagina di sito (1) & co"));
    fireEvent.click((await screen.findAllByRole("button", { name: "Avvia" }))[0] as HTMLElement);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("run_start", { project: "sito (1) & co", action: "static" });
    });
    act(() => {
      runListener?.({ payload: { kind: "state", ...running } });
      runListener?.({ payload: { kind: "log", key: running.key, lines: ["pronto"] } });
    });
    // Its state and output in the Activity panel
    fireEvent.click(screen.getByRole("button", { name: /^Attività/ }));
    expect((await screen.findAllByText(/in ascolto/)).length).toBeGreaterThan(0);
    fireEvent.click(screen.getAllByRole("button", { name: "Apri nel browser" })[0] as HTMLElement);
    expect(invoke).toHaveBeenCalledWith("url_open", { url: "http://localhost:7100/" });
  });

  it("saves the settings without sending the hidden Gemini key", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    fireEvent.click(await screen.findByRole("button", { name: "Rete" }));
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

  it("finds a setting with the search of the side bar", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    expect(await screen.findByLabelText("Lingua")).toBeTruthy();
    // The field draws its placeholder inside the component: found by its name.
    fireEvent.change(screen.getByLabelText("Cerca nelle impostazioni"), { target: { value: "proxy" } });
    expect(await screen.findByLabelText("Proxy HTTPS")).toBeTruthy();
    expect(screen.queryByLabelText("Lingua")).toBeNull();
  });

  it("edits the instructions of a framework and saves only them", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    fireEvent.click(await screen.findByRole("button", { name: "Laravel" }));
    const steps = await screen.findByLabelText("Poi, nella cartella del progetto");
    fireEvent.change(steps, { target: { value: "npm install\nnpm run build" } });
    fireEvent.click(screen.getByRole("button", { name: "Salva" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("frameworks_set", expect.anything());
    });
    const call = invoke.mock.calls.find((c) => c[0] === "frameworks_set");
    expect(JSON.stringify(call?.[1])).toContain('"steps":["npm install","npm run build"]');
    expect(invoke).not.toHaveBeenCalledWith("settings_set", expect.anything());
  });

  it("creates a project with the wizard", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getAllByRole("button", { name: "Nuovo progetto" })[0] as HTMLElement);
    fireEvent.click(await screen.findByRole("radio", { name: /Vite/ }));
    fireEvent.click(await screen.findByRole("checkbox", { name: /React/ }));
    fireEvent.change(screen.getByLabelText("Nome del progetto"), { target: { value: " app " } });
    expect(await screen.findByText("npm create vite@9 {name} -- --template react-ts")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Crea il progetto" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("frameworks_create", {
        base: "vite",
        addons: ["react"],
        name: "app",
        env: [{ key: "APP_NAME", value: "{name}" }],
      });
    });
    expect(await screen.findByText(/Progetto app creato/)).toBeTruthy();
  });

  it("offers the empty projects and those without Git, never a repository", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getAllByRole("button", { name: "Nuovo progetto" })[0] as HTMLElement);
    fireEvent.click(await screen.findByRole("button", { name: /appunti/ }));
    expect(await screen.findByText(/contiene già dei file/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^repo/ })).toBeNull();
    fireEvent.change(screen.getByLabelText("Nome del progetto"), { target: { value: "repo" } });
    expect(await screen.findByText(/è già un repository Git/)).toBeTruthy();
    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Crea il progetto" }).disabled).toBe(true);
    // The .env: a value changed, a variable added.
    fireEvent.change(screen.getByLabelText("Nome del progetto"), { target: { value: "vuoto" } });
    fireEvent.change(await screen.findByLabelText("Valore di APP_NAME"), { target: { value: "Il mio sito" } });
    fireEvent.click(screen.getByRole("button", { name: "Aggiungi una variabile" }));
    const keys = screen.getAllByLabelText("Variabile");
    fireEvent.change(keys[keys.length - 1] as HTMLElement, { target: { value: "APP_URL" } });
    fireEvent.change(screen.getByLabelText("Valore di APP_URL"), { target: { value: "http://localhost:8000" } });
    fireEvent.click(screen.getByRole("button", { name: "Crea il progetto" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith(
        "frameworks_create",
        expect.objectContaining({
          name: "vuoto",
          env: [
            { key: "APP_NAME", value: "Il mio sito" },
            { key: "APP_URL", value: "http://localhost:8000" },
          ],
        }),
      );
    });
  });

  it("asks before closing with something open, and switches language", async () => {
    invoke.mockImplementation(answers({ app_info: { ...info, lang: "en" } }));
    render(<App />);
    expect((await screen.findAllByRole("button", { name: "Projects" })).length).toBeGreaterThan(0);
    act(() => {
      closeListener?.({
        payload: { processes: [], editor: true, terminals: 1, actions: 0, setup: false, databases: 0 },
      });
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
    fireEvent.click(await screen.findByRole("button", { name: "Chiudi NEXKORA" }));
    expect(invoke).toHaveBeenCalledWith("eject_run", undefined);
    expect(invoke).toHaveBeenCalledWith("app_quit", undefined);
  });

  it("shows the progress of an automatic installation", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    await screen.findByRole("heading", { name: "Nuovo progetto" });
    act(() => {
      setupListener?.({
        payload: {
          running: true,
          cancelling: false,
          automatic: true,
          total: 2,
          error: null,
          queue: [],
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
    // In the status bar; the details are in the Activity panel
    expect(screen.getAllByText("Installazione 2/2: PowerShell 7 50%").length).toBe(1);
    fireEvent.click(screen.getByRole("button", { name: /^Attività/ }));
    fireEvent.click(screen.getByRole("button", { name: "Installazione" }));
    expect(screen.getByText("Installazione dei componenti del kit")).toBeTruthy();
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
        unavailableReason: null,
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
        unavailableReason: null,
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
        unavailableReason: null,
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
        unavailableReason: null,
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
        unavailableReason: null,
        needs: [],
      },
      {
        id: "julia",
        name: "Julia",
        group: "languages",
        desc: "calcolo",
        mb: 500,
        chosen: false,
        installed: false,
        present: false,
        locked: false,
        version: null,
        available: false,
        x64Only: false,
        noArm64: false,
        unavailableReason: "non disponibile per linux-x64",
        needs: [],
      },
    ];
    invoke.mockImplementation(
      answers({ app_info: { ...info, firstRun: true }, components_list: comps, components_apply: null }),
    );
    render(<App />);
    expect(await screen.findByText("Benvenuto in NEXKORA")).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>(/^Git(?!Hub)/).disabled).toBe(true);
    expect(screen.getByLabelText<HTMLInputElement>(/^Rstatistica/).disabled).toBe(true);
    expect(screen.getByText("non disponibile su PC ARM64")).toBeTruthy();
    // Not made for this system: the reason from Rust, not "ARM64" (BUG-24c).
    expect(screen.getByLabelText<HTMLInputElement>(/^Juliacalcolo/).disabled).toBe(true);
    expect(screen.getByText("non disponibile per linux-x64")).toBeTruthy();
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
    expect((await screen.findAllByText("2.1.299")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("accesso fatto").length).toBeGreaterThan(0);

    // only the installed agents, and no launch outside a project
    expect(screen.queryByText("Codex")).toBeNull();
    expect(screen.queryByRole("button", { name: "Apri" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Esci" }));
    expect(screen.getByText("Togliere l'accesso salvato nel kit?")).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("agent_sign_out", expect.anything());
  });

  it("opens the Guide from the activity bar and contextual buttons and navigates topics", async () => {
    invoke.mockImplementation(answers({}));
    render(<App />);
    // Open Guide from the bottom-left Activity Bar icon
    const [guideBtn] = screen.getAllByRole("button", { name: "Guida" });
    if (guideBtn) fireEvent.click(guideBtn);
    expect(await screen.findByText(/I 6 principi architetturali cardine/)).toBeTruthy();

    // Jump to the 36-source table
    fireEvent.click(screen.getByRole("button", { name: /Esplora le 36 fonti/ }));
    expect(await screen.findByText(/Catalogo completo delle fonti esterne/)).toBeTruthy();
    expect(screen.getByText("PortableGit-<v>-(64-bit|arm64).7z.exe (estratto in Rust)")).toBeTruthy();

    // Jump to a visual feature topic with screenshots
    fireEvent.click(screen.getByText(/Pagina «Progetti»/));
    expect(await screen.findByText(/05 · Scheda 1 — Avvio/)).toBeTruthy();
  });
});
