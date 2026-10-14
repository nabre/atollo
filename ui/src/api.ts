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
