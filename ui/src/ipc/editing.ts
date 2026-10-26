// The editor, the kit's console, a project's editor settings and extensions.
// Rust: tauri/src/ipc/editing.rs.

import { act, query, subscribe } from "./core";
import { EDITOR_STATUS } from "./events";

export interface EditorStatus {
  installed: boolean;
  running: boolean;
  port: number | null;
  /** The recommended extensions being installed in the background. */
  extensions: EditorExtensions;
}

/** Rust: `ExtState` (tauri/src/editor/ext.rs). */
export interface EditorExtensions {
  running: boolean;
  total: number;
  done: number;
  /** Ids of the extensions that did not install. */
  failed: string[];
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

export const SAMPLE_EDITOR: EditorStatus = {
  installed: true,
  running: false,
  port: null,
  extensions: { running: false, total: 0, done: 0, failed: [] },
};

export function editorStatus(): Promise<EditorStatus> {
  return query<EditorStatus>("editor_status", SAMPLE_EDITOR);
}

/** Opens the editor on a project (`null` = the projects folder). */
export function editorOpen(project: string): Promise<void> {
  return act("editor_open", { project });
}

export function editorRestart(): Promise<void> {
  return act("editor_restart");
}

/** Sets the editor extensions added to or removed from those of the project's languages. */
export function projectExtensions(project: string, added: string[], removed: string[]): Promise<void> {
  return act("project_extensions", { project, added, removed });
}

export const SAMPLE_MATRIX: ExtensionsMatrix = {
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

/** The kit's matrix of editor extensions by language, for `target` (the
 * system chosen on the Tools page; this PC's when omitted). */
export function extensionsMatrix(target?: string): Promise<ExtensionsMatrix> {
  return query<ExtensionsMatrix>("extensions_matrix", SAMPLE_MATRIX, target ? { target } : undefined);
}

/** Opens a kit console (a console window of the app) in a project. */
export function terminalOpen(project: string): Promise<void> {
  return act("terminal_open", { project });
}

/** The editor started, stopped or restarted; returns the function that stops listening. */
export function onEditorStatus(handler: (s: EditorStatus) => void): Promise<() => void> {
  return subscribe<EditorStatus>(EDITOR_STATUS, handler);
}

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
  return query<EditorSettingsView>(
    "project_editor_settings",
    {
      settings: { formatOnSave: true, tabSize: 4, insertSpaces: true, hidden: ["**/vendor"] },
      exists: true,
      readOnly: false,
    },
    { project },
  );
}

export function projectEditorSettingsSet(project: string, settings: EditorSettings): Promise<void> {
  return act("project_editor_settings_set", { project, settings });
}
