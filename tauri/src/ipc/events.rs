//! The events Rust sends to the interface, the other half of the IPC next
//! to the commands: every name in one place, the same as
//! `ui/src/ipc/events.ts` (a test compares the two files). Who emits an
//! event uses the constant, never the string.

/// A project's action changed state or wrote output
/// (`projects::run::Event`).
pub const RUN: &str = "run";
/// The check of new versions ended (no payload).
pub const UPDATES: &str = "updates";
/// A project chosen in the tray menu: its page opens (its id).
pub const OPEN_PROJECT: &str = "open-project";
/// A step of a new project from a framework (`FrameworkEvent`).
pub const FRAMEWORK: &str = "framework";
/// The GitHub account's state (`GithubStatus`).
pub const GITHUB: &str = "github";
/// The editor started, stopped or restarted (`EditorStatus`).
pub const EDITOR_STATUS: &str = "editor-status";
/// The window is closing with something open: the interface asks
/// (`OpenNow`).
pub const CLOSE_REQUESTED: &str = "close-requested";
/// The download of a local model (`PullEvent`).
pub const LLM_PULL: &str = "llm-pull";
/// The preparation task (`Snapshot`).
pub const SETUP: &str = "setup";
/// A terminal session opened: the payload is its `PtyInfo`.
pub const PTY_OPENED: &str = "pty-opened";
/// The program of a terminal session ended: the payload is its `PtyInfo`.
pub const PTY_EXIT: &str = "pty-exit";
/// A terminal session moved to a window of its own or back to the panel:
/// the payload is its `PtyInfo`.
pub const PTY_MOVED: &str = "pty-moved";
/// A terminal session was closed (by the panel or by its own window): the
/// payload is its id.
pub const PTY_CLOSED: &str = "pty-closed";

/// Every fixed name above, for the test that compares them with the
/// interface's.
#[cfg(test)]
pub const ALL: &[&str] = &[
    RUN,
    UPDATES,
    OPEN_PROJECT,
    FRAMEWORK,
    GITHUB,
    EDITOR_STATUS,
    CLOSE_REQUESTED,
    LLM_PULL,
    SETUP,
    PTY_OPENED,
    PTY_EXIT,
    PTY_MOVED,
    PTY_CLOSED,
];

/// The event with the output of terminal session `id`.
pub fn pty_output(id: u64) -> String {
    format!("pty://{id}")
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/events.rs"]
mod tests;
