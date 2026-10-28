//! Running the lines of a creation, one creation at a time, cancellable.

use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::platform::jobs::{self, Job};
use crate::projects::run::{split_lines, take_utf8};

/// The job of the command running now: dropping it ends the whole tree.
static CURRENT: Mutex<Option<Job>> = Mutex::new(None);
static CANCELLED: AtomicBool = AtomicBool::new(false);
static BUSY: AtomicBool = AtomicBool::new(false);

/// One creation at a time: held while it runs.
pub struct Busy;

impl Drop for Busy {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::SeqCst);
    }
}

pub fn begin() -> Option<Busy> {
    BUSY.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .ok()?;
    CANCELLED.store(false, Ordering::SeqCst);
    Some(Busy)
}

fn current() -> std::sync::MutexGuard<'static, Option<Job>> {
    CURRENT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Stops the creation: the command running now ends, no other starts.
pub fn cancel() {
    CANCELLED.store(true, Ordering::SeqCst);
    drop(current().take());
}

pub fn cancelled() -> bool {
    CANCELLED.load(Ordering::SeqCst)
}

/// What every line of a creation gets: no question asked (`CI=1`), no
/// colours in its output.
const QUIET: [(&str, &str); 4] = [
    ("CI", "1"),
    ("NO_COLOR", "1"),
    ("FORCE_COLOR", "0"),
    ("COMPOSER_NO_INTERACTION", "1"),
];

/// `line` as the kit's PowerShell runs it with `-EncodedCommand` (UTF-16,
/// base64): errors stop it, its exit code is the line's.
fn encoded_script(line: &str) -> String {
    use base64::Engine;
    let script = format!(
        "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue'; \
         [Console]::OutputEncoding = [Text.Encoding]::UTF8\n{line}\n\
         if ($LASTEXITCODE) {{ exit $LASTEXITCODE }}"
    );
    let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64::engine::general_purpose::STANDARD.encode(utf16)
}

/// One line with the kit's PowerShell on the PC in `dir`, without a window
/// or input ([`QUIET`]), with `vars` in its environment (`$env:…`, so a
/// password is not in the line shown). Built only by
/// `levels::start::scaffold_command` (D-179); run by [`run_command`].
pub fn line_command(
    kit: &Kit,
    env: &KitEnv,
    vars: &[(String, String)],
    dir: &Path,
    line: &str,
) -> std::process::Command {
    let mut cmd = jobs::kit_command(crate::setup::kit_path(kit, r"pwsh\pwsh.exe"), env);
    cmd.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-EncodedCommand",
    ])
    .arg(encoded_script(line))
    .current_dir(dir)
    .envs(QUIET)
    .envs(vars.iter().map(|(k, v)| (k, v)));
    cmd
}

/// The lines of output a failure returns: a tool that runs others
/// (Composer's scripts, Laravel's `install:features`) ends with a generic
/// line, the cause is above it.
const TAIL: usize = 12;

/// Runs `cmd` (a line of a creation: `levels::start::scaffold_command`) in
/// a job of its own, without input. Every line of output goes to `out`. A
/// failure returns its last [`TAIL`] lines of output.
pub fn run_command(
    mut cmd: std::process::Command,
    out: &(dyn Fn(&str) + Sync),
) -> Result<(), String> {
    use std::io::Read;
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let job = Job::new().map_err(|e| format!("{e:#}"))?;
    let mut child = jobs::spawn_in(&mut cmd, &job).map_err(|e| format!("{e:#}"))?;
    *current() = Some(job);
    if cancelled() {
        cancel();
    }
    let last = Mutex::new(std::collections::VecDeque::<String>::new());
    let pipes: [Option<Box<dyn Read + Send>>; 2] = [
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    ];
    std::thread::scope(|s| {
        for mut pipe in pipes.into_iter().flatten() {
            let last = &last;
            s.spawn(move || {
                let (mut rest, mut buf) = (String::new(), [0u8; 8192]);
                let mut pending = Vec::new();
                let mut emit = |l: String| {
                    if !l.trim().is_empty() {
                        out(&l);
                        let mut last = last
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        if last.len() == TAIL {
                            last.pop_front();
                        }
                        last.push_back(l);
                    }
                };
                // A character split across two reads waits for the
                // second one (BUG-V7-13).
                while let Ok(n @ 1..) = pipe.read(&mut buf) {
                    pending.extend_from_slice(&buf[..n]);
                    let chunk = take_utf8(&mut pending);
                    split_lines(&mut rest, &chunk)
                        .into_iter()
                        .for_each(&mut emit);
                }
                let tail = String::from_utf8_lossy(&pending).into_owned() + "\n";
                split_lines(&mut rest, &tail)
                    .into_iter()
                    .for_each(&mut emit);
            });
        }
    });
    let status = child.wait();
    drop(current().take());
    let last = last
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) if last.is_empty() => Err(format!("exit code {}", s.code().unwrap_or(-1))),
        Ok(_) => Err(Vec::from(last).join("\n")),
        Err(e) => Err(e.to_string()),
    }
}
