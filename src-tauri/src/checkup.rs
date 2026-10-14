//! Final check of the installed components (port of nabre/dev-kit
//! `Test-Installation`, `_prepare-finish.ps1:157-217`): each program answers
//! its version command, with the kit environment and a time limit.

use std::io::Read;
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::catalog::{Catalog, Component};
use crate::env::KitEnv;
use crate::jobs;
use crate::kit::Kit;

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

/// Runs the check command of `c`; `None` if it has none or is not installed.
pub fn check(kit: &Kit, env: &KitEnv, c: &Component) -> Option<Check> {
    let (program, args) = c.check.split_first()?;
    if !c.is_installed(kit) {
        return None;
    }
    let exe = kit.apps().join(program.replace('/', "\\"));
    let result = run(env, &exe, args);
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
mod tests {
    use super::*;

    #[test]
    fn every_check_names_an_installed_file() {
        for c in Catalog::get().all() {
            if let Some((program, _)) = c.check.split_first() {
                let top = program.split('/').next().unwrap_or_default();
                let owned = c.folder.as_deref() == Some(top) || top == "tools" || c.id == "kotlin";
                assert!(owned, "{}: check {program} outside its folder", c.id);
            }
        }
    }
}
