// Projects (tauri/src/projects/): list, creation, clone, status, actions
// (panel.json), site and MCP servers. Rust: tauri/src/commands/projects.rs.

import { act, query } from "./core";

export type ActionKind = "web" | "desktop" | "command";

/** An action of a project (tauri/src/detect). */
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
  /** A folder served by Apache (a shared pool, or the site's own server). */
  apache: string | null;
  /** A folder served by Caddy (a shared pool, or the site's own server). */
  caddy?: string | null;
  /** Web server profile id, "auto" (the automatic pool of the project's
   * versions) or "dedicated" (the site's own server). */
  server?: string | null;
  /** RFC 6761 .localhost domain aliases. */
  domains?: string[];
  /** Portable relative path aliases (/url -> folder). */
  aliases?: Record<string, string>;
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
  /** Lines of some of them named by the images of Compose and Dockerfile
   * (`php:8.3-apache` → `{ php: "8.3" }`); absent when none. */
  lines?: Record<string, string>;
  actions: ProjectAction[];
  warnings: string[];
  custom: boolean;
  default: string | null;
  /** The kit's settings of the project (data\projects\<id>\dev-settings.json). */
  settings: {
    agent?: string;
    lastOpened?: number;
    /** Among the favorites of the Projects home. */
    pinned?: boolean;
    extensions?: { added: string[]; removed: string[] };
  };
  /** Editor extensions of its languages, then those added by hand. */
  extensions: ProjectExtension[];
}

/** An editor extension of a project (tauri/src/editor/ext.rs). */
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
    caddy: null,
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

export function projectsList(): Promise<ProjectInfo[]> {
  return query<ProjectInfo[]>("projects_list", SAMPLE_PROJECTS);
}

/** Installs the kit components the project needs and are missing; waits for the end. */
export function projectInstallMissing(project: string): Promise<void> {
  return act("project_install_missing", { project });
}

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
  return query<ProjectStatus>(
    "project_status",
    {
      git: {
        branch: "main",
        upstream: "origin/main",
        synced: false,
        remote: "https://github.com/nabre/negozio-online.git",
      },
      instructions: ["AGENTS.md"],
    },
    { project },
  );
}

/** A new empty project; returns its folder name. */
export function projectCreate(name: string): Promise<string> {
  return query<string>("project_create", name, { name });
}

/** A database a project's deletion drops (tauri/src/projects/mod.rs). */
export interface PlannedDb {
  engine: "mysql" | "mongodb";
  /** Name of the database and of its user. */
  database: string;
  /** Not installed: skipped, the database stays in the server's data. */
  installed: boolean;
}

/** What deleting a project takes away (D-89). */
export interface DeletePlan {
  folderExists: boolean;
  /** Only an empty folder is deleted from the app. */
  folderEmpty: boolean;
  /** Its databases on the kit's servers, dropped first. */
  databases: PlannedDb[];
  /** The copies of its databases and their bytes. */
  backups: number;
  backupsSize: number;
}

export function projectDeletePlan(project: string): Promise<DeletePlan> {
  return query<DeletePlan>(
    "project_delete_plan",
    {
      folderExists: true,
      folderEmpty: true,
      databases: [{ engine: "mysql", database: project.toLowerCase(), installed: true }],
      backups: 2,
      backupsSize: 3_400_000,
    },
    { project },
  );
}

/** Deletes a project whose folder is empty, with its databases and users on
 * the kit's servers (started if stopped). A failed drop deletes nothing;
 * `forceWithoutDb` deletes without touching the servers. */
export function projectDelete(project: string, forceWithoutDb = false): Promise<void> {
  return act("project_delete", { project, forceWithoutDb });
}

/** Clones `owner/repo` or an https:// Git address; returns the folder name. */
export function projectClone(source: string, name: string | null): Promise<string> {
  return query<string>("project_clone", name ?? "clone", { source, name });
}

export type SiteKind = "static" | "php" | "apache" | "caddy" | "none";

export interface SiteOptions {
  server?: string | null;
  domains?: string[];
  aliases?: Record<string, string>;
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
  return query<McpView>(
    "project_mcp",
    {
      servers: [{ name: "playwright", kind: "stdio", command: "npx", args: ["@playwright/mcp"], url: null, env: {} }],
      readOnly: false,
    },
    { project },
  );
}

export function projectMcpSet(project: string, servers: McpServer[]): Promise<void> {
  return act("project_mcp_set", { project, servers });
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
  return act("project_action_save", { project, action });
}

/** Removes an action written in panel.json; a detected one with that id comes back. */
export function projectActionRemove(project: string, id: string): Promise<void> {
  return act("project_action_remove", { project, id });
}

/** The action that "Avvia" starts. */
export function projectActionDefault(project: string, id: string): Promise<void> {
  return act("project_action_default", { project, id });
}

/** The project among the favorites of the Projects home, or not. */
export function projectPin(project: string, pinned: boolean): Promise<void> {
  return act("project_pin", { project, pinned });
}

/** "Servi come sito web": writes the "site" action in the project's panel.json. */
export function projectSite(project: string, folder: string, kind: SiteKind, options?: SiteOptions): Promise<void> {
  return act("project_site", { project, folder, kind, options });
}
