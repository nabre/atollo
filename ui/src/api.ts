// Typed wrappers of the Rust commands (src-tauri/src/commands.rs). The
// interface calls nothing else: no generic "run this" command exists.
//
// Outside Tauri (`npm run dev` in ui\ opened in a browser) the commands do
// not exist: the wrappers answer with sample data, so the pages can be
// designed in the browser, and the interface shows a "preview" banner.
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface AppInfo {
  version: string;
  root: string;
  data: string;
  projects: string;
  platform: string;
  lang: "it" | "en" | "fr" | "de";
  movedFrom: string | null;
  webview: string;
  pathEntries: number;
  /** A folder of the kit path that stops the editor noticing outside changes. */
  watcherUnsafeFolder: string | null;
  /** No choice saved yet: first start. */
  firstRun: boolean;
  /** Other copies of the kit with programs open on this PC. */
  otherKits: string[];
}

/** True when the page runs in the browser instead of the Atollo window. */
export const browserPreview = !isTauri();

const SAMPLE_INFO: AppInfo = {
  version: "0.1.0",
  root: "E:\\Atollo",
  data: "E:\\Atollo\\data",
  projects: "E:\\Atollo\\projects",
  platform: "win32-x64",
  lang: "it",
  movedFrom: null,
  webview: "anteprima nel browser",
  pathEntries: 0,
  watcherUnsafeFolder: null,
  firstRun: false,
  otherKits: [],
};

export function appInfo(): Promise<AppInfo> {
  return browserPreview ? Promise.resolve(SAMPLE_INFO) : invoke<AppInfo>("app_info");
}

export interface EditorStatus {
  installed: boolean;
  running: boolean;
  port: number | null;
  /** The editor server is x64 only: emulated on ARM64 PCs. */
  emulated: boolean;
}

export type ActionKind = "web" | "desktop" | "command";

/** An action of a project (src-tauri/src/detect). */
export interface ProjectAction {
  id: string;
  label: string;
  command: string;
  type: ActionKind;
  folder: string | null;
  url: string | null;
  env: Record<string, string>;
  /** A folder served by the app's file server. */
  static: string | null;
  custom: boolean;
}

export interface ProjectInfo {
  /** Folder name in projects\. */
  id: string;
  /** Name shown (the folder, or "name" in panel.json). */
  name: string;
  /** Seconds since 1970. */
  modified: number;
  empty: boolean;
  /** Components it needs that are not installed. */
  missing: string[];
  platform: string;
  type: ActionKind;
  reason: string;
  languages: string[];
  requires: string[];
  actions: ProjectAction[];
  warnings: string[];
  custom: boolean;
  default: string | null;
  /** The kit's settings of the project (data\app\projects\<id>\dev-settings.json). */
  settings: { agent?: string; lastOpened?: number };
}

function sampleAction(id: string, label: string, command: string, staticDir: string | null): ProjectAction {
  return { id, label, command, type: "web", folder: null, url: null, env: {}, static: staticDir, custom: false };
}

const SAMPLE_PROJECTS: ProjectInfo[] = [
  {
    id: "sito-vetrina",
    name: "sito-vetrina",
    modified: 1_791_900_000,
    empty: false,
    missing: [],
    platform: "Sito statico",
    type: "web",
    reason: "index.html",
    languages: ["HTML", "CSS"],
    requires: [],
    actions: [sampleAction("static", "sito statico (cartella del progetto)", "", ".")],
    warnings: [],
    custom: false,
    default: "static",
    settings: { lastOpened: Math.floor(Date.now() / 1000) - 3600 },
  },
  {
    id: "api-python",
    name: "api-python",
    modified: 1_791_800_000,
    empty: false,
    missing: ["python"],
    platform: "FastAPI",
    type: "web",
    reason: "Rilevato da: main.py crea FastAPI()",
    languages: ["Python"],
    requires: ["python"],
    actions: [
      sampleAction(
        "py:uvicorn",
        "uvicorn main:app",
        "python -m uvicorn main:app --host 127.0.0.1 --port {port} --reload",
        null,
      ),
    ],
    warnings: ["Nessun ambiente virtuale (.venv)."],
    custom: false,
    default: "py:uvicorn",
    settings: { agent: "codex" },
  },
];

export function editorStatus(): Promise<EditorStatus> {
  return browserPreview
    ? Promise.resolve({ installed: true, running: false, port: null, emulated: false })
    : invoke<EditorStatus>("editor_status");
}

