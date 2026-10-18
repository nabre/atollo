// The app: its information, settings and updates, and what it opens outside
// (the PC's browser, File Explorer). Rust: tauri/src/commands/app.rs.

import { invoke } from "@tauri-apps/api/core";
import { act, browserPreview, query } from "./core";

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
  which: "agents" | "backup" | "db" | "projects",
  project: string | null = null,
): Promise<void> {
  return act("folder_open", { which, project });
}

/** Opens a local address (http://localhost:<port>/) in the PC's browser. */
export function urlOpen(url: string): Promise<void> {
  return act("url_open", { url });
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
  /** Never sent to the interface: see `geminiKeySet`. */
  geminiApiKey: string | null;
  autoUpdate: boolean;
  pythonVersion: string | null;
  phpVersion: string | null;
  rustToolchain: "gnu" | "msvc" | null;
  /** `TEMP` of the programs; applies to the consoles opened after saving. */
  programTemp: "auto" | "kit" | "system";
  /** Keeps the packages of the conda environments to reinstall them
   * without the network after a move (D-118). */
  keepCondaPackages: boolean;
  /** «Piattaforma» of the Tools page (D-121): kept by `settings_set`,
   * written only by `tools_target_set`. */
  toolsTarget?: string | null;
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
    programTemp: "auto",
    keepCondaPackages: true,
  },
  geminiKeySet: false,
  problem: null,
};

export function settingsGet(): Promise<SettingsView> {
  return query<SettingsView>("settings_get", SAMPLE_SETTINGS);
}

/** `gemini`: null keeps the saved key, "" removes it. */
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

export function updateStatus(): Promise<UpdateStatus> {
  return query<UpdateStatus>("update_status", {
    current: "0.1.0",
    latest: null,
    staged: null,
    checking: false,
    error: null,
    manual: false,
  });
}

export function updateCheck(): Promise<UpdateStatus> {
  return browserPreview ? updateStatus() : invoke<UpdateStatus>("update_check");
}
