//! Final check of the installed components (port of nabre/atollo
//! `Test-Installation`, `_prepare-finish.ps1:157-217`): each program answers
//! its version command, with the kit environment and a time limit.

use std::io::Read;
use std::process::Stdio;
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
    let mut out = String::new();
    if let Some(mut s) = child.stdout.take() {
        let _ = s.read_to_string(&mut out);
    }
    if let Some(mut s) = child.stderr.take() {
        let _ = s.read_to_string(&mut out);
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

#[cfg(test)]
#[path = "../../../test/rust/setup/checkup.rs"]
mod tests;
