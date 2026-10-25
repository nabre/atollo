//! The kit's terminal inside the app, on every system (DECISIONS.md D-109,
//! D-148): one pseudo terminal per session, shown by the main window with
//! xterm.js (`ui/src/terminal`). Never the PC's terminal program.
//!
//! - a session runs the kit's PowerShell (on macOS and Linux, without it,
//!   the system's `/bin/bash` with the kit's `bashrc`), or an agent; with
//!   the kit environment only, in the chosen folder;
//! - on macOS and Linux its program leads a session and a process group of
//!   its own (`setsid` of the PTY), put in a job (`platform/jobs.rs`); on Windows
//!   it runs on the kit's ConPTY (`os/windows/conpty.rs`) in a Job Object of
//!   its own.
//!   Closing the session hangs it up (`SIGHUP`, or the console closing),
//!   then ends what is left of its tree, its group and its session (the
//!   jobs of an interactive shell, in groups of their own);
//! - the output goes to the window as text (UTF-8 decoded across reads) in
//!   numbered chunks, event `pty://<id>`; the last [`BACKLOG`] bytes stay in
//!   the session for a view that attaches later (the panel opens after the
//!   first prompt was written), and the numbers tell it which events it
//!   already has;
//! - the keys typed are written in order by a thread of the session, so a
//!   program that does not read never blocks the window.
//!
//! Only the pseudo terminal itself is code of one system (`platform::pty`:
//! `os/unix/pty.rs`, `os/windows/conpty.rs`); the sessions, the output and
//! the choice of the program are plain code, tested on every system.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use serde::Serialize;

use crate::kit::Kit;
use crate::platform::pty::{Io, Spawned};

/// Output kept per session for a view that attaches late.
pub const BACKLOG: usize = 256 * 1024;
/// How long a program has to end after the hang-up before its tree is
/// killed.
pub const GRACE: Duration = Duration::from_secs(2);
/// How long the exit of a program waits for the rest of its output (a job
/// left in the background keeps the PTY open after it).
pub const DRAIN: Duration = Duration::from_millis(500);
/// The system's bash, never one of the user's PATH (cfg.rs uses it too).
pub const BASH: &str = super::cfg::BASH;

/// A session as the window shows it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PtyInfo {
    pub id: u64,
    /// The folder's name and what runs (`app · bash`, `app · claude`).
    pub title: String,
    pub dir: String,
    /// False once its program ended (the tab stays until it is closed).
    pub running: bool,
    /// The exit code, when it ended with one.
    pub code: Option<u32>,
    /// Shown by a window of its own (`terminal-<id>`), not by the panel.
    pub detached: bool,
}

/// Output of a session: the chunk `seq` (1, 2, ...), or for a backlog
/// everything kept up to the chunk `seq`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Chunk {
    pub seq: u64,
    pub data: String,
}

/// UTF-8 decoding of a byte stream read in pieces: a character split
/// between two reads is kept for the next one; invalid bytes become U+FFFD.
#[derive(Debug, Default)]
pub struct Utf8 {
    carry: Vec<u8>,
}

impl Utf8 {
    pub fn push(&mut self, bytes: &[u8]) -> String {
        self.carry.extend_from_slice(bytes);
        let mut out = String::new();
        let mut rest: &[u8] = &self.carry;
        loop {
            match std::str::from_utf8(rest) {
                Ok(s) => {
                    out.push_str(s);
                    rest = &[];
                    break;
                }
                Err(e) => {
                    let (good, bad) = rest.split_at(e.valid_up_to());
                    out.push_str(std::str::from_utf8(good).unwrap_or_default());
                    match e.error_len() {
                        Some(n) => {
                            out.push(char::REPLACEMENT_CHARACTER);
                            rest = &bad[n..];
                        }
                        // An incomplete character at the end: wait for more.
                        None => {
                            rest = bad;
                            break;
                        }
                    }
                }
            }
        }
        self.carry = rest.to_vec();
        out
    }

