// The events Rust sends to the interface, the other half of the IPC next to
// the commands: every name in one place, the same as
// tauri/src/ipc/events.rs (a Rust test compares the two files). The
// functions of this folder subscribe with these constants, never with the
// string.

/** A project's action changed state or wrote output. */
export const RUN = "run";
/** The check of new versions ended. */
export const UPDATES = "updates";
/** A project chosen in the tray menu: its page opens. */
export const OPEN_PROJECT = "open-project";
/** A step of a new project from a framework. */
export const FRAMEWORK = "framework";
/** The GitHub account's state. */
export const GITHUB = "github";
/** The editor started, stopped or restarted. */
export const EDITOR_STATUS = "editor-status";
/** The window is closing with something open: the interface asks. */
export const CLOSE_REQUESTED = "close-requested";
/** The download of a local model. */
export const LLM_PULL = "llm-pull";
/** The preparation task. */
export const SETUP = "setup";
/** A job changed: an install, an update, a VM's preparation. */
export const JOB = "job";
/** The VM went on or off, so the agents were enabled or disabled. */
export const LEVELS_CHANGED = "levels-changed";
/** A terminal session opened. */
export const PTY_OPENED = "pty-opened";
/** The program of a terminal session ended. */
export const PTY_EXIT = "pty-exit";
/** A terminal session moved to a window of its own or back to the panel. */
export const PTY_MOVED = "pty-moved";
/** A terminal session was closed (by the panel or by its own window). */
export const PTY_CLOSED = "pty-closed";

/** The event with the output of terminal session `id`. */
export function ptyOutput(id: number): string {
  return `pty://${String(id)}`;
}
