// Preparation (tauri/src/setup/task.rs): one run at a time, started by
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
 * drop the file on the app (tauri/src/setup/downloads.rs). */
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

/** The six systems, in the order of the columns of the grid (Rust
 * `Target::ALL`). */
export const TARGETS = ["win32-x64", "win32-arm64", "darwin-arm64", "darwin-x64", "linux-x64", "linux-arm64"] as const;

export type TargetId = (typeof TARGETS)[number];

/** What a system can do on this PC (Rust `router::target::TargetProfile`). */
export interface TargetProfile {
  id: TargetId;
  os: "win32" | "darwin" | "linux";
  arch: "x64" | "arm64";
  isHost: boolean;
  executableOnHost: boolean;
  emulatedOnHost: boolean;
}

/** One cell of the grid: a component on a system (Rust
 * `router::probe::TargetToolStatus`). */
export interface TargetToolStatus {
  id: string;
  name: string;
  available: boolean;
  emulated: boolean;
  /** Chosen, or needed on this system only by a chosen one (Zig next to C
   * and C++ on macOS and Linux). */
  chosen: boolean;
  installed: boolean;
  present: boolean;
  /** Something of it is on disk but not all: the first part missing, under
   * the kit's folder. A download or an installation that did not finish. */
  missing: string | null;
  version: string | null;
  needsReinstall: boolean;
  bootstrapStaged: boolean;
  /** Prepared from this PC it is only pre-downloaded: it completes on the
   * first start on that system. */
  firstStart: boolean;
  /** No ARM64 build, on a Windows ARM64 PC looking at its own system. */
  noArm64: boolean;
  /** Why it is not available, already translated. */
  unavailableReason: string | null;
}

export interface TargetSummary extends Omit<TargetProfile, "id"> {
  target: TargetId;
  availableCount: number;
  installedCount: number;
  tools: TargetToolStatus[];
}

/** The component × system grid of the Tools page (D-121). */
export interface TargetGrid {
  profiles: TargetProfile[];
  targets: TargetSummary[];
  /** «Piattaforma» chosen last (settings.json); null: this PC's. */
  chosen: TargetId | null;
}

export interface ComponentCheck {
  id: string;
  name: string;
  ok: boolean;
  answer: string;
}

export interface SetupSnapshot {
  running: boolean;
  cancelling: boolean;
  automatic: boolean;
  target?: string;
  total: number;
  steps: SetupStep[];
  /** The steps not started yet, in order (empty when the run ends). */
  queue: { id: string; name: string }[];
  error: string | null;
}

/** `app`: part of Atollo (the editor), shown with the required ones. */
export type Group = "required" | "app" | "agents" | "languages" | "databases";

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
  /** A system component, or needed by one (PowerShell 7 for the editor): always installed. */
  locked: boolean;
  version: string | null;
  /** This version of the app can install it on this kit. */
  available: boolean;
  x64Only: boolean;
  /** No ARM64 build, this kit is Windows ARM64 and so is this PC: why it is
   * not available. */
  noArm64: boolean;
  /** Not made for this kit's system (Windows only, on macOS or Linux; no
   * ARM64 build, for Windows ARM64 seen from another PC): why it is not
   * available, already translated. null on a Windows ARM64 PC (noArm64). */
  unavailableReason: string | null;
  needs: string[];
  /** x64 only, on an ARM64 PC with Windows 10 (no x64 emulation there). */
  needsWindows11: boolean;
  /** One folder per minor version, side by side; the others are always at
   * their latest version. */
  versioned: boolean;
  /** Its official site. */
  home: string;
  /** Bootstrap archives pre-downloaded for this target; finishes installing on
   * first start on that OS. */
  bootstrapStaged?: boolean;
  /** C and C++ on a Mac without Xcode Command Line Tools (Rust `xcode_missing`). */
  xcodeMissing?: boolean;
}

const IDLE: SetupSnapshot = {
  running: false,
  cancelling: false,
  automatic: false,
  target: "win32-x64",
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
    unavailableReason: null,
    needs: [],
    needsWindows11: false,
    versioned: ["python", "node", "php", "go", "java"].includes(id),
    home: "https://example.org",
    bootstrapStaged: false,
    ...extra,
  };
}

const ON = { chosen: true, installed: true, present: true };

