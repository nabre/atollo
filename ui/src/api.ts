// Typed wrappers of the Rust commands (src-tauri/src/commands.rs). The
// interface calls nothing else: no generic "run this" command exists.
//
// Outside Tauri (`npm run dev` in ui\ opened in a browser) the commands do
// not exist: the wrappers answer with sample data, so the pages can be
// designed in the browser, and the interface shows a "preview" banner.
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";

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

/** True when the page runs in the browser instead of the Dev-Kit window. */
export const browserPreview = !isTauri();

const SAMPLE_INFO: AppInfo = {
  version: "0.1.0",
  root: "E:\\Dev-Kit",
  data: "E:\\Dev-Kit\\data",
  projects: "E:\\Dev-Kit\\projects",
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
  /** A folder served by an Apache of the project's own. */
  apache: string | null;
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
  settings: { agent?: string; lastOpened?: number; extensions?: { added: string[]; removed: string[] } };
  /** Editor extensions of its languages, then those added by hand. */
  extensions: ProjectExtension[];
}

/** An editor extension of a project (src-tauri/src/editor_ext.rs). */
export interface ProjectExtension {
  /** `publisher.name`, lower case. */
  id: string;
  name: string;
  /** One of its languages or tools brings it. */
  byDefault: boolean;
  /** Wanted: a default not removed, or added by hand. */
  on: boolean;
  installed: boolean;
}

/** A row of the kit's matrix language-extension. */
export interface MatrixExtension {
  id: string;
  name: string;
  languages: string[];
  /** Ids of components a project requires (Deno, Bun, CMake). */
  tools: string[];
  /** General only: installed when one of these components is in the kit. */
  programs: string[];
  /** Not a default: only added by hand to a project. */
  optional: boolean;
  installed: boolean;
}

export interface ExtensionsMatrix {
  /** Columns: the languages, then the tools as [id, name]. */
  languages: string[];
  tools: [string, string][];
  extensions: MatrixExtension[];
}

function sampleAction(id: string, label: string, command: string, staticDir: string | null): ProjectAction {
  return {
    id,
    label,
    command,
    type: "web",
    folder: null,
    url: null,
    env: {},
    static: staticDir,
    apache: null,
    custom: false,
  };
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
    extensions: [
      { id: "esbenp.prettier-vscode", name: "Prettier", byDefault: true, on: true, installed: true },
      { id: "bradlc.vscode-tailwindcss", name: "Tailwind CSS", byDefault: true, on: false, installed: false },
      { id: "humao.rest-client", name: "REST Client", byDefault: false, on: true, installed: false },
    ],
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
    extensions: [
      { id: "ms-python.python", name: "Python", byDefault: true, on: true, installed: false },
      { id: "charliermarsh.ruff", name: "Ruff", byDefault: true, on: true, installed: false },
    ],
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

/** Installs the kit components the project needs and are missing; waits for the end. */
export function projectInstallMissing(project: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_install_missing", { project });
}

/** Sets the editor extensions added to or removed from those of the project's languages. */
export function projectExtensions(project: string, added: string[], removed: string[]): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_extensions", { project, added, removed });
}

const SAMPLE_MATRIX: ExtensionsMatrix = {
  languages: ["JavaScript", "TypeScript", "HTML", "CSS", "Python"],
  tools: [["deno", "Deno"]],
  extensions: [
    {
      id: "ms-python.python",
      name: "Python",
      languages: ["Python"],
      tools: [],
      programs: [],
      optional: false,
      installed: false,
    },
    {
      id: "esbenp.prettier-vscode",
      name: "Prettier",
      languages: ["JavaScript", "TypeScript", "HTML", "CSS"],
      tools: [],
      programs: [],
      optional: false,
      installed: true,
    },
    {
      id: "denoland.vscode-deno",
      name: "Deno",
      languages: [],
      tools: ["deno"],
      programs: [],
      optional: false,
      installed: false,
    },
    {
      id: "usernamehw.errorlens",
      name: "Error Lens",
      languages: [],
      tools: [],
      programs: [],
      optional: false,
      installed: true,
    },
    {
      id: "mhutchie.git-graph",
      name: "Git Graph",
      languages: [],
      tools: [],
      programs: ["git"],
      optional: false,
      installed: true,
    },
    {
      id: "eamodio.gitlens",
      name: "GitLens",
      languages: [],
      tools: [],
      programs: [],
      optional: true,
      installed: false,
    },
  ],
};

/** The kit's matrix of editor extensions by language. */
export function extensionsMatrix(): Promise<ExtensionsMatrix> {
  return browserPreview ? Promise.resolve(SAMPLE_MATRIX) : invoke<ExtensionsMatrix>("extensions_matrix");
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
  | { kind: "failed"; message: string; byHand: ByHand | null };

/** A download that failed: its address, to download it in the browser and
 * drop the file on the app (src-tauri/src/downloads.rs). */
export interface ByHand {
  url: string;
  fileName: string;
}

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
  /** The steps not started yet, in order (empty when the run ends). */
  queue: { id: string; name: string }[];
  error: string | null;
}

/** `app`: part of Dev-Kit (the editor), shown with the required ones. */
export type Group = "required" | "app" | "agents" | "languages" | "os" | "databases";

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
  /** x64 only, on an ARM64 PC with Windows 10 (no x64 emulation there). */
  needsWindows11: boolean;
  /** One folder per minor version, side by side; the others are always at
   * their latest version. */
  versioned: boolean;
}

