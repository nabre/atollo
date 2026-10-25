// Removing the drive and closing (tauri/src/platform/sessions.rs), the tray menu,
// the controls of the window's own title bar (shell/titlebar.tsx). Rust:
// tauri/src/ipc/window.rs, tauri/src/app/tray.rs; the window's controls are
// Tauri's (capabilities/main.json).

import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
import { act, browserPreview, query, subscribe } from "./core";
import { CLOSE_REQUESTED, OPEN_PROJECT } from "./events";

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
  /** The server of the local models, when it runs (0 or 1). */
  llm: number;
  /** Downloads of local models under way. */
  downloads: number;
}

/** The browser preview's answer (core.ts): nothing of the kit open. */
export const NOTHING_OPEN: OpenNow = {
  processes: [],
  editor: false,
  terminals: 0,
  actions: 0,
  setup: false,
  databases: 0,
  llm: 0,
  downloads: 0,
};

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
  return subscribe<OpenNow>(CLOSE_REQUESTED, handler);
}

/** A project chosen in the tray menu (tauri/src/app/tray.rs): its page opens. */
export function onOpenProject(handler: (id: string) => void): Promise<() => void> {
  return subscribe<string>(OPEN_PROJECT, handler);
}

/** The window, or null in the browser preview and in the tests. */
function currentWindow(): Window | null {
  if (browserPreview) return null;
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

function onWindow(f: (w: Window) => Promise<void>) {
  const w = currentWindow();
  if (w) f(w).catch(() => undefined);
}

export function windowMinimize(): void {
  onWindow((w) => w.minimize());
}

export function windowToggleMaximize(): void {
  onWindow((w) => w.toggleMaximize());
}

/** Same path as the Windows button: the app asks first if something is open. */
export function windowClose(): void {
  onWindow((w) => w.close());
}

/** Tells `handler` whether the window is maximized, now and after every
 * resize; returns the function that stops it (nothing outside the app). */
export function watchMaximized(handler: (maximized: boolean) => void): () => void {
  const w = currentWindow();
  if (!w) return () => undefined;
  let stop: (() => void) | null = null;
  let gone = false;
  const read = () => {
    w.isMaximized().then(
      (m) => {
        if (!gone) handler(m);
      },
      () => undefined,
    );
  };
  read();
  w.onResized(read).then(
    (f) => {
      if (gone) f();
      else stop = f;
    },
    () => undefined,
  );
  return () => {
    gone = true;
    stop?.();
  };
}