    /// The end of the stream: an incomplete character left is invalid.
    pub fn finish(&mut self) -> String {
        if self.carry.is_empty() {
            String::new()
        } else {
            self.carry.clear();
            char::REPLACEMENT_CHARACTER.to_string()
        }
    }
}

/// The last output of a session, at most `cap` bytes, and the number of
/// the last chunk in it.
#[derive(Debug)]
pub struct Backlog {
    text: String,
    seq: u64,
    cap: usize,
}

impl Default for Backlog {
    fn default() -> Self {
        Backlog::with_cap(BACKLOG)
    }
}

impl Backlog {
    pub fn with_cap(cap: usize) -> Backlog {
        Backlog {
            text: String::new(),
            seq: 0,
            cap,
        }
    }

    /// Adds a chunk; returns its number.
    pub fn push(&mut self, data: &str) -> u64 {
        self.seq += 1;
        self.text.push_str(data);
        if self.text.len() > self.cap {
            let mut cut = self.text.len() - self.cap;
            // Always at a character boundary, and at a line start when one
            // is near, so the view does not begin inside an escape sequence.
            while !self.text.is_char_boundary(cut) {
                cut += 1;
            }
            if let Some(nl) = self.text[cut..].find('\n').filter(|&n| n < 4096) {
                cut += nl + 1;
            }
            self.text.drain(..cut);
        }
        self.seq
    }

    pub fn snapshot(&self) -> Chunk {
        Chunk {
            seq: self.seq,
            data: self.text.clone(),
        }
    }
}

/// What a session runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: PathBuf,
    pub args: Vec<String>,
    /// `SHELL` for the session on macOS and Linux: what runs, or bash for
    /// an agent (Claude Code and Codex run their commands with it), never
    /// the user's own. Not set on Windows.
    pub shell: PathBuf,
    pub title: String,
    /// The folder of the kit's ConPTY (Windows): `conpty.dll` and
    /// `OpenConsole.exe`.
    pub conpty: Option<PathBuf>,
}

/// The kit's ConPTY folder for a session of `kit` (Windows only).
fn conpty_dir(kit: &Kit) -> Option<PathBuf> {
    kit.target()
        .is_windows()
        .then(|| crate::setup::kit_path(kit, "conpty"))
}

/// The kit's PowerShell.
fn pwsh(kit: &Kit) -> PathBuf {
    crate::setup::kit_path(kit, r"pwsh\pwsh.exe")
}

/// A batch file (`agy.cmd`): Windows runs it with cmd only.
fn is_batch(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
}

fn title(dir: &Path, what: &str) -> String {
    let name = dir
        .file_name()
        .map_or_else(|| dir.display().to_string(), |n| n.to_string_lossy().into());
    format!("{name} · {what}")
}

/// The kit's shell for `kit`'s system (D-109, D-148): its PowerShell; on
/// macOS and Linux without it `/bin/bash --rcfile <kit>/data/kit/bashrc -i`
/// (bash 3.2 on macOS reads it too, D-105). Never a shell of the user's
/// PATH: on Windows without the kit's PowerShell the session does not
/// start (it says the program is not installed).
pub fn shell(kit: &Kit, dir: &Path) -> Launch {
    let pwsh = pwsh(kit);
    if pwsh.is_file() || kit.target().is_windows() {
        return Launch {
            args: super::cfg::pwsh_args(kit),
            shell: pwsh.clone(),
            program: pwsh,
            title: title(dir, "pwsh"),
            conpty: conpty_dir(kit),
        };
    }
    Launch {
        program: PathBuf::from(BASH),
        args: super::cfg::bash_args(kit),
        shell: PathBuf::from(BASH),
        title: title(dir, "bash"),
        conpty: None,
    }
}