const IDLE: SetupSnapshot = {
  running: false,
  cancelling: false,
  automatic: false,
  total: 0,
  steps: [],
  queue: [],
  error: null,
};

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

function sampleComponent(
  id: string,
  name: string,
  group: Group,
  desc: string,
  mb: number,
  extra: Partial<ComponentInfo> = {},
): ComponentInfo {
  return {
    id,
    name,
    group,
    desc,
    mb,
    chosen: false,
    installed: false,
    present: false,
    locked: false,
    version: null,
    available: true,
    x64Only: false,
    noArm64: false,
    needs: [],
    needsWindows11: false,
    versioned: ["python", "node", "php", "go", "java"].includes(id),
    ...extra,
  };
}

const ON = { chosen: true, installed: true, present: true };

const SAMPLE_COMPONENTS: ComponentInfo[] = [
  sampleComponent("vscodium-server", "Editor", "app", "VSCodium nel browser", 180, {
    ...ON,
    locked: true,
    version: "1.140.0",
  }),
  sampleComponent("git", "Git", "required", "controllo di versione", 350, { ...ON, locked: true, version: "2.56.0" }),
  sampleComponent("gh", "GitHub CLI", "required", "GitHub dal terminale", 40, {
    ...ON,
    locked: true,
    version: "2.83.0",
    needs: ["git"],
  }),
  sampleComponent("claude", "Claude Code", "agents", "agente di Anthropic", 220, {
    ...ON,
    version: "2.1.299",
    needs: ["node"],
  }),
  sampleComponent("codex", "Codex", "agents", "agente di OpenAI", 120, { needs: ["node"] }),
  sampleComponent("node", "Node.js", "languages", "JavaScript e TypeScript", 90, { ...ON, version: "24.3.0" }),
  sampleComponent("python", "Python", "languages", "con uv", 120),
  sampleComponent("php", "PHP", "languages", "linguaggio per il web", 60, { ...ON, version: "8.4.12" }),
  sampleComponent("apache", "Apache", "languages", "server web per PHP", 30, { needs: ["php"] }),
  sampleComponent("uv", "uv", "languages", "ambienti e pacchetti Python", 40, { ...ON, version: "0.9.5" }),
  sampleComponent("typst", "Typst", "languages", "documenti e PDF da testo", 50),
  sampleComponent("mariadb", "MariaDB", "databases", "compatibile con MySQL", 210, { ...ON, version: "11.8.3" }),
  sampleComponent("mongodb", "MongoDB", "databases", "database a documenti", 650, { x64Only: true }),
  sampleComponent("terminal", "Windows Terminal", "os", "il terminale del kit", 25, { ...ON, version: "1.23" }),
];

/** When the Tools page opens: what was deleted by hand from the kit's
 * folder is no longer chosen; the names of those. */
export function componentsSync(): Promise<string[]> {
  return browserPreview ? Promise.resolve([]) : invoke<string[]>("components_sync");
}

export function componentsList(): Promise<ComponentInfo[]> {
  return browserPreview ? Promise.resolve(SAMPLE_COMPONENTS) : invoke<ComponentInfo[]>("components_list");
}

export function setupStart(only: string[] | null, updateAll: boolean): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("setup_start", { only, updateAll });
}

/** Files downloaded by hand, kept for the next installation: their names. */
export function downloadsAdd(paths: string[]): Promise<string[]> {
  return browserPreview ? Promise.resolve(paths) : invoke<string[]>("downloads_add", { paths });
}

/** The same, picked in Windows' "Open" window; empty when closed. */
export function downloadsPick(): Promise<string[]> {
  return browserPreview ? Promise.resolve([]) : invoke<string[]>("downloads_pick");
}

/** Opens in the browser the address of the failed download of a step. */
export function setupByHandOpen(id: string, name: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("setup_by_hand_open", { id, name });
}

/** Files dragged over the window: `over` with the position in CSS pixels,
 * `drop` with their paths, `leave`. */
export type FileDrag =
  { kind: "over"; x: number; y: number } | { kind: "drop"; x: number; y: number; paths: string[] } | { kind: "leave" };

export function onFileDrag(handler: (d: FileDrag) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return getCurrentWebview().onDragDropEvent((e) => {
    const p = e.payload;
    if (p.type === "leave") {
      handler({ kind: "leave" });
      return;
    }
    const scale = window.devicePixelRatio || 1;
    const x = p.position.x / scale;
    const y = p.position.y / scale;
    handler(p.type === "drop" ? { kind: "drop", x, y, paths: p.paths } : { kind: "over", x, y });
  });
}

export function setupCancel(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("setup_cancel");
}

/** Saves the choice (ids to keep installed) and installs/removes in the background. */
export function componentsApply(chosen: string[], updateAll: boolean): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("components_apply", { chosen, updateAll });
}

/** A minor line (`22.11`) of a language, with its full version when known. */
export interface VersionLine {
  line: string;
  version: string | null;
}

