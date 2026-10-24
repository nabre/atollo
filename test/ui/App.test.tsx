import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import {
  NOTHING_OPEN,
  SAMPLE_AGENTS,
  SAMPLE_EDITOR,
  SAMPLE_FRAMEWORKS,
  SAMPLE_INFO,
  SAMPLE_PROJECTS,
  SAMPLE_SERVERS,
  SAMPLE_SERVICES,
  SAMPLE_SETTINGS,
  SAMPLE_SETUP,
  SAMPLE_TARGETS,
  SAMPLE_UPDATE,
  type OpenNow,
  type ProjectInfo,
  type RunEvent,
  type RunInfo,
  type ServiceInfo,
  type SetupSnapshot,
  type WebServerPool,
} from "@/api";
import { invoke } from "@test/support/tauri";

// The answers start from the browser preview's (ui/src/api): a test
// changes only what it is about.

/** Default answer of every command; tests override single commands. */
function answers(overrides: Record<string, unknown> = {}): (cmd: string) => Promise<unknown> {
  const base: Record<string, unknown> = {
    app_info: SAMPLE_INFO,
    editor_status: SAMPLE_EDITOR,
    setup_state: SAMPLE_SETUP,
    projects_list: [],
    components_list: [],
    agents_status: [],
    editor_open: null,
    terminal_open: null,
    run_list: [],
    run_ports: { ports: {}, locked: [] },
    url_open: null,
    settings_get: SAMPLE_SETTINGS,
    settings_set: null,
    frameworks_get: SAMPLE_FRAMEWORKS,
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
    frameworks_targets: [...SAMPLE_TARGETS, { name: "repo", state: "git" }],
    update_status: SAMPLE_UPDATE,
    eject_list: NOTHING_OPEN,
    app_quit: null,
    ...overrides,
  };
  return (cmd) => (cmd in base ? Promise.resolve(base[cmd]) : Promise.reject(new Error(`unexpected ${cmd}`)));
}

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

let setupListener: ((e: { payload: SetupSnapshot }) => void) | null = null;
let runListener: ((e: { payload: RunEvent }) => void) | null = null;
let closeListener: ((e: { payload: OpenNow }) => void) | null = null;
let openProjectListener: ((e: { payload: string }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (e: { payload: unknown }) => void) => {
    if (name === "setup") setupListener = handler;
    if (name === "run") runListener = handler;
    if (name === "close-requested") closeListener = handler;
    if (name === "open-project") openProjectListener = handler;
    return Promise.resolve(() => undefined);
  },
}));

const [SITE] = SAMPLE_PROJECTS as [ProjectInfo, ...ProjectInfo[]];
const [MYSQL] = SAMPLE_SERVICES as [ServiceInfo, ...ServiceInfo[]];
const [POOL] = SAMPLE_SERVERS.pools as [WebServerPool, ...WebServerPool[]];
/** How long a lazily loaded page may take to appear the first time. */
const LAZY = 5000;

/** The preview's static site under another name, never opened. */
function project(name: string): ProjectInfo {
  return { ...SITE, id: name, name, settings: {} };
}

const running: RunInfo = {
  key: "sito (1) & co|static",
  project: "sito (1) & co",
  action: "static",
  label: SITE.actions[0]?.label ?? "",
  type: "web",
  command: "Atollo.exe serve . 7100",
  port: 7100,
  url: "http://localhost:7100/",
  state: "running",
  start: 1,
  end: null,
  code: null,
  pid: 7,
  skippedAliases: [],
};

const { App } = await import("@/App");
const { setLang } = await import("@/i18n");

afterEach(() => {
  cleanup();
  setLang("it");
  invoke.mockReset();
  setupListener = null;
  runListener = null;
  closeListener = null;
  openProjectListener = null;
});