/// An agent or another program of the kit, started directly (Antigravity's
/// `agy` is a sh script of the kit, written with its execute bit). On
/// Windows a batch (`agy.cmd`) runs through the kit's PowerShell
/// (`-Command "& '<path>'; exit $LASTEXITCODE"`): cmd would read `&` and
/// `%` in the path (D-17, D-45).
pub fn command(kit: &Kit, command: &Path, dir: &Path) -> Launch {
    let name = command.file_stem().map_or_else(
        || command.display().to_string(),
        |n| n.to_string_lossy().into(),
    );
    let (program, args) = if kit.target().is_windows() && is_batch(command) {
        let quoted = super::terminal::ps_single_quoted(&command.display().to_string());
        let mut args: Vec<String> = [
            "-NoLogo",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
        ]
        .map(String::from)
        .into();
        args.push(format!("& '{quoted}'; exit $LASTEXITCODE"));
        (pwsh(kit), args)
    } else {
        (command.to_path_buf(), Vec::new())
    };
    Launch {
        program,
        args,
        shell: PathBuf::from(BASH),
        title: title(dir, &name),
        conpty: conpty_dir(kit),
    }
}

/// A program of the PC or of the kit with its arguments (`sbx exec` of a
/// project's microVM, D-167), started directly.
pub fn program(kit: &Kit, program: &Path, args: Vec<String>, dir: &Path, what: &str) -> Launch {
    Launch {
        program: program.to_path_buf(),
        args,
        shell: PathBuf::from(BASH),
        title: title(dir, what),
        conpty: conpty_dir(kit),
    }
}

/// What portable-pty and Windows would not tell clearly: it falls back to
/// `$HOME` for a folder that is not one, and a program that cannot start
/// would look like one that printed nothing and ended with 1. Errors in
/// `lang`.
pub fn ready(launch: &Launch, dir: &Path, lang: &str) -> Result<()> {
    let shown = dir.display().to_string();
    anyhow::ensure!(
        dir.is_dir(),
        crate::common::i18n::t(lang, "La cartella {0} non esiste.", &[&shown])
    );
    let shown = launch.program.display().to_string();
    anyhow::ensure!(
        crate::platform::pty::runnable(&launch.program),
        crate::common::i18n::t(lang, "{0} non è installato.", &[&shown])
    );
    Ok(())
}

/// The bytes of xterm.js's binary input (its `onBinary`: mouse reports of
/// the X10 kind past column 95), sent as text with one character per byte;
/// `None` for a character that is no byte.
pub fn latin1(data: &str) -> Option<Vec<u8>> {
    data.chars()
        .map(|c| u8::try_from(u32::from(c)).ok())
        .collect()
}

/// How long the exit of a program waits for its output to end by itself
/// before the PTY is told the program ended.
const SETTLE: Duration = Duration::from_millis(200);

struct State {
    backlog: Backlog,
    decoder: Utf8,
    running: bool,
    code: Option<u32>,
    detached: bool,
}

pub struct Session {
    id: u64,
    title: String,
    dir: String,
    io: Box<dyn Io>,
    state: Mutex<State>,
}

impl Session {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn info(&self) -> PtyInfo {
        let s = self.state();
        PtyInfo {
            id: self.id,
            title: self.title.clone(),
            dir: self.dir.clone(),
            running: s.running,
            code: s.code,
            detached: s.detached,
        }
    }

    /// Output read from the PTY; the chunk to send, if any text came out.
    pub fn output(&self, bytes: &[u8]) -> Option<Chunk> {
        let mut s = self.state();
        let data = s.decoder.push(bytes);
        Self::chunk(&mut s, data)
    }

    /// The end of the output.
    pub fn flush(&self) -> Option<Chunk> {
        let mut s = self.state();
        let data = s.decoder.finish();
        Self::chunk(&mut s, data)
    }

    fn chunk(s: &mut State, data: String) -> Option<Chunk> {
        if data.is_empty() {
            return None;
        }
        let seq = s.backlog.push(&data);
        Some(Chunk { seq, data })
    }

    /// The program ended with `code` (`None`: by a signal).
    pub fn exited(&self, code: Option<u32>) {
        let mut s = self.state();
        s.running = false;
        s.code = code;
    }

    pub fn running(&self) -> bool {
        self.state().running
    }

    /// Whether the program ended and nothing is left in its session.
    fn settled(&self) -> bool {
        !self.running() && self.io.left(false).is_empty()
    }