/** Opens the editor on a project (`null` = the projects folder). */
export function editorOpen(project: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("editor_open", { project });
}

export function editorRestart(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("editor_restart");
}

export function projectsList(): Promise<ProjectInfo[]> {
  return browserPreview ? Promise.resolve(SAMPLE_PROJECTS) : invoke<ProjectInfo[]>("projects_list");
}

/** Opens a kit console (a console window of the app) in a project. */
export function terminalOpen(project: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("terminal_open", { project });
}

// Preparation (src-tauri/src/setup_task.rs): one run at a time, started by
// the app at launch when a required component is missing, by the editor and
// console buttons, or by the "Linguaggi e strumenti" page.

export type Outcome =
  | { kind: "installed"; version: string }
  | { kind: "upToDate"; version: string | null }
  | { kind: "removed" }
  | { kind: "inUse" }
  | { kind: "unsupported" }
  | { kind: "cancelled" }
  | { kind: "failed"; message: string };

export interface SetupStep {
  id: string;
  name: string;
  note: string | null;
  done: number | null;
  size: number | null;
  outcome: Outcome | null;
}

export interface SetupSnapshot {
  running: boolean;
  cancelling: boolean;
  automatic: boolean;
  total: number;
  steps: SetupStep[];
  error: string | null;
}

/** `app`: part of Atollo (the editor), shown with the required ones. */
export type Group = "required" | "app" | "agents" | "languages";

export interface ComponentInfo {
  id: string;
  name: string;
  group: Group;
  desc: string;
  mb: number;
  chosen: boolean;
  installed: boolean;
  /** Something of it is on disk, even half installed: it can be removed. */
  present: boolean;
  /** A system component, or needed by one (Git for GitHub CLI): always installed. */
  locked: boolean;
  version: string | null;
  /** This version of the app can install it on this kit. */
  available: boolean;
  x64Only: boolean;
  /** No ARM64 build and this kit is ARM64: why it is not available. */
  noArm64: boolean;
  needs: string[];
}

const IDLE: SetupSnapshot = { running: false, cancelling: false, automatic: false, total: 0, steps: [], error: null };

export function setupState(): Promise<SetupSnapshot> {
  return browserPreview ? Promise.resolve(IDLE) : invoke<SetupSnapshot>("setup_state");
}

/** Every change of the preparation; returns the function that stops listening. */
export function onSetup(handler: (s: SetupSnapshot) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return listen<SetupSnapshot>("setup", (e) => {
    handler(e.payload);
  });
}

/** The editor started, stopped or restarted; returns the function that stops listening. */
export function onEditorStatus(handler: (s: EditorStatus) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return listen<EditorStatus>("editor-status", (e) => {
    handler(e.payload);
  });
}

export function componentsList(): Promise<ComponentInfo[]> {
  return browserPreview ? Promise.resolve([]) : invoke<ComponentInfo[]>("components_list");
}

export function setupStart(only: string[] | null, updateAll: boolean): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("setup_start", { only, updateAll });
}

export function setupCancel(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("setup_cancel");
}

/** Saves the choice (ids to keep installed) and installs/removes in the background. */
export function componentsApply(chosen: string[], updateAll: boolean): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("components_apply", { chosen, updateAll });
}

export type AgentId = "claude" | "codex" | "antigravity";

export interface AgentInfo {
  id: AgentId;
  name: string;
  installed: boolean;
  version: string | null;
  signedIn: boolean;
  /** "account" or "api key". */
  signInKind: string | null;
  running: boolean;
}

export function agentsStatus(): Promise<AgentInfo[]> {
  return browserPreview ? Promise.resolve([]) : invoke<AgentInfo[]>("agents_status");
}

// Plan limits and usage of the agents (src-tauri/src/stats).

export type Level = "none" | "ok" | "warning" | "critical";

export interface LimitWindow {
  label: string;
  /** % used, 0-100. */
  percent: number | null;
  /** Milliseconds since 1970. */
  resetsAt: number | null;
  level: Level;
}

export interface Limits {
  plan: string | null;
  windows: LimitWindow[];
  credits: { used: number; limit: number | null; currency: string; percent: number | null; level: Level } | null;
  source: string | null;
  note: string | null;
  error: string | null;
  updated: number | null;
  limited: boolean;
  retryAt: number | null;
}

