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
}

export function updateStatus(): Promise<UpdateStatus> {
  return query<UpdateStatus>("update_status", {
    current: "0.1.0",
    latest: null,
    staged: null,
    checking: false,
    error: null,
  });
}

export function updateCheck(): Promise<UpdateStatus> {
  return browserPreview ? updateStatus() : invoke<UpdateStatus>("update_check");
}