    /// Hangs up the program, waits up to `grace` for it and its session to
    /// end, kills what did not, and lets go of the PTY and the job.
    pub fn end(&self, grace: Duration) {
        // A program that already ended had its hang-up from its shell, or
        // never had a terminal of its own to lose: what outlived it is
        // killed at once.
        if self.running() {
            self.io.hang_up();
            let until = Instant::now() + grace;
            while !self.settled() && Instant::now() < until {
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        self.finish();
    }

    /// Kills the program's tree if it still runs and whatever is left in
    /// its session, and lets go of the PTY and the job. Not only while the
    /// program runs: an interactive bash puts each job in a group of its
    /// own, and on the hang-up it ends at once while a job that ignores
    /// SIGHUP (`caddy run`, `nohup cmd &`) keeps running, reparented, out
    /// of reach of the job's group and of the app's tree.
    fn finish(&self) {
        let running = self.running();
        let mut pids = self.io.left(running);
        if running {
            pids.extend(self.io.pid());
        }
        pids.sort_unstable();
        pids.dedup();
        if !pids.is_empty() {
            self.io.kill(&pids);
        }
        self.io.release();
    }
}

/// Reads the PTY until its end, sending every chunk to `emit`.
pub fn pump(session: &Session, mut reader: impl Read, emit: impl Fn(&Chunk)) {
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Some(chunk) = session.output(&buf[..n]) {
                    emit(&chunk);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            // EIO on Linux once the program and its children closed the PTY.
            Err(_) => break,
        }
    }
    if let Some(chunk) = session.flush() {
        emit(&chunk);
    }
}

/// The sessions of the app, open or ended but still shown.
#[derive(Default, Clone)]
pub struct Sessions {
    map: Arc<Mutex<BTreeMap<u64, Arc<Session>>>>,
    next: Arc<AtomicU64>,
}

impl Sessions {
    fn lock(&self) -> MutexGuard<'_, BTreeMap<u64, Arc<Session>>> {
        self.map.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn get(&self, id: u64) -> Result<Arc<Session>> {
        match self.lock().get(&id) {
            Some(s) => Ok(Arc::clone(s)),
            None => bail!("terminal session {id} not found"),
        }
    }

    /// Adds a session whose program runs now.
    pub(crate) fn insert(&self, title: String, dir: &Path, io: Box<dyn Io>) -> Arc<Session> {
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        let session = Arc::new(Session {
            id,
            title,
            dir: dir.display().to_string(),
            io,
            state: Mutex::new(State {
                backlog: Backlog::default(),
                decoder: Utf8::default(),
                running: true,
                code: None,
                detached: false,
            }),
        });
        self.lock().insert(id, Arc::clone(&session));
        session
    }

    pub fn list(&self) -> Vec<PtyInfo> {
        self.lock().values().map(|s| s.info()).collect()
    }

    /// Sessions whose program still runs.
    pub fn running(&self) -> usize {
        self.lock().values().filter(|s| s.running()).count()
    }

    /// Process ids of the programs that still run and of what is left in
    /// their sessions, also of a session whose program ended (its jobs):
    /// "Rimuovi l'unità" ends them with their trees.
    pub fn pids(&self) -> Vec<u32> {
        let mut pids: Vec<u32> = Vec::new();
        for s in self.lock().values() {
            let running = s.running();
            pids.extend(s.io.left(running));
            if running {
                pids.extend(s.io.pid());
            }
        }
        pids.sort_unstable();
        pids.dedup();
        pids
    }

    pub fn info(&self, id: u64) -> Result<PtyInfo> {
        Ok(self.get(id)?.info())
    }

    /// Moves session `id` to a window of its own (`true`) or back to the
    /// panel; its new state.
    pub fn set_detached(&self, id: u64, detached: bool) -> Result<PtyInfo> {
        let session = self.get(id)?;
        session.state().detached = detached;
        Ok(session.info())
    }

    pub fn backlog(&self, id: u64) -> Result<Chunk> {
        Ok(self.get(id)?.state().backlog.snapshot())
    }

    pub fn write(&self, id: u64, data: &str) -> Result<()> {
        self.write_bytes(id, data.as_bytes())
    }