const SAMPLE_COMPONENTS: ComponentInfo[] = [
  sampleComponent("vscodium-server", "Editor", "app", "VSCodium nel browser", 180, {
    ...ON,
    locked: true,
    version: "1.135.06055",
  }),
  sampleComponent("git", "Git", "languages", "controllo di versione", 350, { ...ON, version: "2.56.0" }),
  sampleComponent("gh", "GitHub CLI", "required", "GitHub dal terminale", 40, {
    ...ON,
    locked: true,
    version: "2.83.0",
  }),
  sampleComponent("claude", "Claude Code", "agents", "agente di Anthropic", 220, {
    ...ON,
    version: "2.1.299",
    needs: ["git"],
  }),
  sampleComponent("codex", "Codex", "agents", "agente di OpenAI", 120),
  sampleComponent("node", "Node.js", "languages", "JavaScript e TypeScript", 90, { ...ON, version: "24.11.0" }),
  sampleComponent("python", "Python", "languages", "con uv", 120),
  sampleComponent("php", "PHP", "languages", "linguaggio per il web", 60, { ...ON, version: "8.4.12" }),
  sampleComponent("apache", "Apache", "languages", "server web per PHP", 30, { needs: ["php"], x64Only: true }),
  sampleComponent("uv", "uv", "languages", "ambienti e pacchetti Python", 40, { ...ON, version: "0.9.5" }),
  sampleComponent("typst", "Typst", "languages", "documenti e PDF da testo", 50),
  sampleComponent("mysql", "MySQL", "databases", "database SQL del kit: server e client MySQL", 520, {
    ...ON,
    version: "9.7.2",
  }),
  sampleComponent("mongodb", "MongoDB", "databases", "database a documenti", 650, { x64Only: true }),
];

/** When the Tools page opens: what was deleted by hand from the kit's
 * folder is no longer chosen; the names of those. */
export function componentsSync(target?: string): Promise<string[]> {
  return query<string[]>("components_sync", [], target ? { target } : undefined);
}

export function componentsList(target?: string): Promise<ComponentInfo[]> {
  return query<ComponentInfo[]>("components_list", SAMPLE_COMPONENTS, target ? { target } : undefined);
}

export function componentsCheck(target?: string): Promise<ComponentCheck[]> {
  return query<ComponentCheck[]>("components_check", [], target ? { target } : undefined);
}

/** The preview's grid: the sample components on the six systems, this PC
 * Windows x64. */
function sampleGrid(): TargetGrid {
  const profiles: TargetProfile[] = TARGETS.map((id) => {
    const [os, arch] = id.split("-") as [TargetProfile["os"], TargetProfile["arch"]];
    const win = os === "win32";
    return { id, os, arch, isHost: id === "win32-x64", executableOnHost: win, emulatedOnHost: false };
  });
  const targets = profiles.map(({ id, ...p }): TargetSummary => {
    const tools = SAMPLE_COMPONENTS.map((c): TargetToolStatus => {
      // Every sample component has a build for every system (Apache too,
      // D-116); the x64 ones run emulated on Windows ARM64.
      const emulated = c.x64Only && id === "win32-arm64";
      const here = id === "win32-x64";
      return {
        id: c.id,
        name: c.name,
        available: true,
        emulated,
        chosen: c.chosen,
        installed: here && c.installed,
        present: here && c.present,
        missing: null,
        version: here ? c.version : null,
        needsReinstall: false,
        bootstrapStaged: false,
        firstStart: !p.executableOnHost && ["git", "python"].includes(c.id),
        noArm64: false,
        unavailableReason: null,
      };
    });
    return {
      ...p,
      target: id,
      availableCount: tools.filter((x) => x.available).length,
      installedCount: tools.filter((x) => x.installed).length,
      tools,
    };
  });
  return { profiles, targets, chosen: null };
}

/** The component × system grid and the system chosen last (D-121). */
export function targetProfiles(): Promise<TargetGrid> {
  return query<TargetGrid>("target_profiles", sampleGrid());
}

/** Remembers «Piattaforma» between sessions; null: this PC's system. */
export function toolsTargetSet(target: string | null): Promise<void> {
  return act("tools_target_set", { target });
}

export function setupStart(only: string[] | null, updateAll: boolean, target?: string): Promise<void> {
  return act("setup_start", target ? { only, updateAll, target } : { only, updateAll });
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
export function componentsApply(chosen: string[], updateAll: boolean, target?: string): Promise<void> {
  return act("components_apply", target ? { chosen, updateAll, target } : { chosen, updateAll });
}

/** Installs `id` now at its latest version; it becomes chosen, the other
 * changes of the page stay pending. Progress as `setup` events. */
export function componentsInstall(id: string, target?: string): Promise<void> {
  return act("components_install", target ? { id, target } : { id });
}

/** The packages kept to reinstall the conda environments without the
 * network after the kit moves (D-118). */
export interface CondaKept {
  /** Some component cannot follow the kit on this system. */
  applies: boolean;
  bytes: number;
  /** `<id>@<line>` of each list. */
  lists: string[];
}

const SAMPLE_CONDA: CondaKept = { applies: true, bytes: 412 << 20, lists: ["r@4.5", "ruby@3.4"] };

export function condaKept(): Promise<CondaKept> {
  return query<CondaKept>("conda_kept", SAMPLE_CONDA);
}

/** «Libera»: the next move downloads them again. */
export function condaKeptFree(): Promise<CondaKept> {
  return query<CondaKept>("conda_kept_free", { ...SAMPLE_CONDA, bytes: 0, lists: [] });
}