/** A language or tool whose versions can be kept side by side (versions.rs). */
export interface VersionedTool {
  id: string;
  name: string;
  installed: boolean;
  /** The default folder, replaced by updates. */
  default: VersionLine | null;
  /** The other lines kept, newest first. */
  saved: VersionLine[];
  /** The kit's preferred line; null = the default folder. */
  preferred: string | null;
  /** Other lines can be downloaded. */
  downloadable: boolean;
  /** The lines its source offered the last time it was asked, newest first
   * (also those on disk). */
  available: string[];
  /** When the source was asked (Unix seconds); null = never. */
  checked: number | null;
  /** The projects that chose each line. */
  projects: Record<string, string[]>;
}

export interface ProjectVersions {
  tools: VersionedTool[];
  /** Lines chosen for the project, by component id; the others follow the kit. */
  chosen: Record<string, string>;
}

const SAMPLE_VERSIONS: VersionedTool[] = [
  {
    id: "node",
    name: "Node.js",
    installed: true,
    default: { line: "24.11", version: "v24.11.0" },
    saved: [
      { line: "22.21", version: "v22.21.1" },
      { line: "20.19", version: "v20.19.5" },
    ],
    preferred: null,
    downloadable: true,
    available: ["25.1", "24.11", "24.10", "22.21", "22.20", "20.19", "18.20"],
    checked: 1760500000,
    projects: { "20.19": ["shop"] },
  },
  {
    id: "php",
    name: "PHP",
    installed: true,
    default: { line: "8.4", version: "8.4.12 (Composer 2.8.12)" },
    saved: [{ line: "8.3", version: "8.3.26 (Composer 2.8.12)" }],
    preferred: null,
    downloadable: true,
    available: [],
    checked: null,
    projects: {},
  },
  {
    id: "python",
    name: "Python",
    installed: false,
    default: null,
    saved: [],
    preferred: null,
    downloadable: true,
    available: [],
    checked: null,
    projects: {},
  },
];

export function versionsList(): Promise<VersionedTool[]> {
  return browserPreview ? Promise.resolve(SAMPLE_VERSIONS) : invoke<VersionedTool[]>("versions_list");
}

/** The lines of `id` that can be downloaded, newest first (asks its source;
 * versionsList keeps them in `available`). */
export function versionsAvailable(id: string): Promise<string[]> {
  return browserPreview
    ? Promise.resolve(id === "php" ? ["8.5", "8.4", "8.3", "8.2", "8.1"] : ["24.11", "22.21", "20.19", "18.20"])
    : invoke<string[]>("versions_available", { id });
}

/** Downloads `line` next to the default folder; progress as `setup` events. */
export function versionsAdd(id: string, line: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("versions_add", { id, line });
}

export function versionsRemove(id: string, line: string): Promise<VersionedTool[]> {
  return browserPreview ? Promise.resolve(SAMPLE_VERSIONS) : invoke<VersionedTool[]>("versions_remove", { id, line });
}

export function projectVersions(project: string): Promise<ProjectVersions> {
  return browserPreview
    ? Promise.resolve({ tools: SAMPLE_VERSIONS, chosen: { node: "20.19" } })
    : invoke<ProjectVersions>("project_versions", { project });
}

/** The line of `id` for the project; null = the kit's. One not on disk is
 * downloaded (progress as `setup` events). */
/** A relation of the matrix (relations.rs, D-81): `from` needs or uses
 * `to`, at the versions `range`; parts are `component/part`. */
export interface RelationLink {
  from: string;
  fromName: string;
  to: string;
  toName: string;
  kind: "needs" | "uses" | "runs-on" | "loads";
  optional: boolean;
  range: string | null;
  note: string | null;
  status: "inactive" | "ok" | "missing" | "optional" | "incompatible";
  /** The version of `to` the kit uses. */
  uses: string | null;
  /** Lines of `to` on disk outside `range`. */
  outside: string[];
}

/** What the PC must provide for some components. */
export interface RelationRuntime {
  id: string;
  name: string;
  note: string | null;
  neededBy: string[];
  present: boolean;
  detail: string | null;
}

/** A version a project declares, and what the kit can do for it. */
export interface RelationNeed {
  component: string;
  name: string;
  /** `package.json engines.node`. */
  source: string;
  declared: string;
  status: "ok" | "choose" | "download" | "unavailable" | "notInstalled" | "unknown";
  uses: string | null;
  /** For choose and download: the line that fits. */
  line: string | null;
  /** A database server (one for every project): choose sets the version
   * the server runs. */
  server: boolean;
}

export interface RelationsMatrix {
  fingerprint: string;
  generated: number;
  platform: string;
  links: RelationLink[];
  runtimes: RelationRuntime[];
  projects: { project: string; needs: RelationNeed[] }[];
}

