//! The kit's terminal inside the app, on every system (DECISIONS.md D-109,
//! D-148): one pseudo terminal per session, shown by the main window with
//! xterm.js (`ui/src/terminal`). Never the PC's terminal program.
//!
//! - a session runs the kit's PowerShell (on macOS and Linux, without it,
//!   the system's `/bin/bash` with the kit's `bashrc`), or an agent; with
//!   the kit environment only, in the chosen folder;
//! - on macOS and Linux its program leads a session and a process group of
//!   its own (`setsid` of the PTY), put in a job (`os/jobs.rs`); on Windows
//!   it runs on the kit's ConPTY (`conpty.rs`) in a Job Object of its own.
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
//! Only the pseudo terminal itself is code of one system (`unix` below,
//! `conpty.rs`); the sessions, the output and the choice of the program
//! are plain code, tested on every system.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use serde::Serialize;

use crate::kit::Kit;

/// Output kept per session for a view that attaches late.
pub const BACKLOG: usize = 256 * 1024;
/// How long a program has to end after the hang-up before its tree is
/// killed.
pub const GRACE: Duration = Duration::from_secs(2);
/// How long the exit of a program waits for the rest of its output (a job
/// left in the background keeps the PTY open after it).
pub const DRAIN: Duration = Duration::from_millis(500);
/// A session opened: the payload is its [`PtyInfo`].
pub const OPENED_EVENT: &str = "pty-opened";
/// The program of a session ended: the payload is its [`PtyInfo`].
pub const EXIT_EVENT: &str = "pty-exit";
/// The system's bash, never one of the user's PATH (cfg.rs uses it too).
pub const BASH: &str = super::cfg::BASH;

/// The event with the output of session `id`.
pub fn output_event(id: u64) -> String {
    format!("pty://{id}")
}

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
        .then(|| crate::setup::catalog::kit_path(kit, "conpty"))
}

