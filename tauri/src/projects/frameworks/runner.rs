//! Running the lines of a creation, one creation at a time, cancellable.

use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
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

/// Runs one line with the kit's PowerShell in `dir`, without a window or
/// input (`CI=1`: the tools do not ask), with `vars` in its environment
/// (`$env:…`, so a password is not in the line shown). Every line of
/// output goes to `out`. A failure returns its last line of output.
pub fn run_line(
    kit: &Kit,
    env: &KitEnv,
    vars: &[(String, String)],
    dir: &Path,
    line: &str,
    out: &(dyn Fn(&str) + Sync),
) -> Result<(), String> {
    use base64::Engine;
    use std::io::Read;
    let script = format!(
        "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue'; \
         [Console]::OutputEncoding = [Text.Encoding]::UTF8\n{line}\n\
         if ($LASTEXITCODE) {{ exit $LASTEXITCODE }}"
    );
    let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut cmd = jobs::kit_command(crate::setup::kit_path(kit, r"pwsh\pwsh.exe"), env);
    cmd.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-EncodedCommand",
    ])
    .arg(base64::engine::general_purpose::STANDARD.encode(utf16))
    .current_dir(dir)
    .env("CI", "1")
    .env("NO_COLOR", "1")
    .env("FORCE_COLOR", "0")
    .env("COMPOSER_NO_INTERACTION", "1")
    .envs(vars.iter().map(|(k, v)| (k, v)))
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    let job = Job::new().map_err(|e| format!("{e:#}"))?;
    let mut child = jobs::spawn_in(&mut cmd, &job).map_err(|e| format!("{e:#}"))?;
    *current() = Some(job);
    if cancelled() {
        cancel();
    }
    let last = Mutex::new(String::new());
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
                        *last
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = l;
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
        Ok(_) => Err(last),
        Err(e) => Err(e.to_string()),
    }
}