const SAMPLE_RELATIONS: RelationsMatrix = {
  fingerprint: "sample",
  generated: 1760500000,
  platform: "win32-x64",
  links: [
    {
      from: "php/composer",
      fromName: "Composer",
      to: "php",
      toName: "PHP",
      kind: "runs-on",
      optional: false,
      range: ">=7.2.5",
      note: "Composer 2.3 e successivi; per PHP più vecchi il kit sceglie il 2.2 LTS",
      status: "ok",
      uses: "8.4.12 (Composer 2.8.12)",
      outside: [],
    },
    {
      from: "pnpm",
      fromName: "pnpm, yarn",
      to: "node",
      toName: "Node.js",
      kind: "runs-on",
      optional: false,
      range: ">=18.12",
      note: "pnpm e yarn usano il Node.js del kit",
      status: "inactive",
      uses: "v24.11.0",
      outside: [],
    },
    {
      from: "claude",
      fromName: "Claude Code",
      to: "git",
      toName: "Git",
      kind: "uses",
      optional: true,
      range: null,
      note: "Git Bash per lo strumento Bash; senza, Claude usa PowerShell",
      status: "ok",
      uses: "2.56.0",
      outside: [],
    },
  ],
  runtimes: [
    {
      id: "vcruntime",
      name: "Visual C++ 2015-2026 x64",
      note: "DLL copiate dal kit accanto ai programmi",
      neededBy: ["PHP"],
      present: true,
      detail: null,
    },
    {
      id: "webview2",
      name: "WebView2",
      note: "la finestra di NEXKORA",
      neededBy: ["NEXKORA"],
      present: true,
      detail: "141.0.3537.71",
    },
  ],
  projects: [
    {
      project: "shop",
      needs: [
        {
          component: "node",
          name: "Node.js",
          source: "package.json engines.node",
          declared: ">=22",
          status: "choose",
          uses: "20.19",
          line: "22.21",
          server: false,
        },
        {
          component: "php",
          name: "PHP",
          source: "composer.json require.php",
          declared: "^8.2",
          status: "ok",
          uses: "8.4",
          line: null,
          server: false,
        },
      ],
    },
  ],
};

/** The relations matrix: the saved one while nothing it is made from
 * changed; `force` builds it again. */
export function relationsMatrix(force = false): Promise<RelationsMatrix> {
  return browserPreview ? Promise.resolve(SAMPLE_RELATIONS) : invoke<RelationsMatrix>("relations_matrix", { force });
}