describe("App", () => {
  it("shows where the kit lives, in the settings", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    fireEvent.click(await screen.findByRole("button", { name: "Il kit" }));
    expect(await screen.findByText("E:\\Atollo")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("app_info", undefined);
  });

  it("warns when the kit was moved", async () => {
    invoke.mockImplementation(answers({ app_info: { ...SAMPLE_INFO, movedFrom: "F:\\Old" } }));
    render(<App />);
    expect(await screen.findByText(/prima era in F:\\Old/)).toBeTruthy();
  });

  it("warns about kit folders that break the editor watcher", async () => {
    invoke.mockImplementation(answers({ app_info: { ...SAMPLE_INFO, watcherUnsafeFolder: "2026" } }));
    render(<App />);
    expect(await screen.findByText(/La cartella «2026»/)).toBeTruthy();
  });

  it("switches pages from the activity bar", () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "AI" }));
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("AI");
    expect(screen.getByRole("button", { name: "AI" }).getAttribute("aria-current")).toBe("page");
  });

  it("measures its frame with VSCodium's size tokens", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    const classes = (e: Element | null) => (e?.getAttribute("class") ?? "").split(" ");
    // Title bar: 30px, window controls 46px and as high as the bar, no
    // border under it (the modern UI sets no titleBar.border).
    const close = screen.getByRole("button", { name: "Chiudi" });
    expect(classes(close.parentElement)).toContain("h-vsc-titlebar");
    expect(classes(close.parentElement).some((c) => c.startsWith("border"))).toBe(false);
    for (const name of ["Riduci a icona", "Ingrandisci", "Chiudi"]) {
      expect(classes(screen.getByRole("button", { name }))).toEqual(
        expect.arrayContaining(["w-vsc-window-control", "h-full"]),
      );
    }
    // Activity bar: on the window's edge, no border, 36px items 8px apart,
    // the active one on a rounded square with no mark on its left.
    const nav = screen.getByRole("navigation", { name: "Pagine" });
    expect(classes(nav)).toEqual(expect.arrayContaining(["gap-vsc-activitybar-gap", "rounded-r-vsc-medium"]));
    expect(classes(nav).some((c) => c === "border" || c.startsWith("border-"))).toBe(false);
    expect(classes(nav.parentElement)).not.toContain("px-1");
    const projects = within(nav).getByRole("button", { name: "Progetti" });
    expect(projects.getAttribute("aria-current")).toBe("page");
    expect(classes(projects)).toEqual(
      expect.arrayContaining(["size-vsc-activitybar-action", "text-vsc-activitybar-item-active-fg"]),
    );
    expect(projects.children).toHaveLength(1);
    expect(classes(projects.firstElementChild)).toEqual(
      expect.arrayContaining(["rounded-vsc-small", "bg-vsc-activitybar-item-active"]),
    );
    // Status bar: labels as VSCodium's, the bar's height, rounded.
    const bar = screen.getByRole("contentinfo", { name: "Barra di stato" });
    expect(classes(bar)).toEqual(expect.arrayContaining(["px-vsc-size60", "py-vsc-size20"]));
    const item = await within(bar).findByText("win32-x64");
    expect(classes(item)).toEqual(
      expect.arrayContaining([
        "h-full",
        "rounded-vsc-small",
        "mx-vsc-statusbar-item-margin",
        "px-vsc-statusbar-item-padding",
      ]),
    );
    // The last item keeps the bar's edge.
    const version = within(bar).getByText(/^v\d/).closest("button");
    expect(classes(version)).toEqual(expect.arrayContaining(["mr-vsc-size20", "px-vsc-size60"]));
    // One item opens and closes the Activity panel: its counter; the
    // other button is the version, which looks for updates.
    const buttons = within(bar).getAllByRole("button");
    expect(buttons).toHaveLength(2);
    expect(buttons[0]?.textContent).toMatch(/^Attività/);
    expect(buttons[1]).toBe(version);
    expect(buttons[0]?.getAttribute("aria-pressed")).toBe("false");
    fireEvent.click(buttons[0] as HTMLElement);
    expect(buttons[0]?.getAttribute("aria-pressed")).toBe("true");
  });

  it("looks for updates from the version in the status bar", async () => {
    let answer: (st: unknown) => void = () => undefined;
    const base = answers();
    invoke.mockImplementation((cmd) =>
      cmd === "update_check"
        ? new Promise((done) => {
            answer = done;
          })
        : base(cmd),
    );
    render(<App />);
    const bar = screen.getByRole("contentinfo", { name: "Barra di stato" });
    const version = await within(bar).findByRole("button", { name: /^Cerca aggiornamenti/ });
    fireEvent.click(version);
    expect(invoke).toHaveBeenCalledWith("update_check", undefined);
    expect(version.getAttribute("title")).toBe("Ricerca degli aggiornamenti…");
    // A second click while it looks asks nothing more.
    fireEvent.click(version);
    expect(invoke.mock.calls.filter(([c]) => c === "update_check")).toHaveLength(1);
    act(() => {
      answer({ ...SAMPLE_UPDATE, latest: "9.9.9", staged: "9.9.9" });
    });
    const said = "La versione 9.9.9 è pronta: sostituisce questa alla chiusura di Atollo.";
    expect((await within(bar).findByText(said)).getAttribute("role")).toBe("status");
    expect(version.getAttribute("title")).toBe(`${said}\nCerca aggiornamenti`);
    expect(version.textContent).toContain("→ v9.9.9");
  });

  it("says in the status bar when the update check fails", async () => {
    const base = answers();
    invoke.mockImplementation((cmd) => (cmd === "update_check" ? Promise.reject(new Error("offline")) : base(cmd)));
    render(<App />);
    const bar = screen.getByRole("contentinfo", { name: "Barra di stato" });
    fireEvent.click(await within(bar).findByRole("button", { name: /^Cerca aggiornamenti/ }));
    expect((await within(bar).findByText(/^Controllo non riuscito: .*offline/)).getAttribute("role")).toBe("status");
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
    // Esc answers no, as in the other confirmations
    fireEvent.click(screen.getByRole("button", { name: "Elimina da GitHub" }));
    fireEvent.keyDown(screen.getByRole("alertdialog"), { key: "Escape" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Elimina da GitHub" }));
    fireEvent.click(screen.getByRole("button", { name: "Elimina" }));
    expect(await screen.findByRole("button", { name: "Concedi il permesso" })).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("github_delete", expect.anything());
  });

  it("opens editor, console and agent only in a project from Home", async () => {
    // The preview's agents: Claude Code and Codex installed.
    invoke.mockImplementation(
      answers({
        projects_list: [{ ...project("app & co"), languages: ["TypeScript", "CSS"], settings: { agent: "codex" } }],
        agents_status: SAMPLE_AGENTS,
        agent_launch: null,
      }),
    );
    render(<App />);
    const list = within(await screen.findByRole("list", { name: "Progetti" }));
    expect(await list.findByText("app & co")).toBeTruthy();
    // the list shows no language logos (the project's page does)
    expect(list.queryByRole("img", { name: "TypeScript" })).toBeNull();
    fireEvent.click(list.getByRole("button", { name: "Terminale" }));
    expect(invoke).toHaveBeenCalledWith("terminal_open", { project: "app & co" });
    // one button per installed agent, whatever was used last: one click
    // opens that agent (D-146); none for an agent not installed
    expect(list.getByRole("button", { name: "Apri Codex" }).title).toBe("Apre Codex nel progetto app & co");
    expect(list.queryByRole("button", { name: "Apri Antigravity" })).toBeNull();
    fireEvent.click(await list.findByRole("button", { name: "Apri Claude Code" }));
    expect(invoke).toHaveBeenCalledWith("agent_launch", { agent: "claude", project: "app & co" });
    expect(invoke).not.toHaveBeenCalledWith("terminal_open", { project: null });
    // on the project's page, the same buttons, no choice to make first
    fireEvent.click(list.getByTitle("Apre la pagina di app & co"));
    const header = within(await screen.findByRole("toolbar", { name: "Agenti" }));
    expect(screen.queryByRole("radio", { name: "Codex" })).toBeNull();
    fireEvent.click(header.getByRole("button", { name: "Apri Codex" }));
    expect(invoke).toHaveBeenCalledWith("agent_launch", { agent: "codex", project: "app & co" });
  });

  it("shows the error of a launch on that agent's button", async () => {
    const base = answers({ projects_list: [project("app")], agents_status: SAMPLE_AGENTS });
    invoke.mockImplementation((cmd) =>
      cmd === "agent_launch" ? Promise.reject(new Error("Codex non parte")) : base(cmd),
    );
    render(<App />);
    const list = within(await screen.findByRole("list", { name: "Progetti" }));
    fireEvent.click(await list.findByRole("button", { name: "Apri Codex" }));
    await waitFor(() => {
      expect(list.getByRole("button", { name: "Apri Codex" }).title).toMatch(/Codex non parte/);
    });
    expect(list.getByRole("button", { name: "Apri Claude Code" }).title).toBe("Apre Claude Code nel progetto app");
  });

  it("offers no agent button when no agent is installed", async () => {
    invoke.mockImplementation(
      answers({
        projects_list: [project("app")],
        agents_status: SAMPLE_AGENTS.map((a) => ({ ...a, installed: false })),
      }),
    );
    render(<App />);
    const list = within(await screen.findByRole("list", { name: "Progetti" }));
    expect(await list.findByRole("button", { name: "Terminale" })).toBeTruthy();
    expect(list.queryByRole("button", { name: /^Apri (Claude Code|Codex|Antigravity)$/ })).toBeNull();
  });

  it("opens a project's page with its sections and creates its database", async () => {
    invoke.mockImplementation(
      answers({
        projects_list: [{ ...project("negozio"), requires: ["php", "mysql"], missing: ["mysql"] }],
        project_status: { git: { branch: "main", upstream: null, synced: null, remote: null }, instructions: [] },
        project_databases: {
          dbs: [],
          detected: ["mysql"],
          envExists: true,
          envIgnored: true,
          versions: { mysql: { chosen: null, server: "11.4", lines: ["11.4", "10.6"] } },
        },
        project_db_line: null,
        project_db_create: {
          engine: "mysql",
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
    const list = within(await screen.findByRole("list", { name: "Progetti" }));
    fireEvent.click(await list.findByTitle("Apre la pagina di negozio"));
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
      expect(invoke).toHaveBeenCalledWith("project_db_create", { project: "negozio", engine: "mysql" });
    });
    // The version of the server for its database, each on its own port.
    const version = screen.getByLabelText<HTMLElementTagNameMap["vscode-single-select"]>("Versione del server");
    expect(version.querySelectorAll("vscode-option")).toHaveLength(3);
    fireEvent.change(version, { target: { value: "10.6" } });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_db_line", { project: "negozio", engine: "mysql", line: "10.6" });
    });
    // The activity bar's Projects again: back to the list
    fireEvent.click(screen.getAllByRole("button", { name: "Progetti" })[0] as HTMLElement);
    expect((await screen.findAllByRole("heading", { name: "Nuovo progetto" })).length).toBeGreaterThan(0);
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
        run_log: { lines: ["> Atollo.exe serve . 7100"], seq: 1 },
      }),
    );
    render(<App />);
    const list = within(await screen.findByRole("list", { name: "Progetti" }));
    expect(await list.findByText("sito (1) & co")).toBeTruthy();
    fireEvent.click(list.getByTitle("Apre sito (1) & co nell'editor"));
    expect(invoke).toHaveBeenCalledWith("editor_open", { project: "sito (1) & co" });
    // The row starts its default action too (▶)
    expect(list.getByRole("button", { name: "Avvia" })).toBeTruthy();
    fireEvent.click(list.getByTitle("Apre la pagina di sito (1) & co"));
    const main = within(screen.getByRole("main"));
    fireEvent.click((await main.findAllByRole("button", { name: "Avvia" }))[0] as HTMLElement);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("run_start", { project: "sito (1) & co", action: "static" });
    });
    act(() => {
      runListener?.({ payload: { kind: "state", ...running } });
      runListener?.({ payload: { kind: "log", key: running.key, lines: ["pronto"], seq: 2 } });
    });
    // Its state and output in the Activity panel
    fireEvent.click(screen.getByRole("button", { name: /^Attività/ }));
    expect((await screen.findAllByText(/in ascolto/)).length).toBeGreaterThan(0);
    fireEvent.click(screen.getAllByRole("button", { name: "Apri nel browser" })[0] as HTMLElement);
    expect(invoke).toHaveBeenCalledWith("url_open", { url: "http://localhost:7100/" });
  });

  it("saves a setting as it changes, without sending the hidden Gemini key", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    fireEvent.click(await screen.findByRole("button", { name: "Rete" }));
    const proxy = await screen.findByLabelText("Proxy HTTPS");
    fireEvent.change(proxy, { target: { value: " http://proxy:8080 " } });
    // No Save button: the change is saved a moment later (D-137).
    expect(screen.queryByRole("button", { name: "Salva" })).toBeNull();
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("settings_set", expect.objectContaining({ gemini: null }));
    });
    const call = invoke.mock.calls.find((c) => c[0] === "settings_set");
    expect(JSON.stringify(call?.[1])).toContain('"httpsProxy":"http://proxy:8080"');
    expect(await screen.findByText("Salvato.")).toBeTruthy();
  });

  it("saves a Gemini key typed on Antigravity's page when the field loses the focus", async () => {
    const agents = SAMPLE_AGENTS.map((a) => (a.id === "antigravity" ? { ...a, installed: true } : a));
    invoke.mockImplementation(answers({ agents_status: agents }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "AI" }));
    const panes = within(await screen.findByRole("list", { name: "Agenti" }));
    fireEvent.click(await panes.findByText("Antigravity"));
    const key = await screen.findByLabelText("Chiave API di Gemini (Antigravity senza account Google)");
    fireEvent.change(key, { target: { value: "AIza-key" } });
    expect(invoke).not.toHaveBeenCalledWith("settings_set", expect.anything());
    fireEvent.blur(key);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("settings_set", expect.objectContaining({ gemini: "AIza-key" }));
    });
  });

  it("changes the theme at once, with no Save", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    const theme = await screen.findByLabelText("Tema");
    fireEvent.change(theme, { target: { value: "dark" } });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("settings_set", expect.objectContaining({ gemini: null }));
    });
    const call = invoke.mock.calls.find((c) => c[0] === "settings_set");
    expect(JSON.stringify(call?.[1])).toContain('"theme":"dark"');
  });

  it("saves who signs the commits when a field loses the focus", async () => {
    invoke.mockImplementation(
      answers({
        git_identity_get: { name: "", email: "" },
        git_identity_set: { name: "Ada", email: "ada@example.org" },
        github_status: { installed: true, signedIn: false, login: null, signingIn: false, code: null, error: null },
      }),
    );
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    const sections = within(await screen.findByRole("list", { name: "Impostazioni" }));
    fireEvent.click(sections.getByRole("button", { name: "Account GitHub" }));
    const name = await screen.findByLabelText("Nome");
    fireEvent.change(name, { target: { value: "Ada" } });
    fireEvent.change(screen.getByLabelText("E-mail"), { target: { value: "ada@example.org" } });
    fireEvent.blur(name);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("git_identity_set", {
        identity: { name: "Ada", email: "ada@example.org" },
      });
    });
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

  it("edits what an addition does with one base, from under the base in the tree", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    const tree = within(await screen.findByRole("list", { name: "Framework" }));
    // Group → base → slot → addition
    fireEvent.click(await tree.findByRole("button", { name: /^Vite/ }));
    const under = within(await tree.findByRole("list", { name: "Aggiunte con Vite" }));
    expect(under.getByRole("button", { name: /Interfaccia/ })).toBeTruthy();
    fireEvent.click(under.getByRole("button", { name: /React/ }));
    expect(await screen.findByRole("heading", { name: "React con Vite" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /React/, selected: true })).toBeTruthy();
    const create = screen.getByLabelText("Crea il progetto al posto della base (facoltativo)");
    fireEvent.change(create, { target: { value: "npm create vite@9 {name} -- --template react-swc-ts" } });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("frameworks_set", expect.anything());
    });
    const call = invoke.mock.calls.find((c) => c[0] === "frameworks_set");
    expect(JSON.stringify(call?.[1])).toContain("react-swc-ts");
  });

  it("edits the instructions of a framework and saves only them", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    fireEvent.click(await screen.findByRole("button", { name: "Laravel" }));
    const steps = await screen.findByLabelText("Poi, nella cartella del progetto");
    fireEvent.change(steps, { target: { value: "npm install\nnpm run build" } });
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
    fireEvent.click(await screen.findByRole("button", { name: "Nuovo progetto da una base" }));
    const groups = within(await screen.findByRole("group", { name: "Gruppi di basi" }));
    fireEvent.click(groups.getByRole("button", { name: /JavaScript/ }));
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

  it("starts, pins and finds the projects from the home", async () => {
    invoke.mockImplementation(
      answers({
        projects_list: [
          project("sito (1) & co"),
          { ...project("negozio"), requires: ["php", "mysql"], missing: ["mysql"] },
        ],
        run_start: { ...running, state: "starting" },
        run_log: { lines: [], seq: 0 },
        project_pin: null,
      }),
    );
    render(<App />);
    expect(await screen.findByRole("tab", { name: "Pannello rapido", selected: true })).toBeTruthy();
    const recent = within(await screen.findByRole("region", { name: "Recenti" }));
    // The card starts the default action, the star makes it a favorite
    fireEvent.click(recent.getAllByRole("button", { name: "Avvia" })[0] as HTMLElement);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("run_start", expect.objectContaining({ action: "static" }));
    });
    fireEvent.click(recent.getByRole("button", { name: "Aggiungi negozio ai preferiti" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_pin", { project: "negozio", pinned: true });
    });
    // What runs, with its stop; what is missing, with the way to its page
    act(() => {
      runListener?.({ payload: { kind: "state", ...running } });
    });
    const live = within(await screen.findByRole("region", { name: "In esecuzione" }));
    expect(live.getByText("sito (1) & co")).toBeTruthy();
    expect(screen.getByText(/negozio: mancano/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Installa" }));
    expect(await screen.findByText("da installare")).toBeTruthy();
    // Ctrl+P: a project by its name, Enter opens its page
    fireEvent.keyDown(window, { key: "p", ctrlKey: true });
    const box = within(await screen.findByRole("dialog", { name: "Apri un progetto" }));
    fireEvent.change(box.getByLabelText("Apri un progetto"), { target: { value: "(1)" } });
    expect(box.getAllByRole("option")).toHaveLength(1);
    fireEvent.keyDown(box.getByRole("option"), { key: "Enter" });
    expect(await screen.findByRole("tab", { name: "sito (1) & co", selected: true })).toBeTruthy();
    expect(screen.queryByRole("dialog", { name: "Apri un progetto" })).toBeNull();
    // Ctrl+Shift+P: the command palette, a page by its name.
    fireEvent.keyDown(window, { key: "P", ctrlKey: true, shiftKey: true });
    const commands = within(await screen.findByRole("dialog", { name: "Comandi" }));
    fireEvent.change(commands.getByLabelText("Comandi"), { target: { value: "vai impostazioni" } });
    expect(commands.getAllByRole("option")).toHaveLength(1);
    fireEvent.keyDown(commands.getByRole("option"), { key: "Enter" });
    expect(screen.queryByRole("dialog", { name: "Comandi" })).toBeNull();
    expect(await screen.findByRole("heading", { name: "Impostazioni", level: 1 })).toBeTruthy();
  });

  it("opens each way of making a project from the side bar's tree", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    const projectsPane = () =>
      screen.getByRole("button", { name: "Progetti", expanded: true }).closest("section") as HTMLElement;
    // Alone, «Progetti» fills the side bar; beside another open pane it is
    // as tall as its rows, the rest to the other one.
    await screen.findByRole("button", { name: "Progetti", expanded: true });
    expect(projectsPane().hasAttribute("data-fit")).toBe(false);
    fireEvent.click(await screen.findByRole("button", { name: "Nuovo progetto", expanded: false }));
    const tree = await screen.findByRole("list", { name: "Nuovo progetto" });
    expect(projectsPane().hasAttribute("data-fit")).toBe(true);
    expect(tree.closest("section")?.hasAttribute("data-fit")).toBe(false);
    fireEvent.click(within(tree).getByRole("button", { name: /Progetto vuoto/ }));
    expect(await screen.findByRole("button", { name: "Crea" })).toBeTruthy();
    fireEvent.click(within(tree).getByRole("button", { name: /Da un indirizzo Git/ }));
    expect(await screen.findByRole("button", { name: /Clona/ })).toBeTruthy();
    // A group opens the wizard on its first base, its branch open; the tab
    // names the group.
    fireEvent.click(await within(tree).findByRole("button", { name: /^Web\s*2$/ }));
    expect(await screen.findByRole("tab", { name: /Nuovo progetto\s*Web/ })).toBeTruthy();
    expect(
      within(tree)
        .getByRole("button", { name: /^Web\s*2$/ })
        .getAttribute("aria-expanded"),
    ).toBe("true");
    // A base under a category: the wizard opens with it chosen.
    fireEvent.click(await within(tree).findByRole("button", { name: /Web con JavaScript e TypeScript/ }));
    fireEvent.click(await within(tree).findByRole("button", { name: /Vite/ }));
    await waitFor(() => {
      expect(screen.getByRole<HTMLInputElement>("radio", { name: /Vite/ }).checked).toBe(true);
    });
    expect(screen.getAllByRole("tab", { name: /Nuovo progetto/ })).toHaveLength(1);
  });

  it("offers the empty projects and those without Git, never a repository", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Nuovo progetto da una base" }));
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
    invoke.mockImplementation(answers({ app_info: { ...SAMPLE_INFO, lang: "en" } }));
    render(<App />);
    expect((await screen.findAllByRole("button", { name: "Projects" })).length).toBeGreaterThan(0);
    act(() => {
      closeListener?.({
        payload: { ...NOTHING_OPEN, editor: true, terminals: 1 },
      });
    });
    const dialog = await screen.findByRole("alertdialog");
    expect(dialog.textContent).toContain("Kit consoles: 1");
    fireEvent.click(screen.getByRole("button", { name: "Close everything" }));
    expect(invoke).toHaveBeenCalledWith("app_quit", undefined);
  });

  it("opens the project chosen in the tray menu, from any page", async () => {
    invoke.mockImplementation(answers({ projects_list: [project("negozio"), project("blog")] }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "AI" }));
    act(() => {
      openProjectListener?.({ payload: "negozio" });
    });
    expect(await screen.findByRole("tab", { name: /negozio/, selected: true })).toBeTruthy();
    const nav = within(screen.getByRole("navigation", { name: "Pagine" }));
    expect(nav.getByRole("button", { name: "Progetti" }).getAttribute("aria-current")).toBe("page");
  });

  it("reads the list again for a project of the tray menu it does not know yet", async () => {
    const before = answers({ projects_list: [project("negozio")] });
    const after = answers({ projects_list: [project("negozio"), project("nuovo")] });
    let reads = 0;
    invoke.mockImplementation((cmd) => (cmd === "projects_list" && reads++ > 0 ? after : before)(cmd));
    render(<App />);
    expect((await screen.findAllByText("negozio")).length).toBeGreaterThan(0);
    act(() => {
      openProjectListener?.({ payload: "nuovo" });
    });
    expect(await screen.findByRole("heading", { name: "nuovo" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /nuovo/, selected: true })).toBeTruthy();
    expect(reads).toBe(2);
  });

  it("prepares the drive for removal", async () => {
    const open = {
      ...NOTHING_OPEN,
      processes: [{ pid: 42, name: "node.exe", program: "data\apps\win32-x64\node\node.exe" }],
    };
    invoke.mockImplementation(answers({ eject_list: open, eject_run: NOTHING_OPEN, app_quit: null }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Rimuovi l'unità" }));
    expect(await screen.findByText("node.exe")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Chiudi tutto e prepara la rimozione" }));
    fireEvent.click(await screen.findByRole("button", { name: "Chiudi Atollo" }));
    expect(invoke).toHaveBeenCalledWith("eject_run", undefined);
    expect(invoke).toHaveBeenCalledWith("app_quit", undefined);
  });

  it("clears a failed read of what is open when «Aggiorna» reads it", async () => {
    let fail = true;
    invoke.mockImplementation((cmd) =>
      cmd === "eject_list" && fail ? Promise.reject(new Error("letto male")) : answers()(cmd),
    );
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Rimuovi l'unità" }));
    expect(await screen.findByText("Error: letto male")).toBeTruthy();
    fail = false;
    fireEvent.click(screen.getByRole("button", { name: "Aggiorna" }));
    expect(await screen.findByText("Nessun programma del kit aperto.")).toBeTruthy();
    expect(screen.queryByText("Error: letto male")).toBeNull();
  });

  it("clears a failed read of the settings with «Riprova»", async () => {
    let fail = true;
    invoke.mockImplementation((cmd) =>
      cmd === "settings_get" && fail ? Promise.reject(new Error("letto male")) : answers()(cmd),
    );
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Impostazioni" }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Error: letto male");
    fail = false;
    fireEvent.click(within(alert).getByRole("button", { name: "Riprova" }));
    expect(await screen.findByLabelText("Lingua")).toBeTruthy();
    expect(screen.queryByText(/letto male/)).toBeNull();
  });

  it("shows the progress of an automatic installation", async () => {
    invoke.mockImplementation(answers());
    render(<App />);
    await screen.findAllByRole("heading", { name: "Nuovo progetto" });
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

  it("counts a database version running beside a stopped main one, and an active pool (UX-V5-01)", async () => {
    // The preview's MySQL the other way round: 9.7 stopped, 8.4 running.
    const mysql: ServiceInfo = {
      ...MYSQL,
      state: "stopped",
      others: MYSQL.others.map((o) => ({ ...o, state: "running" })),
    };
    const pool: WebServerPool = { ...POOL, running: true };
    const servers = (pools: WebServerPool[]) => ({ ...SAMPLE_SERVERS, pools });
    invoke.mockImplementation(answers({ services_list: [mysql], servers_list: servers([]) }));
    render(<App />);
    const bar = within(screen.getByRole("contentinfo", { name: "Barra di stato" }));
    expect(await bar.findByLabelText("In esecuzione: 1")).toBeTruthy();
    // The same count on the Projects icon
    const projects = screen.getByTitle("Progetti - In esecuzione: 1");
    // 3px from the item's right and 2px from its bottom, as the modern
    // UI's .badge-content (top 18px of the 36px item, 16px high).
    const badge = within(projects).getByLabelText("In esecuzione: 1");
    expect((badge.getAttribute("class") ?? "").split(" ")).toEqual(
      expect.arrayContaining([
        "right-[calc(var(--vscode-spacing-size20)+var(--vscode-strokeThickness))]",
        "bottom-vsc-size20",
        "h-vsc-size160",
      ]),
    );
    cleanup();

    invoke.mockImplementation(answers({ services_list: [mysql], servers_list: servers([pool]) }));
    render(<App />);
    const again = within(screen.getByRole("contentinfo", { name: "Barra di stato" }));
    expect(await again.findByLabelText("In esecuzione: 2")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^Attività/ }));
    const list = within(screen.getByRole("list", { name: "Attività" }));
    expect(list.getByText("MySQL 8.4")).toBeTruthy();
    expect(list.getByText(pool.name)).toBeTruthy();
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
      answers({ app_info: { ...SAMPLE_INFO, firstRun: true }, components_list: comps, components_apply: null }),
    );
    render(<App />);
    // The first start is a lazily loaded page: its first import can take
    // over the default second on a Windows runner.
    expect(await screen.findByText("Benvenuto in Atollo", {}, { timeout: LAZY })).toBeTruthy();
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
    // The preview's agents: Claude Code signed in, Codex not, Antigravity
    // not installed; none running, so the sign-out is offered.
    const agents = SAMPLE_AGENTS.map((a) => ({ ...a, running: false }));
    invoke.mockImplementation(answers({ agents_status: agents, agent_launch: null, agent_sign_out: null }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "AI" }));
    expect((await screen.findAllByText("2.1.299")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("accesso fatto").length).toBeGreaterThan(0);

    // only the installed agents, and no launch outside a project
    expect(screen.getAllByText("Codex").length).toBeGreaterThan(0);
    expect(screen.queryByText("Antigravity")).toBeNull();
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
    // It opens on the welcome topic, with a card per chapter
    expect(await screen.findByText("Che cos'è, in parole semplici", {}, { timeout: LAZY })).toBeTruthy();
    expect(screen.getByText("Come funziona dentro")).toBeTruthy();

    // The search narrows the topics; the sources table counts its rows
    fireEvent.change(screen.getByLabelText("Cerca nella guida"), { target: { value: "sha-256" } });
    fireEvent.click(await screen.findByText("Da dove arrivano gli strumenti"));
    expect(await screen.findByText("Tabella completa delle fonti (39 componenti e 5 parti)")).toBeTruthy();
    expect(screen.getByText("PortableGit-<v>-(64-bit|arm64).7z.exe (estratto in Rust)")).toBeTruthy();
    expect(screen.getByText(/macOS e Linux: GitHub desktop\/dugite-native/)).toBeTruthy();

    // A link inside a topic opens another topic
    fireEvent.click(screen.getByRole("button", { name: /Rete aziendale: proxy, certificati, rete locale/ }));
    expect(await screen.findByText("Capire se ti servono")).toBeTruthy();
  });
});
