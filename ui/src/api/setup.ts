// Preparation (tauri/src/setup_task.rs): one run at a time, started by
// the app at launch when a required component is missing, by the editor and
// console buttons, or by the Tools page; files downloaded by hand (D-82).
// Rust: tauri/src/commands/setup.rs.

import { getCurrentWebview } from "@tauri-apps/api/webview";
import { act, browserPreview, query, subscribe } from "./core";

export type Outcome =
  | { kind: "installed"; version: string }
  | { kind: "upToDate"; version: string | null }
  | { kind: "removed" }
  | { kind: "inUse" }
  | { kind: "unsupported" }
  | { kind: "cancelled" }
  | { kind: "failed"; message: string; byHand: ByHand | null };

/** A download that failed: its address, to download it in the browser and
 * drop the file on the app (tauri/src/downloads.rs). */
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
  return query<SetupSnapshot>("setup_state", IDLE);
}

/** Every change of the preparation; returns the function that stops listening. */
export function onSetup(handler: (s: SetupSnapshot) => void): Promise<() => void> {
  return subscribe<SetupSnapshot>("setup", handler);
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
  return query<string[]>("components_sync", []);
}

export function componentsList(): Promise<ComponentInfo[]> {
  return query<ComponentInfo[]>("components_list", SAMPLE_COMPONENTS);
}

export function setupStart(only: string[] | null, updateAll: boolean): Promise<void> {
  return act("setup_start", { only, updateAll });
}

/** Files downloaded by hand, kept for the next installation: their names. */
export function downloadsAdd(paths: string[]): Promise<string[]> {
  return query<string[]>("downloads_add", paths, { paths });
}

/** The same, picked in Windows' "Open" window; empty when closed. */
export function downloadsPick(): Promise<string[]> {
  return query<string[]>("downloads_pick", []);
}

/** Opens in the browser the address of the failed download of a step. */
export function setupByHandOpen(id: string, name: string): Promise<void> {
  return act("setup_by_hand_open", { id, name });
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
  return act("setup_cancel");
}

/** Saves the choice (ids to keep installed) and installs/removes in the background. */
export function componentsApply(chosen: string[], updateAll: boolean): Promise<void> {
  return act("components_apply", { chosen, updateAll });
}