export interface Tokens {
  input: number;
  cached: number;
  output: number;
  total: number;
}

export interface Usage {
  /** False when the agent saves no token counts (Antigravity). */
  tokens: boolean;
  since: number;
  today: Tokens;
  week: Tokens;
  sessionsToday: number;
  sessionsWeek: number;
  last: number | null;
  models: Record<string, number>;
  days: (Tokens & { start: number; sessions: number })[];
  error?: string;
}

export interface AgentStats {
  id: AgentId;
  name: string;
  account: "none" | "expired" | "apikey" | "ok";
  limits: Limits;
  usage: Usage;
  peak: number | null;
  peakLabel: string | null;
  level: Level;
}

export interface StatsReport {
  updated: number;
  warning: number;
  critical: number;
  agents: AgentStats[];
}

function sampleStats(): StatsReport {
  const day = 24 * 3600 * 1000;
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const tokens = (n: number): Tokens => ({ input: n, cached: n * 8, output: n / 4, total: n * 9.25 });
  const days = Array.from({ length: 7 }, (_, i) => ({
    start: today.getTime() - (6 - i) * day,
    ...tokens((i + 1) * 20000),
    sessions: i % 3,
  }));
  const usage: Usage = {
    tokens: true,
    since: days[0]?.start ?? 0,
    today: tokens(140000),
    week: tokens(560000),
    sessionsToday: 2,
    sessionsWeek: 6,
    last: Date.now() - 600000,
    models: { "claude-opus": 4000000, "claude-haiku": 1180000 },
    days,
  };
  const limits = (windows: LimitWindow[]): Limits => ({
    plan: "max",
    windows,
    credits: null,
    source: "server di Anthropic",
    note: null,
    error: null,
    updated: Date.now(),
    limited: false,
    retryAt: null,
  });
  return {
    updated: Date.now(),
    warning: 70,
    critical: 90,
    agents: [
      {
        id: "claude",
        name: "Claude Code",
        account: "ok",
        limits: limits([
          { label: "5 ore", percent: 34, resetsAt: Date.now() + 2 * 3600000, level: "ok" },
          { label: "Settimana", percent: 76, resetsAt: Date.now() + 3 * day, level: "warning" },
        ]),
        usage,
        peak: 76,
        peakLabel: "Settimana",
        level: "warning",
      },
    ],
  };
}

/** Limits and usage of the installed agents; `force`: the Refresh button. */
export function agentsStats(force: boolean): Promise<StatsReport> {
  return browserPreview ? Promise.resolve(sampleStats()) : invoke<StatsReport>("agents_stats", { force });
}

/** Opens the agent in a kit console in the project (only there). */
export function agentLaunch(agent: AgentId, project: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("agent_launch", { agent, project });
}

export function agentSignOut(agent: AgentId): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("agent_sign_out", { agent });
}

/** Opens a kit folder in File Explorer: "agents" or "projects" (+ project). */
export function folderOpen(which: "agents" | "projects", project: string | null = null): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("folder_open", { which, project });
}

// Projects (src-tauri/src/projects.rs) and their actions (run.rs).

/** A new empty project; returns its folder name. */
export function projectCreate(name: string): Promise<string> {
  return browserPreview ? Promise.resolve(name) : invoke<string>("project_create", { name });
}

/** Deletes a project whose folder is empty. */
export function projectDelete(project: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_delete", { project });
}

/** Clones `owner/repo` or an https:// Git address; returns the folder name. */
export function projectClone(source: string, name: string | null): Promise<string> {
  return browserPreview ? Promise.resolve(name ?? "clone") : invoke<string>("project_clone", { source, name });
}

export type SiteKind = "static" | "php" | "none";

/** "Servi come sito web": writes the "site" action in the project's panel.json. */
export function projectSite(project: string, folder: string, kind: SiteKind): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_site", { project, folder, kind });
}

export type RunState = "starting" | "running" | "finished" | "error" | "stopped";

export interface RunInfo {
  /** "<project>|<action>". */
  key: string;
  project: string;
  action: string;
  label: string;
  type: ActionKind;
  command: string;
  port: number | null;
  url: string | null;
  state: RunState;
  start: number;
  end: number | null;
  code: number | null;
  pid: number | null;
}

export type RunEvent = ({ kind: "state" } & RunInfo) | { kind: "log"; key: string; lines: string[] };

