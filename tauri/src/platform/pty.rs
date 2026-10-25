//! A program on a new pseudo terminal, for the app's terminal (DECISIONS.md
//! D-109, D-148): the PTY of macOS and Linux through portable-pty, where
//! the program leads a session and a process group of its own, in a job
//! (`os/unix/pty.rs`); the kit's ConPTY on Windows, in a Job Object of its
//! own (`os/windows/conpty.rs`). The sessions (`editor/pty.rs`) see only
//! [`Io`] and the reader.

use std::io::Read;
use std::path::Path;

use anyhow::Result;

use crate::env::KitEnv;
use crate::os::current::pty as imp;

/// What runs on the pseudo terminal.
pub struct Program<'a> {
    pub program: &'a Path,
    pub args: &'a [String],
    /// `SHELL` for the session on macOS and Linux; not set on Windows.
    #[allow(dead_code)] // read on macOS and Linux only
    pub shell: &'a Path,
    /// The folder of the kit's ConPTY (Windows): `conpty.dll` and
    /// `OpenConsole.exe`; without it Windows' own, and the session says so.
    #[allow(dead_code)] // read on Windows only
    pub conpty: Option<&'a Path>,
}

/// The PTY and the program of a session, as the session uses them.
pub trait Io: Send + Sync {
    /// Queues `data` for the program; written in order, never blocking.
    fn write(&self, data: &[u8]) -> Result<()>;
    fn resize(&self, cols: u16, rows: u16) -> Result<()>;
    /// The program's process id (also its process group's).
    fn pid(&self) -> Option<u32>;
    /// Asks the program to end (`SIGHUP`, as a terminal closing).
    fn hang_up(&self);
    /// The processes left in the program's session (its process group's
    /// session on macOS and Linux, its job on Windows);
    /// `leader_running`: the program itself was not reaped yet.
    fn left(&self, leader_running: bool) -> Vec<u32>;
    /// Ends `pids` and their trees now.
    fn kill(&self, pids: &[u32]);
    /// Lets go of the PTY and of the job: what is left of the program's
    /// group is killed.
    fn release(&self);
    /// The program ended and its output was not closed by the system:
    /// the PTY lets the reader end (Windows closes the pseudo console,
    /// whose host would keep the pipe open). Default: nothing.
    fn program_ended(&self) {}
}

/// A program started on a new PTY.
pub struct Spawned {
    pub io: Box<dyn Io>,
    pub reader: Box<dyn Read + Send>,
    /// Waits for the program to end: its exit code (`None`: by a signal).
    pub wait: Box<dyn FnOnce() -> Option<u32> + Send>,
    /// A line of the app shown before the program's output.
    pub note: Option<String>,
}

/// Starts `program` in `dir` on a new pseudo terminal, with `env` only (and
/// what a terminal needs: `TERM`, `COLORTERM`, a UTF-8 locale). Texts in
/// `lang`.
pub fn spawn(program: &Program, env: &KitEnv, dir: &Path, lang: &str) -> Result<Spawned> {
    imp::spawn(program, env, dir, lang)
}

/// `program` can be started: a file, and on macOS and Linux one with the
/// execute permission, not on a `noexec` mount.
pub fn runnable(program: &Path) -> bool {
    imp::runnable(program)
}