export function projectVersionsSet(project: string, id: string, line: string | null): Promise<ProjectVersions> {
  return browserPreview
    ? Promise.resolve({ tools: SAMPLE_VERSIONS, chosen: line ? { [id]: line } : {} })
    : invoke<ProjectVersions>("project_versions_set", { project, id, line });
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

const SAMPLE_AGENTS: AgentInfo[] = [
  {
    id: "claude",
    name: "Claude Code",
    installed: true,
    version: "2.1.299",
    signedIn: true,
    signInKind: "account",
    running: true,
  },
  { id: "codex", name: "Codex", installed: true, version: "0.98.0", signedIn: false, signInKind: null, running: false },
  {
    id: "antigravity",
    name: "Antigravity",
    installed: false,
    version: null,
    signedIn: false,
    signInKind: null,
    running: false,
  },
];

export function agentsStatus(): Promise<AgentInfo[]> {
  return browserPreview ? Promise.resolve(SAMPLE_AGENTS) : invoke<AgentInfo[]>("agents_status");
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

/** Opens a kit folder in File Explorer: "agents", "db" or "projects" (+ project). */
export function folderOpen(
  which: "agents" | "backup" | "db" | "projects",
  project: string | null = null,
): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("folder_open", { which, project });
}

// Database servers of the kit (src-tauri/src/services.rs, D-73).

export type ServiceState = "stopped" | "starting" | "running" | "error";

export interface ServiceInfo {
  id: "mariadb" | "mongodb";
  name: string;
  installed: boolean;
  state: ServiceState;
  port: number;
  autostart: boolean;
  dataDir: string;
  log: string;
  url: string;
  user: string;
  password: string | null;
  message: string | null;
  /** The version it runs, and those installed, newest first (D-81). */
  line: string | null;
  lines: string[];
}

const SAMPLE_SERVICES: ServiceInfo[] = [
  {
    id: "mariadb",
    name: "MariaDB",
    installed: true,
    state: "running",
    port: 3306,
    autostart: true,
    dataDir: "E:\\Dev-Kit\\data\\db\\mariadb",
    log: "E:\\Dev-Kit\\data\\app\\logs\\mariadb.log",
    url: "mysql://root@127.0.0.1:3306/",
    user: "root",
    password: "3f9c2a71d8e04b6a9c51e2d7f0a8b346",
    message: null,
    line: "12.3",
    lines: ["12.3", "11.4"],
  },
  {
    id: "mongodb",
    name: "MongoDB",
    installed: false,
    state: "stopped",
    port: 27017,
    autostart: false,
    dataDir: "E:\\Dev-Kit\\data\\db\\mongodb",
    log: "E:\\Dev-Kit\\data\\app\\logs\\mongodb.log",
    url: "mongodb://admin@127.0.0.1:27017/?authSource=admin",
    user: "admin",
    password: null,
    message: null,
    line: null,
    lines: [],
  },
];

export function servicesList(): Promise<ServiceInfo[]> {
  return browserPreview ? Promise.resolve(SAMPLE_SERVICES) : invoke<ServiceInfo[]>("services_list");
}

/** Starts a server (installing it first if needed) and waits until it answers. */
export function serviceStart(id: ServiceInfo["id"]): Promise<ServiceInfo[]> {
  return browserPreview ? Promise.resolve(SAMPLE_SERVICES) : invoke<ServiceInfo[]>("service_start", { id });
}

export function serviceStop(id: ServiceInfo["id"]): Promise<ServiceInfo[]> {
  return browserPreview ? Promise.resolve(SAMPLE_SERVICES) : invoke<ServiceInfo[]>("service_stop", { id });
}

/** Port (null: the default) and "start with Dev-Kit"; the port applies at the next start. */
export function serviceSet(id: ServiceInfo["id"], port: number | null, autostart: boolean): Promise<ServiceInfo[]> {
  return browserPreview
    ? Promise.resolve(SAMPLE_SERVICES)
    : invoke<ServiceInfo[]>("service_set", { id, port, autostart });
}

/** The version a server runs (null: the newest installed), while stopped;
 * each version keeps its own data, the first start copies the older one's. */
export function serviceLine(id: ServiceInfo["id"], line: string | null): Promise<ServiceInfo[]> {
  return browserPreview ? Promise.resolve(SAMPLE_SERVICES) : invoke<ServiceInfo[]>("service_line", { id, line });
}

// What is inside a running server, read only (src-tauri/src/db_browse.rs, D-79).

export interface DbTable {
  name: string;
  /** Estimated; null for views or when the server does not say. */
  rows: number | null;
  view: boolean;
}

export interface DbDatabase {
  name: string;
  /** The server's own (mysql, information_schema, admin, local…). */
  system: boolean;
  tables: DbTable[];
}

export interface DbRows {
  /** kind: MariaDB's column type, empty for MongoDB. */
  columns: { name: string; kind: string }[];
  /** JSON values; binary data as {"$bytes": n}, MongoDB's as Extended JSON. */
  rows: unknown[][];
  total: number;
  offset: number;
}

/** Rows of one page (db_browse::PAGE). */
export const DB_PAGE = 100;

const SAMPLE_TREE: DbDatabase[] = [
  {
    name: "api_python",
    system: false,
    tables: [
      { name: "orders", rows: 1240, view: false },
      { name: "products", rows: 87, view: false },
      { name: "users", rows: 3, view: false },
      { name: "top_products", rows: null, view: true },
    ],
  },
  { name: "sito_vetrina", system: false, tables: [] },
  { name: "information_schema", system: true, tables: [{ name: "TABLES", rows: null, view: true }] },
  { name: "mysql", system: true, tables: [{ name: "user", rows: 5, view: false }] },
];

function sampleRows(table: string, offset: number): DbRows {
  const users = [
    [1, "ada", "ada@example.com", "2026-10-01 09:12:00", null, { $bytes: 2048 }],
    [2, "linus", "linus@example.com", "2026-10-03 18:40:12", "admin", null],
    [3, "grace", "grace@example.com", "2026-10-14 07:05:55", "editor", { $bytes: 512 }],
  ];
  if (table === "users")
    return {
      columns: [
        { name: "id", kind: "int(11)" },
        { name: "name", kind: "varchar(64)" },
        { name: "email", kind: "varchar(255)" },
        { name: "created_at", kind: "datetime" },
        { name: "role", kind: "varchar(16)" },
        { name: "avatar", kind: "blob" },
      ],
      rows: users,
      total: 3,
      offset: 0,
    };
  const total = table === "orders" ? 1240 : 87;
  const rows = Array.from({ length: Math.max(0, Math.min(DB_PAGE, total - offset)) }, (_, i) => [
    offset + i + 1,
    1 + ((offset + i) % 3),
    Math.round((offset + i) * 7.3 * 100) / 100,
    "2026-10-14",
  ]);
  return {
    columns: [
      { name: "id", kind: "int(11)" },
      { name: "user_id", kind: "int(11)" },
      { name: "total", kind: "decimal(10,2)" },
      { name: "day", kind: "date" },
    ],
    rows,
    total,
    offset,
  };
}

export function dbBrowseTree(engine: ServiceInfo["id"]): Promise<DbDatabase[]> {
  return browserPreview ? Promise.resolve(SAMPLE_TREE) : invoke<DbDatabase[]>("db_browse_tree", { engine });
}

export function dbBrowseRows(
  engine: ServiceInfo["id"],
  database: string,
  table: string,
  offset: number,
): Promise<DbRows> {
  return browserPreview
    ? Promise.resolve(sampleRows(table, offset))
    : invoke<DbRows>("db_browse_rows", { engine, database, table, offset });
}

// Git state and agent instructions of a project (src-tauri/src/projects.rs).

export interface GitInfo {
  branch: string | null;
  upstream: string | null;
  /** Same commit as its remote branch (last fetch); null without one. */
  synced: boolean | null;
  remote: string | null;
}

export interface ProjectStatus {
  git: GitInfo | null;
  instructions: string[];
}

export function projectStatus(project: string): Promise<ProjectStatus> {
  return browserPreview
    ? Promise.resolve({
        git: {
          branch: "main",
          upstream: "origin/main",
          synced: false,
          remote: "https://github.com/nabre/negozio-online.git",
        },
        instructions: ["AGENTS.md"],
      })
    : invoke<ProjectStatus>("project_status", { project });
}

// Databases of a project (src-tauri/src/db_auth.rs, D-74).

export interface ProjectDb {
  engine: "mariadb" | "mongodb";
  database: string;
  user: string;
  password: string;
  host: string;
  port: number;
  url: string;
}

export interface ProjectDatabases {
  dbs: ProjectDb[];
  /** "mariadb", "mongodb", "sqlite" the project seems to use. */
  detected: string[];
  envExists: boolean;
  envIgnored: boolean;
}

export interface EnvChange {
  key: string;
  old: string | null;
  new: string;
}

const SAMPLE_PROJECT_DATABASES: ProjectDatabases = {
  dbs: [
    {
      engine: "mariadb",
      database: "negozio_online",
      user: "negozio_online",
      password: "5be1c0d29a7f4e8b93c6d2a1f0e47b58",
      host: "127.0.0.1",
      port: 3306,
      url: "mysql://negozio_online:5be1c0d29a7f4e8b93c6d2a1f0e47b58@127.0.0.1:3306/negozio_online",
    },
  ],
  detected: ["mariadb", "sqlite"],
  envExists: true,
  envIgnored: true,
};

export function projectDatabases(project: string): Promise<ProjectDatabases> {
  return browserPreview
    ? Promise.resolve(SAMPLE_PROJECT_DATABASES)
    : invoke<ProjectDatabases>("project_databases", { project });
}

/** Creates the project's database and user, installing and starting the server if needed. */
export function projectDbCreate(project: string, engine: ProjectDb["engine"]): Promise<ProjectDb> {
  return browserPreview
    ? Promise.resolve(SAMPLE_PROJECT_DATABASES.dbs[0] as ProjectDb)
    : invoke<ProjectDb>("project_db_create", { project, engine });
}

export function projectEnvPreview(project: string, engine: ProjectDb["engine"]): Promise<EnvChange[]> {
  return browserPreview
    ? Promise.resolve([{ key: "DB_DATABASE", old: "laravel", new: "negozio_online" }])
    : invoke<EnvChange[]>("project_env_preview", { project, engine });
}

export function projectEnvApply(project: string, engine: ProjectDb["engine"]): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_env_apply", { project, engine });
}