    pub fn write_bytes(&self, id: u64, data: &[u8]) -> Result<()> {
        let session = self.get(id)?;
        if !session.running() {
            return Ok(());
        }
        session.io.write(data)
    }

    pub fn resize(&self, id: u64, cols: u16, rows: u16) -> Result<()> {
        if !(1..=1000).contains(&cols) || !(1..=1000).contains(&rows) {
            bail!("terminal size {cols}x{rows} out of range");
        }
        self.get(id)?.io.resize(cols, rows)
    }

    /// Ends the session's program (`grace` for it to end by itself) and
    /// forgets the session.
    pub fn close(&self, id: u64, grace: Duration) -> Result<()> {
        let session = self.lock().remove(&id);
        let Some(session) = session else {
            bail!("terminal session {id} not found");
        };
        session.end(grace);
        Ok(())
    }

    /// Every session, at the app's exit: hung up together, then what did
    /// not end within `grace` is killed.
    pub fn close_all(&self, grace: Duration) {
        let all: Vec<Arc<Session>> = std::mem::take(&mut *self.lock()).into_values().collect();
        // Only those hung up now are waited for, as in `Session::end`.
        let hung: Vec<&Arc<Session>> = all.iter().filter(|s| s.running()).collect();
        for s in &hung {
            s.io.hang_up();
        }
        let until = Instant::now() + grace;
        while !hung.iter().all(|s| s.settled()) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(50));
        }
        for s in &all {
            s.finish();
        }
    }

    /// Opens a session running `launch` in `dir` with the kit environment
    /// only; `on_open` gets the new session before anything else is sent,
    /// `on_output` every chunk, `on_exit` the session once its program
    /// ended (a program that ends at once never has its exit told first)
    /// and its output was read (see [`DRAIN`]). Errors in `lang`.
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        &self,
        launch: &Launch,
        env: &crate::env::KitEnv,
        dir: &Path,
        lang: &str,
        on_open: impl FnOnce(&PtyInfo),
        on_output: impl Fn(u64, &Chunk) + Send + 'static,
        on_exit: impl FnOnce(PtyInfo) + Send + 'static,
    ) -> Result<PtyInfo> {
        let spawned = spawn(launch, env, dir, lang)?;
        let session = self.insert(launch.title.clone(), dir, spawned.io);
        let opened = session.info();
        on_open(&opened);
        let id = session.id;
        if let Some(chunk) = spawned.note.and_then(|n| session.output(n.as_bytes())) {
            on_output(id, &chunk);
        }
        let reader = spawned.reader;
        let out = Arc::clone(&session);
        // Dropped once the pump sent its last chunk.
        let (drained, read_all) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            pump(&out, reader, |chunk| on_output(id, chunk));
            drop(drained);
        });
        let wait = spawned.wait;
        let waited = Arc::clone(&session);
        std::thread::spawn(move || {
            let code = wait();
            waited.exited(code);
            // The exit after the program's last lines, so the view's note
            // never lands inside them; not later than DRAIN when a job left
            // in the background keeps the PTY open. A pseudo console of
            // Windows keeps its output open until it is closed.
            let settled = read_all.recv_timeout(SETTLE);
            if matches!(settled, Err(std::sync::mpsc::RecvTimeoutError::Timeout)) {
                waited.io.program_ended();
                let _ = read_all.recv_timeout(DRAIN);
            }
            on_exit(waited.info());
        });
        Ok(opened)
    }
}

/// Starts `launch` on a new pseudo terminal (`platform::pty`) with the kit
/// environment only, once [`ready`] says it can.
fn spawn(launch: &Launch, env: &crate::env::KitEnv, dir: &Path, lang: &str) -> Result<Spawned> {
    ready(launch, dir, lang)?;
    let program = crate::platform::pty::Program {
        program: &launch.program,
        args: &launch.args,
        shell: &launch.shell,
        conpty: launch.conpty.as_deref(),
    };
    crate::platform::pty::spawn(&program, env, dir, lang)
}

#[cfg(test)]
#[path = "../../../test/rust/editor/pty.rs"]
mod tests;
