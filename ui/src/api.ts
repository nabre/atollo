// Typed wrappers of the Rust commands (src-tauri/src/commands.rs). The
// interface calls nothing else: no generic "run this" command exists.
//
// Outside Tauri (`npm run dev` in ui\ opened in a browser) the commands do
// not exist: the wrappers answer with sample data, so the pages can be
// designed in the browser, and the interface shows a "preview" banner.
import { invoke, isTauri } from "@tauri-apps/api/core";

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

export interface ProjectEntry {
  name: string;
  /** Seconds since 1970. */
  modified: number;
}

const SAMPLE_PROJECTS: ProjectEntry[] = [
  { name: "sito-vetrina", modified: 1_791_900_000 },
  { name: "api-python", modified: 1_791_800_000 },
];

export function editorStatus(): Promise<EditorStatus> {
  return browserPreview
    ? Promise.resolve({ installed: true, running: false, port: null, emulated: false })
    : invoke<EditorStatus>("editor_status");
}

/** Opens the editor on a project (`null` = the projects folder). */
export function editorOpen(project: string | null): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("editor_open", { project });
}

export function editorRestart(): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>("editor_restart");
}

export function projectsList(): Promise<ProjectEntry[]> {
  return browserPreview ? Promise.resolve(SAMPLE_PROJECTS) : invoke<ProjectEntry[]>("projects_list");
}

/** Opens a kit console (Windows Terminal) in a project or in `projects\`. */
export function terminalOpen(project: string | null): Promise<void> {
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

export type Group = "required" | "agents" | "languages" | "optional";

export interface ComponentInfo {
  id: string;
  name: string;
  group: Group;
  desc: string;
  mb: number;
  chosen: boolean;
  installed: boolean;
  version: string | null;
  available: boolean;
  x64Only: boolean;
  needs: string[];
}

const IDLE: SetupSnapshot = { running: false, cancelling: false, automatic: false, total: 0, steps: [], error: null };

export function setupState(): Promise<SetupSnapshot> {
  return browserPreview ? Promise.resolve(IDLE) : invoke<SetupSnapshot>("setup_state");
}

/** Every change of the preparation; returns the function that stops listening. */
export async function onSetup(handler: (s: SetupSnapshot) => void): Promise<() => void> {
  if (browserPreview) return () => undefined;
  const { listen } = await import("@tauri-apps/api/event");
  return listen<SetupSnapshot>("setup", (e) => {
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
