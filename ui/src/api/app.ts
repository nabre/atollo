// The app: its information, settings and updates, and what it opens outside
// (the PC's browser, File Explorer). Rust: tauri/src/commands/app.rs.

import { invoke } from "@tauri-apps/api/core";
import { act, browserPreview, previewLang, query } from "./core";
// The preview's version is the app's (ui/package.json, as tauri/Cargo.toml):
// the real app reads its own from Tauri.
import { version as PREVIEW_VERSION } from "../../package.json";

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
  /** File system of the kit's disk; `null` when it could not be read. */
  diskFs: "NTFS" | "ReFS" | "FAT32" | "exFAT" | "POSIX" | "other" | null;
  diskKind: "fixed" | "removable" | "remote" | "other" | null;
  /** The disk the kit's programs run from is mounted `noexec` (macOS and
   * Linux, D-105); `null` on Windows. */
  diskNoexec: boolean | null;
  /** The app's notice about the kit's disk, translated; `null` on Windows
   * or when all is well. */
  diskNote: string | null;
  /** The host's temporary folder when the programs use it; `null`: `data\tmp`. */
  systemTemp: string | null;
}

/** The browser preview's answers (core.ts), reused by the tests. */
export const SAMPLE_INFO: AppInfo = {
  version: PREVIEW_VERSION,
  root: "E:\\Atollo",
  data: "E:\\Atollo\\data",
  projects: "E:\\Atollo\\projects",
  platform: "win32-x64",
  lang: previewLang,
  movedFrom: null,
  webview: "anteprima nel browser",
  pathEntries: 0,
  watcherUnsafeFolder: null,
  firstRun: false,
  otherKits: [],
  diskFs: "NTFS",
  diskKind: "fixed",
  diskNoexec: null,
  diskNote: null,
  systemTemp: null,
};

export function appInfo(): Promise<AppInfo> {
  return query<AppInfo>("app_info", SAMPLE_INFO);
}

/** Opens a kit folder in File Explorer: "agents", "db" or "projects" (+ project). */
export function folderOpen(
  which: "agents" | "backup" | "db" | "diagnostics" | "models" | "projects",
  project: string | null = null,
): Promise<void> {
  return act("folder_open", { which, project });
}

/** «Esporta diagnostica»: the zip for a report of a problem, its secrets
 * masked (tauri/src/diagnostics.rs); its path. */
export function diagnosticsExport(): Promise<string> {
  return query<string>(
    "diagnostics_export",
    "E:\\Atollo\\data\\app\\diagnostics\\atollo-diagnostica-20261022-101500.zip",
  );
}

/** One check of «Salute del kit» (tauri/src/health.rs). */
export interface HealthFinding {
  id: "disk" | "space" | "path" | "owners" | "libs";
  level: "ok" | "warn" | "err";
  text: string;
  /** What to do, when there is something to do. */
  fix: string | null;
}

export const SAMPLE_HEALTH: HealthFinding[] = [
  {
    id: "disk",
    level: "ok",
    text: "Il disco del kit è exFAT: va bene; mancano solo collegamenti e permessi, di cui il kit fa a meno.",
    fix: null,
  },
  {
    id: "space",
    level: "warn",
    text: "Sul disco del kit restano 3.2 GB: bastano per poco (un modello locale ne pesa da 5 a 20).",
    fix: null,
  },
  { id: "path", level: "ok", text: "Percorso del kit senza caratteri a rischio.", fix: null },
  { id: "owners", level: "ok", text: "I file del kit si leggono tutti.", fix: null },
];

/** «Salute del kit»: the drive, the space, the kit's folder, files of
 * another account, the libraries of Linux. */
export function kitHealth(): Promise<HealthFinding[]> {
  return query<HealthFinding[]>("kit_health", SAMPLE_HEALTH);
}

/** «Segnala un problema»: a new issue of Atollo on GitHub, in the browser,
 * with the version and the system filled in. */
export function issueOpen(): Promise<void> {
  return act("issue_open");
}

/** Opens a local address (http://localhost:<port>/) in the PC's browser. */
export function urlOpen(url: string): Promise<void> {
  return act("url_open", { url });
}

/** Rust's compiler on Windows (Tools › Rust, D-137): "msvc", or null for
 * the default (GNU). From the next install or update of Rust. */
export function rustToolchainSet(toolchain: "msvc" | null): Promise<void> {
  return act("rust_toolchain_set", { toolchain });
}

/** Atollo's privacy policy, in the app's language, in the PC's browser. */
export function privacyOpen(): Promise<void> {
  return act("privacy_open");
}

export interface Settings {
  lang: "it" | "en" | "fr" | "de" | null;
  network: {
    httpsProxy: string | null;
    httpProxy: string | null;
    noProxy: string | null;
    extraCaCerts: string | null;
    allowLan: boolean;
  };
  /** Never sent to the interface (Rust skips it): see `geminiKeySet`;
   * the key goes to `settingsSet` on its own. */
  geminiApiKey?: string | null;
  autoUpdate: boolean;
  rustToolchain: "gnu" | "msvc" | null;
  /** `TEMP` of the programs; applies to the consoles opened after saving. */
  programTemp: "auto" | "kit" | "system";
  /** Light or dark for every window, the editor's too (D-137). */
  theme: "system" | "light" | "dark";
  /** Keeps the packages of the conda environments to reinstall them
   * without the network after a move (D-118). */
  keepCondaPackages: boolean;
  /** «Piattaforma» of the Tools page (D-121): kept by `settings_set`,
   * written only by `tools_target_set`. */
  toolsTarget: string | null;
}

export interface SettingsView {
  settings: Settings;
  geminiKeySet: boolean;
  /** The file could not be read: defaults shown, the file rewritten at the
   * first change. */
  problem: string | null;
}

export const SAMPLE_SETTINGS: SettingsView = {
  settings: {
    lang: null,
    network: { httpsProxy: null, httpProxy: null, noProxy: null, extraCaCerts: null, allowLan: false },
    autoUpdate: true,
    rustToolchain: null,
    programTemp: "auto",
    theme: "system",
    keepCondaPackages: true,
    toolsTarget: null,
  },
  geminiKeySet: false,
  problem: null,
};

export function settingsGet(): Promise<SettingsView> {
  return query<SettingsView>("settings_get", SAMPLE_SETTINGS);
}

/** Saves and applies: the Settings page calls it for each change (D-137).
 * `gemini`: null keeps the saved key, "" removes it. */
export function settingsSet(settings: Settings, gemini: string | null): Promise<void> {
  return act("settings_set", { settings, gemini });
}

export interface UpdateStatus {
  current: string;
  latest: string | null;
  /** Downloaded: replaces the exe when the app closes. */
  staged: string | null;
  checking: boolean;
  error: string | null;
  /** A newer version that this build cannot install itself (a plain binary
   * off Windows, from the tar.gz): downloaded by hand from the release page. */
  manual: boolean;
}

export const SAMPLE_UPDATE: UpdateStatus = {
  current: PREVIEW_VERSION,
  latest: null,
  staged: null,
  checking: false,
  error: null,
  manual: false,
};

export function updateStatus(): Promise<UpdateStatus> {
  return query<UpdateStatus>("update_status", SAMPLE_UPDATE);
}

export function updateCheck(): Promise<UpdateStatus> {
  return browserPreview ? updateStatus() : invoke<UpdateStatus>("update_check");
}
