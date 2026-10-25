//! The programs the kit starts, and the promise that none of them outlives
//! the app, even when the app is killed (DECISIONS.md D-07, D-105).
//!
//! - Windows (`os/windows/jobs.rs`): one Job Object for the whole app, with
//!   `KILL_ON_JOB_CLOSE`; the app enters it at start, so every child is
//!   created inside it; a [`Job`] of its own nests in it.
//! - macOS and Linux (`os/unix/jobs.rs`): each program of a [`Job`] leads
//!   a process group of its own, killed when the job is dropped; the guard
//!   (`os/unix/guard.rs`), a fork of the app, ends the groups the app
//!   started if the app dies; what the app started is also stopped at exit
//!   by `sessions::stop_children`.

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command};

use anyhow::Result;

use crate::env::KitEnv;
use crate::os::current::jobs as imp;

/// A group of programs that ends together when it is dropped: `new`, and
/// `assign` of a child already started.
pub use imp::Job;

// What every system's `Job` offers, checked by the build of each.
const _: fn() -> Result<Job> = Job::new;
const _: fn(&Job, &Child) -> Result<()> = Job::assign;

/// Sets up what keeps the kit's programs from outliving the app: the app's
/// Job Object, or the guard. Must run before the app starts its first
/// program. The text, when there is one, says what works only in part, for
/// the log.
pub fn init() -> Result<Option<String>> {
    imp::init()
}

/// A command for a kit program: the kit environment only, no inherited
/// variables; no console window on Windows, a process group of its own
/// (no terminal signals) elsewhere.
pub fn kit_command(program: impl AsRef<OsStr>, env: &KitEnv) -> Command {
    imp::kit_command(program, env)
}

/// A script of the kit run in its own folder: a `.bat`/`.cmd` through
/// `cmd.exe` on Windows (DECISIONS.md D-17), a shell script through
/// `/bin/sh` elsewhere. `args` must be fixed words of the app, never text
/// from the user: cmd would interpret `&`, `|` and `%` in them.
pub fn batch_command(script: &Path, args: &[&str], env: &KitEnv) -> Command {
    imp::batch_command(script, args, env)
}

/// A command line of the user (a project's action) through the system
/// shell, with the kit environment: `cmd.exe /d /s /c "<line>"` on Windows
/// (passed as is: cmd reads its own quoting), `/bin/sh -c <line>` elsewhere.
pub fn shell_command(line: &str, env: &KitEnv) -> Command {
    imp::shell_command(line, env)
}

/// `s` as one word of the system shell of [`shell_command`]: in double
/// quotes for `cmd.exe` when it holds spaces or `&()^,;=`; in single
/// quotes for `/bin/sh` when it holds anything but plain characters, where
/// `$(…)`, backticks and `;` of a folder name in a cloned repository stay
/// text.
pub fn shell_quote(s: &str) -> String {
    imp::shell_quote(s)
}

/// The path `rel` under the environment variable `var`, quoted, as the
/// system shell reads it: `"%CP_DATA%\kit\x"` for `cmd.exe`,
/// `"$CP_DATA/kit/x"` for `/bin/sh`.
pub fn shell_var_path(var: &str, rel: &[&str]) -> String {
    imp::shell_var_path(var, rel)
}

/// Starts a program that belongs to the session: it ends with the app.
pub fn spawn(cmd: &mut Command) -> Result<Child> {
    imp::spawn(cmd)
}

/// Starts a program in `job`: dropping the job ends the program and
/// everything it started.
pub fn spawn_in(cmd: &mut Command, job: &Job) -> Result<Child> {
    imp::spawn_in(cmd, job)
}

/// Opens a folder (or a web address on macOS and Linux) with the system's
/// file manager, outside the app's processes: it stays open after the app.
pub fn open_with_explorer(target: impl AsRef<OsStr>) -> Result<()> {
    imp::open_with_explorer(target)
}

/// Opens a web address in the PC's browser, outside the app's processes.
pub fn open_url(url: &str) -> Result<()> {
    imp::open_url(url)
}

/// The guard's pid (macOS, Linux): a copy of the app, not one of its
/// programs, so the lists of the kit's programs leave it out. `None` on
/// Windows, and before the guard starts.
pub fn guard_pid() -> Option<u32> {
    imp::guard_pid()
}
