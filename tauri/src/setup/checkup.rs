//! Final check of the installed components: each program answers its
//! version command, with the kit environment and a time limit.

use std::io::Read;
use std::process::Stdio;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::catalog::{kit_path, Catalog, Component};
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub id: String,
    pub name: String,
    /// First line of the answer, or why it failed.
    pub answer: String,
    pub ok: bool,
}

const LIMIT: Duration = Duration::from_secs(60);

/// As the only argument of a check: a PHP script (phpMyAdmin's
/// `Version.php`), compiled by the kit's PHP (`php -l`); the answer is the
/// `@version` of its header or its `const VERSION`.
pub const PHP_SCRIPT: &str = "<php>";

/// As the only argument of a check: a Windows program that must not be run
/// (`OpenConsole.exe` would open a console); the answer is the version in
/// its resources.
pub const FILE_VERSION: &str = "<file-version>";

/// The file version in the resources of the Windows program `exe`.
#[cfg(windows)]
pub fn file_version(exe: &std::path::Path) -> Result<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    };
    if !exe.is_file() {
        bail!("{} not found", exe.display());
    }
    let path: Vec<u16> = exe.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: a NUL-terminated path.
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), None) };
    anyhow::ensure!(size > 0, "{}: no version", exe.display());
    let mut data = vec![0u8; size as usize];
    // SAFETY: a buffer of `size` bytes.
    unsafe { GetFileVersionInfoW(PCWSTR(path.as_ptr()), None, size, data.as_mut_ptr().cast()) }
        .context("GetFileVersionInfoW")?;
    let mut info: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut len = 0u32;
    // SAFETY: `data` holds the version resource read above; `info` points
    // into it while it lives.
    let found = unsafe { VerQueryValueW(data.as_ptr().cast(), w!("\\"), &mut info, &mut len) };
    anyhow::ensure!(
        found.as_bool() && !info.is_null() && len as usize >= size_of::<VS_FIXEDFILEINFO>(),
        "{}: no version",
        exe.display()
    );
    // SAFETY: checked above: a VS_FIXEDFILEINFO inside `data`.
    let fixed = unsafe { &*info.cast::<VS_FIXEDFILEINFO>() };
    let (ms, ls) = (fixed.dwFileVersionMS, fixed.dwFileVersionLS);
    Ok(format!(
        "{}.{}.{}.{}",
        ms >> 16,
        ms & 0xffff,
        ls >> 16,
        ls & 0xffff
    ))
}

#[cfg(not(windows))]
pub fn file_version(exe: &std::path::Path) -> Result<String> {
    bail!("{}: only on Windows", exe.display())
}

/// Runs the check command of `c`; `None` if it has none or nothing of it is
/// on disk. Half installed (a part missing): failed, naming the part.
pub fn check(kit: &Kit, env: &KitEnv, c: &Component) -> Option<Check> {
    if !crate::router::target::can_execute_on_host(kit.target()) {
        return crate::router::probe::check_static(kit, c);
    }
    let (program, args) = c.check.split_first()?;
    if !c.is_present(kit) {
        return None;
    }
    let result = match c.missing_part(kit) {
        Some(part) => Err(anyhow::anyhow!("{} not found", part.display())),
        None if args == [PHP_SCRIPT] => php_script(kit, env, &kit_path(kit, program)),
        None if args == [FILE_VERSION] => file_version(&kit_path(kit, program)),
        None => run(env, &kit_path(kit, program), args),
    };
    Some(Check {
        id: c.id.clone(),
        name: c.name.clone(),
        ok: result.is_ok(),
        answer: result.unwrap_or_else(|e| format!("{e:#}")),
    })
}

/// Every installed component with a check, in catalog order.
pub fn check_all(kit: &Kit, env: &KitEnv) -> Vec<Check> {
    Catalog::get()
        .all()
        .iter()
        .filter_map(|c| check(kit, env, c))
        .collect()
}

fn php_script(kit: &Kit, env: &KitEnv, script: &std::path::Path) -> Result<String> {
    if !script.is_file() {
        bail!("{} not found", script.display());
    }
    run(
        env,
        &kit_path(kit, "php/php.exe"),
        &["-l".to_owned(), script.display().to_string()],
    )?;
    let head = std::fs::read(script)?;
    let head = String::from_utf8_lossy(&head[..head.len().min(2048)]);
    Ok(header_version(&head).unwrap_or("compiled").to_owned())
}

/// `x.y.z` of the `@version x.y.z` line of a PHP header, or of a
/// `const VERSION = 'x.y.z'` line.
fn header_version(head: &str) -> Option<&str> {
    head.lines()
        .find_map(|l| {
            l.split("@version").nth(1).map(str::trim).or_else(|| {
                l.split("const VERSION = '")
                    .nth(1)
                    .and_then(|v| v.split('\'').next())
            })
        })
        .filter(|v| !v.is_empty())
}

fn run(env: &KitEnv, exe: &std::path::Path, args: &[String]) -> Result<String> {
    if !exe.is_file() {
        bail!("{} not found", exe.display());
    }
    let is_batch = exe
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("bat") || e.eq_ignore_ascii_case("cmd"));
    let mut cmd = if is_batch {
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        jobs::batch_command(exe, &words, env)
    } else {
        let mut c = jobs::kit_command(exe, env);
        c.args(args);
        c
    };
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = jobs::spawn(&mut cmd).context("start")?;
    // Both pipes read while it runs: a program that writes more than a
    // pipe holds would otherwise wait for a reader until the limit.
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > LIMIT {
            let _ = child.kill();
            bail!("no answer within {} s", LIMIT.as_secs());
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    // A child it left running may keep a pipe open: wait no longer than
    // the limit for the rest of the output.
    let mut out = String::new();
    for pipe in [stdout, stderr] {
        let left = LIMIT.saturating_sub(start.elapsed());
        if let Ok(bytes) = pipe.recv_timeout(left) {
            out.push_str(&String::from_utf8_lossy(&bytes));
        }
    }
    let first = out
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default()
        .to_owned();
    if !status.success() {
        bail!("exit {status}: {first}");
    }
    if first.is_empty() {
        bail!("empty answer");
    }
    Ok(first)
}

/// Reads `pipe` to its end on a thread; its bytes arrive on the channel.
fn drain(pipe: Option<impl Read + Send + 'static>) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        let _ = tx.send(bytes);
    });
    rx
}

#[cfg(test)]
#[path = "../../../test/rust/setup/checkup.rs"]
mod tests;