// Projects (src-tauri/src/projects.rs) and their actions (run.rs).

/** A new empty project; returns its folder name. */
export function projectCreate(name: string): Promise<string> {
  return browserPreview ? Promise.resolve(name) : invoke<string>("project_create", { name });
}

// --- Frameworks of the new-project wizard (frameworks.rs, D-83) ---

export interface FrameworkAction {
  name: string;
  command: string;
}

/** A line of the project's .env; {name} and {package} are filled in. */
export interface EnvVar {
  key: string;
  value: string;
}

/** What an addition does with one base. */
export interface FrameworkVariant {
  needs?: string[];
  /** Replaces the base's creation (a starter kit, another template). */
  create?: string[];
  steps?: string[];
  actions?: FrameworkAction[];
  /** A database server of the kit (mariadb, mongodb): the project gets its database there. */
  database?: string;
  env?: EnvVar[];
  note?: string;
}

export interface FrameworkPiece {
  id: string;
  kind: "base" | "addon";
  name: string;
  /** Italian: shown through FrameworksView.texts. */
  desc: string;
  home: string;
  /** Additions of the same slot do not go together. */
  slot?: string;
  needs?: string[];
  /** Run in projects\: makes the folder {name}. */
  create?: string[];
  /** Run inside the project. */
  steps?: string[];
  actions?: FrameworkAction[];
  env?: EnvVar[];
  note?: string;
  /** Of an addition: what it does with each base, by base id. */
  on?: Record<string, FrameworkVariant>;
}

export interface FrameworksView {
  pieces: FrameworkPiece[];
  official: FrameworkPiece[];
  /** Descriptions and notes in the app's language, by Italian source. */
  texts: Record<string, string>;
  problem: string | null;
}

export interface FrameworkPlan {
  needs: string[];
  create: string[];
  steps: string[];
  actions: FrameworkAction[];
  /** The kit's database server where the project gets its database. */
  database: string | null;
  /** Variables of .env, {name} and {package} not filled in yet. */
  env: EnvVar[];
  /** Italian sources (FrameworksView.texts). */
  notes: string[];
}

/** A folder of projects\ for the wizard's name: an empty one or one with
 * files but no Git can be used; a Git repository cannot. */
export interface FrameworkTarget {
  name: string;
  state: "empty" | "files" | "git";
}

/** Progress of a creation: a command starting, or a line of its output. */
export interface FrameworkEvent {
  /** 1-based number of the command; 0 = installing the kit's tools. */
  step: number;
  total: number;
  command: string | null;
  line: string | null;
}

const SAMPLE_PIECES: FrameworkPiece[] = [
  {
    id: "laravel",
    kind: "base",
    name: "Laravel",
    desc: "Framework PHP per applicazioni web",
    home: "https://laravel.com/docs/13.x/installation",
    needs: ["php", "node"],
    create: ["composer create-project laravel/laravel:^13.0 {name} --prefer-dist"],
    steps: ["npm install"],
    actions: [{ name: "Avvia", command: "composer run dev" }],
  },
  {
    id: "vite",
    kind: "base",
    name: "Vite",
    desc: "Sito o applicazione web in TypeScript",
    home: "https://vite.dev/guide/",
    needs: ["node"],
    create: ["npm create vite@9 {name} -- --template vanilla-ts --no-interactive"],
    steps: ["npm install"],
    actions: [{ name: "Avvia", command: "npm run dev" }],
  },
  {
    id: "react",
    kind: "addon",
    slot: "frontend",
    name: "React",
    desc: "Interfaccia in React con TypeScript",
    home: "https://react.dev/learn/creating-a-react-app",
    on: {
      laravel: { create: ["composer create-project laravel/react-starter-kit {name} --prefer-dist --stability=dev"] },
      vite: { create: ["npm create vite@9 {name} -- --template react-ts --no-interactive"] },
    },
  },
  {
    id: "tailwind",
    kind: "addon",
    slot: "css",
    name: "Tailwind CSS",
    desc: "Stili con le classi di utilità di Tailwind CSS 4",
    home: "https://tailwindcss.com/docs/installation/using-vite",
    on: {
      laravel: { note: "Già incluso in Laravel e nei suoi starter kit." },
      vite: { steps: ["npm install -D tailwindcss@4 @tailwindcss/vite@4"] },
    },
  },
];

