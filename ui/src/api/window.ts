// Removing the drive and closing (tauri/src/os/sessions.rs), the tray menu.
// Rust: tauri/src/commands/window.rs, tauri/src/tray.rs.

import { act, query, subscribe } from "./core";

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
  return query<OpenNow>("eject_list", NOTHING_OPEN);
}

/** Stops every program of the kit and saves Antigravity's login in the kit. */
export function ejectRun(): Promise<OpenNow> {
  return query<OpenNow>("eject_run", NOTHING_OPEN);
}

/** Closes the app (after the user confirmed). */
export function appQuit(): Promise<void> {
  return act("app_quit");
}

/** The window is closing with something open: the interface asks. */
export function onCloseRequested(handler: (open: OpenNow) => void): Promise<() => void> {
  return subscribe<OpenNow>("close-requested", handler);
}

/** A project chosen in the tray menu (tauri/src/tray.rs): its page opens. */
export function onOpenProject(handler: (id: string) => void): Promise<() => void> {
  return subscribe<string>("open-project", handler);
}