export interface Ports {
  ports: Record<string, number>;
  locked: string[];
}

export function runList(): Promise<RunInfo[]> {
  return browserPreview ? Promise.resolve([]) : invoke<RunInfo[]>("run_list");
}

export function runStart(project: string, action: string): Promise<RunInfo> {
  return invoke<RunInfo>("run_start", { project, action });
}

export function runStop(key: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("run_stop", { key });
}

export function runLog(key: string): Promise<string[]> {
  return browserPreview ? Promise.resolve([]) : invoke<string[]>("run_log", { key });
}

export function runClear(key: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("run_clear", { key });
}

export function runPorts(): Promise<Ports> {
  return browserPreview ? Promise.resolve({ ports: {}, locked: [] }) : invoke<Ports>("run_ports");
}

/** The user's port for an action (`null`: automatic again). */
export function runSetPort(key: string, port: number | null): Promise<Ports> {
  return invoke<Ports>("run_set_port", { key, port });
}

/** State changes and output of the projects' actions. */
export function onRun(handler: (e: RunEvent) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return listen<RunEvent>("run", (e) => {
    handler(e.payload);
  });
}

/** Opens a local address (http://localhost:<port>/) in the PC's browser. */
export function urlOpen(url: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("url_open", { url });
}

// Settings (src-tauri/src/settings.rs).

export interface Settings {
  lang: "it" | "en" | "fr" | "de" | null;
  network: {
    httpsProxy: string | null;
    httpProxy: string | null;
    noProxy: string | null;
    extraCaCerts: string | null;
    allowLan: boolean;
  };
  /** Never sent to the interface: see `geminiKeySet`. */
  geminiApiKey: string | null;
  autoUpdate: boolean;
  pythonVersion: string | null;
  phpVersion: string | null;
  rustToolchain: "gnu" | "msvc" | null;
  /** `null` = the kit's Windows Terminal; `classic` = Windows' classic console. */
  terminal: "classic" | null;
}

export interface SettingsView {
  settings: Settings;
  geminiKeySet: boolean;
  /** The file could not be read: defaults shown. */
  problem: string | null;
}

const SAMPLE_SETTINGS: SettingsView = {
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
  geminiKeySet: false,
  problem: null,
};

export function settingsGet(): Promise<SettingsView> {
  return browserPreview ? Promise.resolve(SAMPLE_SETTINGS) : invoke<SettingsView>("settings_get");
}

/** `gemini`: null keeps the saved key, "" removes it. */
export function settingsSet(settings: Settings, gemini: string | null): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("settings_set", { settings, gemini });
}

export interface UpdateStatus {
  current: string;
  latest: string | null;
  /** Downloaded: replaces the exe when the app closes. */
  staged: string | null;
  checking: boolean;
  error: string | null;
}

export function updateStatus(): Promise<UpdateStatus> {
  return browserPreview
    ? Promise.resolve({ current: "0.1.0", latest: null, staged: null, checking: false, error: null })
    : invoke<UpdateStatus>("update_status");
}

export function updateCheck(): Promise<UpdateStatus> {
  return browserPreview ? updateStatus() : invoke<UpdateStatus>("update_check");
}

// Removing the drive and closing (src-tauri/src/sessions.rs).

export interface KitProcess {
  pid: number;
  name: string;
  /** Relative to the kit folder. */
  program: string;
}

export interface OpenNow {
  processes: KitProcess[];
  editor: boolean;
  terminals: number;
  actions: number;
  setup: boolean;
}

const NOTHING_OPEN: OpenNow = { processes: [], editor: false, terminals: 0, actions: 0, setup: false };

export function ejectList(): Promise<OpenNow> {
  return browserPreview ? Promise.resolve(NOTHING_OPEN) : invoke<OpenNow>("eject_list");
}

/** Stops every program of the kit and saves Antigravity's login in the kit. */
export function ejectRun(): Promise<OpenNow> {
  return browserPreview ? Promise.resolve(NOTHING_OPEN) : invoke<OpenNow>("eject_run");
}

/** Closes the app (after the user confirmed). */
export function appQuit(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("app_quit");
}

/** The window is closing with something open: the interface asks. */
export function onCloseRequested(handler: (open: OpenNow) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return listen<OpenNow>("close-requested", (e) => {
    handler(e.payload);
  });
}