/// The kit's PowerShell.
fn pwsh(kit: &Kit) -> PathBuf {
    crate::setup::catalog::kit_path(kit, r"pwsh\pwsh.exe")
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

/// What portable-pty and Windows would not tell clearly: it falls back to
/// `$HOME` for a folder that is not one, and a program that cannot start
/// would look like one that printed nothing and ended with 1. Errors in
/// `lang`.
pub fn ready(launch: &Launch, dir: &Path, lang: &str) -> Result<()> {
    let shown = dir.display().to_string();
    anyhow::ensure!(
        dir.is_dir(),
        crate::i18n::t(lang, "La cartella {0} non esiste.", &[&shown])
    );
    let shown = launch.program.display().to_string();
    anyhow::ensure!(
        runnable(&launch.program),
        crate::i18n::t(lang, "{0} non è installato.", &[&shown])
    );
    Ok(())
}

#[cfg(unix)]
fn runnable(program: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(path) = std::ffi::CString::new(program.as_os_str().as_bytes()) else {
        return false;
    };
    // access(X_OK) also refuses a file on a `noexec` mount.
    // SAFETY: a NUL-terminated path, only read.
    program.is_file() && unsafe { libc::access(path.as_ptr(), libc::X_OK) } == 0
}

#[cfg(windows)]
fn runnable(program: &Path) -> bool {
    program.is_file()
}

/// The bytes of xterm.js's binary input (its `onBinary`: mouse reports of
/// the X10 kind past column 95), sent as text with one character per byte;
/// `None` for a character that is no byte.
pub fn latin1(data: &str) -> Option<Vec<u8>> {
    data.chars()
        .map(|c| u8::try_from(u32::from(c)).ok())
        .collect()
}

/// The variable and value to add so that a session on `os` reads and
/// writes UTF-8 (typed accents, file names such as `projects/già`), from
/// the kit environment's `LC_ALL`, `LC_CTYPE` and `LANG`; `None` when the
/// first of them that is set (the one the C library reads) already names a
/// UTF-8 locale. An app started from Finder or by launchd has none of them,
/// and bash 3.2 or pwsh would then run in the C locale. `LANG` when none is
/// set, as VS Code does (`en_US.UTF-8` on macOS, which has no `C.UTF-8`);
/// `LC_CTYPE` over a `LANG` of another charset, which keeps its messages;
/// never over an `LC_ALL` chosen by the user. Not on Windows, where the
/// console is UTF-16 and the kit's profile sets the code pages.
#[cfg_attr(windows, allow(dead_code))]
pub fn utf8_locale(
    os: crate::kit::Os,
    lc_all: Option<&str>,
    lc_ctype: Option<&str>,
    lang: Option<&str>,
) -> Option<(&'static str, &'static str)> {
    fn set(v: Option<&str>) -> Option<&str> {
        v.filter(|v| !v.is_empty())
    }
    let utf8 = |v: &str| {
        let v = v.to_ascii_lowercase();
        v.contains("utf-8") || v.contains("utf8")
    };
    let value = if os.is_macos() {
        "en_US.UTF-8"
    } else {
        "C.UTF-8"
    };
    match (set(lc_all), set(lc_ctype), set(lang)) {
        (Some(_), _, _) => None,
        (None, Some(v), _) | (None, None, Some(v)) if utf8(v) => None,
        (None, None, None) => Some(("LANG", value)),
        _ => Some(("LC_CTYPE", value)),
    }
}

/// The processes of `procs` (pid, session id) left in the session that
/// `leader` led. Every job of the session keeps its id, the leader's pid,
/// unless it calls `setsid` itself, and the system gives that pid to no new
/// process while one of them lives. So a process with that pid, once the
/// leader was reaped, means the session is gone and the pid now another
/// program's: then none. Never the app itself. Unix only: on Windows the
/// session's Job Object holds them.
#[cfg_attr(windows, allow(dead_code))]
pub fn in_session(procs: &[(u32, u32)], leader: u32, leader_running: bool) -> Vec<u32> {
    if leader == 0 || (!leader_running && procs.iter().any(|&(pid, _)| pid == leader)) {
        return Vec::new();
    }
    let me = std::process::id();
    procs
        .iter()
        .filter(|&&(pid, sid)| sid == leader && pid != me)
        .map(|&(pid, _)| pid)
        .collect()
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
    /// The processes left in the program's session (see [`in_session`]);
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
struct Spawned {
    io: Box<dyn Io>,
    reader: Box<dyn Read + Send>,
    /// Waits for the program to end: its exit code (`None`: by a signal).
    wait: Box<dyn FnOnce() -> Option<u32> + Send>,
    /// A line of the app shown before the program's output.
    note: Option<String>,
}

/// How long the exit of a program waits for its output to end by itself
/// before the PTY is told the program ended.
const SETTLE: Duration = Duration::from_millis(200);

struct State {
    backlog: Backlog,
    decoder: Utf8,
    running: bool,
    code: Option<u32>,
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

/// Starts `launch` on a new PTY (unix) with the kit environment only.
#[cfg(unix)]
fn spawn(launch: &Launch, env: &crate::env::KitEnv, dir: &Path, lang: &str) -> Result<Spawned> {
    let spawned = unix::spawn(launch, env, dir, lang)?;
    let mut child = spawned.child;
    Ok(Spawned {
        io: Box::new(spawned.io),
        reader: spawned.reader,
        wait: Box::new(move || match child.wait() {
            Ok(status) if status.signal().is_none() => Some(status.exit_code()),
            _ => None,
        }),
        note: None,
    })
}

/// Starts `launch` on a new pseudo console (Windows, `conpty.rs`) with the
/// kit environment only; without the kit's ConPTY the session says that
/// Windows' own is used.
#[cfg(windows)]
fn spawn(launch: &Launch, env: &crate::env::KitEnv, dir: &Path, lang: &str) -> Result<Spawned> {
    ready(launch, dir, lang)?;
    let mut env = env.clone();
    env.set("COLORTERM", "truecolor");
    let spawned = super::conpty::spawn(
        &launch.program,
        &launch.args,
        &env,
        dir,
        launch.conpty.as_deref(),
    )?;
    let note = spawned.system.then(|| {
        let text = crate::i18n::t(
            lang,
            "Il ConPTY del kit non è installato: questo terminale usa quello di Windows, e alcuni programmi (Claude Code) possono non ricevere i tasti.",
            &[],
        );
        format!("\x1b[2m{text}\x1b[0m\r\n")
    });
    let process = spawned.process;
    Ok(Spawned {
        io: Box::new(spawned.io),
        reader: Box::new(spawned.reader),
        wait: Box::new(move || process.wait()),
        note,
    })
}

#[cfg(unix)]
mod unix {
    use std::io::{Read, Write};
    use std::path::Path;
    use std::sync::mpsc::{channel, Sender};
    use std::sync::{Mutex, PoisonError};

    use anyhow::{Context, Result};
    use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

    use super::{Io, Launch};
    use crate::env::KitEnv;
    use crate::os::jobs::Job;

    pub struct Spawned {
        pub io: PtyIo,
        pub reader: Box<dyn Read + Send>,
        pub child: Box<dyn Child + Send + Sync>,
    }

    pub struct PtyIo {
        pid: Option<u32>,
        master: Mutex<Option<Box<dyn MasterPty + Send>>>,
        input: Mutex<Option<Sender<Vec<u8>>>>,
        job: Mutex<Option<Job>>,
    }

    /// The size before the view measures itself (`pty_resize`).
    const SIZE: PtySize = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    pub fn spawn(launch: &Launch, env: &KitEnv, dir: &Path, lang: &str) -> Result<Spawned> {
        super::ready(launch, dir, lang)?;
        let pair = native_pty_system().openpty(SIZE).context("openpty")?;
        let mut cmd = CommandBuilder::new(&launch.program);
        cmd.args(&launch.args);
        cmd.cwd(dir);
        cmd.env_clear();
        for (k, v) in env.iter() {
            cmd.env(k, v);
        }
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("SHELL", &launch.shell);
        let locale = super::utf8_locale(
            crate::kit::Os::current(),
            env.get("LC_ALL"),
            env.get("LC_CTYPE"),
            env.get("LANG"),
        );
        if let Some((name, value)) = locale {
            cmd.env(name, value);
        }
        let mut child = pair
            .slave
            .spawn_command(cmd)
            .with_context(|| format!("starting {}", launch.program.display()))?;
        // The reader ends only once every copy of the PTY's other side is
        // closed: the app keeps none.
        drop(pair.slave);
        let pid = child.process_id();
        // The program leads its own group (setsid): the job kills that
        // group when the session lets it go, as for the app's other jobs.
        let prepared = (|| -> Result<_> {
            let job = Job::new()?;
            let leader: &dyn Child = &*child;
            let std_child = leader
                .downcast_ref::<std::process::Child>()
                .context("pty child is not a std::process::Child")?;
            job.assign(std_child)?;
            let reader = pair.master.try_clone_reader().context("pty reader")?;
            let writer = pair.master.take_writer().context("pty writer")?;
            Ok((job, reader, writer))
        })();
        // Never a session outside the job on any error once it started:
        // its group would outlive it (BUG-V6-25).
        let (job, reader, mut writer) = match prepared {
            Ok(p) => p,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        };
        let (tx, rx) = channel::<Vec<u8>>();
        std::thread::spawn(move || {
            for data in rx {
                if writer
                    .write_all(&data)
                    .and_then(|()| writer.flush())
                    .is_err()
                {
                    break;
                }
            }
        });
        Ok(Spawned {
            io: PtyIo {
                pid,
                master: Mutex::new(Some(pair.master)),
                input: Mutex::new(Some(tx)),
                job: Mutex::new(Some(job)),
            },
            reader,
            child,
        })
    }

    impl Io for PtyIo {
        fn write(&self, data: &[u8]) -> Result<()> {
            let input = self.input.lock().unwrap_or_else(PoisonError::into_inner);
            match input.as_ref() {
                Some(tx) => tx.send(data.to_vec()).context("terminal closed"),
                None => anyhow::bail!("terminal closed"),
            }
        }

        fn resize(&self, cols: u16, rows: u16) -> Result<()> {
            let master = self.master.lock().unwrap_or_else(PoisonError::into_inner);
            match master.as_ref() {
                Some(m) => m.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                }),
                None => anyhow::bail!("terminal closed"),
            }
        }

        fn pid(&self) -> Option<u32> {
            self.pid
        }

        fn hang_up(&self) {
            // Only while the program has not been reaped (the session
            // checks `running`): its pid cannot belong to another process.
            if let Some(pid @ 1..) = self.pid.and_then(|p| libc::pid_t::try_from(p).ok()) {
                // SAFETY: a plain signal to one process.
                unsafe { libc::kill(pid, libc::SIGHUP) };
            }
        }

        fn left(&self, leader_running: bool) -> Vec<u32> {
            let Some(leader) = self.pid else {
                return Vec::new();
            };
            // Zombies are not listed (no program path): a job that ended
            // but was not reaped yet does not hold the session up.
            let procs: Vec<(u32, u32)> = crate::os::processes::all()
                .iter()
                .filter_map(|p| {
                    let pid = libc::pid_t::try_from(p.pid).ok()?;
                    // SAFETY: only reads the session id; -1 when it is gone.
                    let sid = unsafe { libc::getsid(pid) };
                    Some((p.pid, u32::try_from(sid).ok()?))
                })
                .collect();
            super::in_session(&procs, leader, leader_running)
        }

        fn kill(&self, pids: &[u32]) {
            crate::os::sessions::stop(pids);
        }

        fn release(&self) {
            self.input
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take();
            self.master
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take();
            // Dropping the job kills what is left of the group.
            self.job
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take();
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/editor/pty.rs"]
mod tests;