export function frameworksGet(): Promise<FrameworksView> {
  return browserPreview
    ? Promise.resolve({ pieces: SAMPLE_PIECES, official: SAMPLE_PIECES, texts: {}, problem: null })
    : invoke<FrameworksView>("frameworks_get");
}

/** Saves the instructions; those equal to the official ones are not kept. */
export function frameworksSet(pieces: FrameworkPiece[]): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("frameworks_set", { pieces });
}

/** What a base with its additions would run, with `pieces` as edited. */
export function frameworksPlan(pieces: FrameworkPiece[], base: string, addons: string[]): Promise<FrameworkPlan> {
  if (browserPreview) {
    const b = pieces.find((p) => p.id === base);
    return Promise.resolve({
      needs: b?.needs ?? [],
      create: b?.create ?? [],
      steps: b?.steps ?? [],
      actions: b?.actions ?? [],
      database: null,
      env: b?.env ?? [],
      notes: [],
    });
  }
  return invoke<FrameworkPlan>("frameworks_plan", { pieces, base, addons });
}

/** Creates projects\<name> (or fills an existing one) with `env` in its .env
 * and returns its name; progress comes as `framework` events. */
export function frameworksCreate(base: string, addons: string[], name: string, env: EnvVar[]): Promise<string> {
  return browserPreview ? Promise.resolve(name) : invoke<string>("frameworks_create", { base, addons, name, env });
}

export function frameworksTargets(): Promise<FrameworkTarget[]> {
  return browserPreview
    ? Promise.resolve([
        { name: "vuoto", state: "empty" },
        { name: "appunti", state: "files" },
      ])
    : invoke<FrameworkTarget[]>("frameworks_targets");
}

export function frameworksCancel(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("frameworks_cancel");
}

export function onFramework(handler: (e: FrameworkEvent) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return listen<FrameworkEvent>("framework", (e) => {
    handler(e.payload);
  });
}

/** Deletes a project whose folder is empty. */
export function projectDelete(project: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_delete", { project });
}

/** Clones `owner/repo` or an https:// Git address; returns the folder name. */
export function projectClone(source: string, name: string | null): Promise<string> {
  return browserPreview ? Promise.resolve(name ?? "clone") : invoke<string>("project_clone", { source, name });
}

export type SiteKind = "static" | "php" | "apache" | "none";

/** A project's editor settings in .vscode\settings.json (D-78); null: not set. */
export interface EditorSettings {
  formatOnSave: boolean | null;
  tabSize: number | null;
  insertSpaces: boolean | null;
  hidden: string[];
}

export interface EditorSettingsView {
  settings: EditorSettings;
  exists: boolean;
  /** Comments or not plain JSON: shown, never written. */
  readOnly: boolean;
}

export function projectEditorSettings(project: string): Promise<EditorSettingsView> {
  return browserPreview
    ? Promise.resolve({
        settings: { formatOnSave: true, tabSize: 4, insertSpaces: true, hidden: ["**/vendor"] },
        exists: true,
        readOnly: false,
      })
    : invoke<EditorSettingsView>("project_editor_settings", { project });
}

export function projectEditorSettingsSet(project: string, settings: EditorSettings): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_editor_settings_set", { project, settings });
}

/** A copy of a project's database (data\db\backup\<project>, D-78). */
export interface Backup {
  file: string;
  engine: "mariadb" | "mongodb";
  size: number;
}

export function projectBackups(project: string): Promise<Backup[]> {
  return browserPreview
    ? Promise.resolve([{ file: "negozio_online-20261014-171500.sql", engine: "mariadb", size: 48211 }])
    : invoke<Backup[]>("project_backups", { project });
}

/** A copy of the project's database; returns the file name. */
export function projectDbExport(project: string, engine: Backup["engine"]): Promise<string> {
  return browserPreview ? Promise.resolve("copia.sql") : invoke<string>("project_db_export", { project, engine });
}

export function projectDbRestore(project: string, file: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_db_restore", { project, file });
}

/** A project's own MCP server (.mcp.json, D-78). */
export interface McpServer {
  name: string;
  kind: "stdio" | "http";
  command: string | null;
  args: string[];
  url: string | null;
  env: Record<string, string>;
}

export interface McpView {
  servers: McpServer[];
  /** .mcp.json is not plain JSON: never written. */
  readOnly: boolean;
}

export function projectMcp(project: string): Promise<McpView> {
  return browserPreview
    ? Promise.resolve({
        servers: [{ name: "playwright", kind: "stdio", command: "npx", args: ["@playwright/mcp"], url: null, env: {} }],
        readOnly: false,
      })
    : invoke<McpView>("project_mcp", { project });
}

export function projectMcpSet(project: string, servers: McpServer[]): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_mcp_set", { project, servers });
}

/** An action as the project page edits it (panel.json, D-78). */
export interface ActionEdit {
  id: string;
  label: string;
  command: string;
  kind: ActionKind;
  folder: string | null;
  url: string | null;
  env: Record<string, string>;
}

/** Adds or changes an action in the project's panel.json (same id: replaced). */
export function projectActionSave(project: string, action: ActionEdit): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_action_save", { project, action });
}

/** Removes an action written in panel.json; a detected one with that id comes back. */
export function projectActionRemove(project: string, id: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_action_remove", { project, id });
}

/** The action that "Avvia" starts. */
export function projectActionDefault(project: string, id: string): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("project_action_default", { project, id });
}

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

// The kit's GitHub account (src-tauri/src/github.rs): one sign-in for this
// page, the kit's Git (editor, consoles, agents) and GitHub CLI.

export interface GithubStatus {
  /** GitHub CLI is in the kit. */
  installed: boolean;
  signedIn: boolean;
  login: string | null;
  /** A sign-in is waiting for the user on github.com. */
  signingIn: boolean;
  /** The one-time code to type at github.com/login/device. */
  code: string | null;
  /** Why the last sign-in failed. */
  error: string | null;
}

export interface GithubRepo {
  fullName: string;
  name: string;
  owner: string;
  private: boolean;
  fork: boolean;
  archived: boolean;
  description: string;
  language: string;
  stars: number;
  pushedAt: string | null;
  htmlUrl: string;
  /** The account administers it: rename and delete. */
  admin: boolean;
  /** The project in projects\ that is its clone. */
  local: string | null;
}

export interface GithubRepos {
  login: string;
  /** The account may delete repositories (null: the token does not say). */
  canDelete: boolean | null;
  repos: GithubRepo[];
}

const SAMPLE_GITHUB: GithubStatus = {
  installed: true,
  signedIn: true,
  login: "octocat",
  signingIn: false,
  code: null,
  error: null,
};

function sampleRepo(owner: string, name: string, extra: Partial<GithubRepo>): GithubRepo {
  return {
    fullName: `${owner}/${name}`,
    name,
    owner,
    private: false,
    fork: false,
    archived: false,
    description: "",
    language: "",
    stars: 0,
    pushedAt: "2026-10-01T10:00:00Z",
    htmlUrl: `https://github.com/${owner}/${name}`,
    admin: owner === "octocat",
    local: null,
    ...extra,
  };
}

const SAMPLE_REPOS: GithubRepos = {
  login: "octocat",
  canDelete: false,
  repos: [
    sampleRepo("octocat", "sito-vetrina", {
      language: "HTML",
      local: "sito-vetrina",
      description: "Il sito del negozio",
    }),
    sampleRepo("octocat", "api-python", { language: "Python", private: true, stars: 3 }),
    sampleRepo("octocat", "dotfiles", { fork: true }),
    sampleRepo("acme", "gestionale", { language: "TypeScript", private: true, description: "Gestionale interno" }),
  ],
};

export function githubStatus(): Promise<GithubStatus> {
  return browserPreview ? Promise.resolve(SAMPLE_GITHUB) : invoke<GithubStatus>("github_status");
}

/** Starts the sign-in; answers with the one-time code. `deleteScope`: also the
 * permission to delete repositories. */
export function githubSignIn(deleteScope: boolean): Promise<GithubStatus> {
  return browserPreview
    ? Promise.resolve({ ...SAMPLE_GITHUB, signedIn: false, signingIn: true, code: "1A2B-3C4D" })
    : invoke<GithubStatus>("github_sign_in", { delete: deleteScope });
}

export function githubSignInCancel(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("github_sign_in_cancel");
}

export function githubSignOut(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("github_sign_out");
}

export function githubRepos(): Promise<GithubRepos> {
  return browserPreview ? Promise.resolve(SAMPLE_REPOS) : invoke<GithubRepos>("github_repos");
}

/** Clones a repository of the account into projects\; returns the project. */
export function githubClone(fullName: string): Promise<string> {
  return browserPreview ? Promise.resolve(fullName.split("/")[1] ?? "") : invoke<string>("github_clone", { fullName });
}

export function githubRename(fullName: string, name: string): Promise<{ repo: GithubRepo; local: string | null }> {
  return invoke<{ repo: GithubRepo; local: string | null }>("github_rename", { fullName, name });
}

export function githubDelete(fullName: string): Promise<"deleted" | "needsPermission"> {
  return browserPreview
    ? Promise.resolve("needsPermission")
    : invoke<"deleted" | "needsPermission">("github_delete", { fullName });
}

/** Opens a repository on github.com (`null`: the page for the sign-in code). */
export function githubOpen(fullName: string | null): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("github_open", { fullName });
}

/** The sign-in got its code or ended. */
export function onGithub(handler: (s: GithubStatus) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return listen<GithubStatus>("github", (e) => {
    handler(e.payload);
  });
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
  databases: number;
}

const NOTHING_OPEN: OpenNow = { processes: [], editor: false, terminals: 0, actions: 0, setup: false, databases: 0 };

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
