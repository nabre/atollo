//! Project-local environments (D-188): where a tool has a standard one,
//! the project's dependencies live in the project's folder, not in the
//! kit, one folder per system where its contents are native (Windows' and
//! macOS' builds do not run in the VM's Linux).
//!
//! | Tool   | On the PC                        | In the agents' sandbox               |
//! | ------ | -------------------------------- | ------------------------------------ |
//! | Python | `.venv` (made by Atollo)         | `.venv-linux` (made at the session)  |
//! | Ruby   | `BUNDLE_PATH=vendor/bundle`      | `BUNDLE_PATH=vendor/bundle-linux`    |
//! | Rust   | `target/` (Cargo's own)          | `CARGO_TARGET_DIR=target/linux`      |
//! | Julia  | `JULIA_PROJECT=@.`               | `JULIA_PROJECT=@.`                   |
//! | R      | renv, its cache in the kit       | renv, its cache in the VM's home     |
//!
//! Node's `node_modules` stays one folder: npm has no variable for it, and
//! the VM sees the project's own. A native addon built on the PC needs
//! `npm rebuild` (or a new install) in the sandbox before an agent runs it
//! there. PHP's `vendor/` is the same on every system. The other tools
//! keep their caches in the kit (`env::langs`).
//!
//! The markers are read in the project's folder itself: an environment is
//! the root's. Only Python's is made by Atollo, in the background
//! ([`ensure_pc`]), empty: the dependencies stay the user's action (`uv
//! sync`, `pip install`, `bundle install`), since installing them runs the
//! packages' own code, which opening a project never does.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::Serialize;

use crate::env::KitEnv;
use crate::kit::Kit;

/// Python's environment on the PC.
pub const VENV: &str = ".venv";
/// Python's environment in the VM's Linux.
pub const VENV_LINUX: &str = ".venv-linux";
/// What a session in the VM reads to make and use [`VENV_LINUX`]
/// (`isolation::vm::LOCAL_ENV`).
pub const VENV_VAR: &str = "ATOLLO_VENV";

/// The files that make the project's folder a Python project.
const PYTHON: [&str; 6] = [
    "pyproject.toml",
    "requirements.txt",
    "uv.lock",
    "setup.py",
    "Pipfile",
    ".python-version",
];

/// The tools with a local environment found in `dir` itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Python,
    Ruby,
    Rust,
    Julia,
    R,
}

/// The tools of `dir` with a local environment, from its own files.
pub fn tools(dir: &Path) -> Vec<Tool> {
    let has = |name: &str| dir.join(name).is_file();
    let mut out = Vec::new();
    if PYTHON.iter().any(|n| has(n)) {
        out.push(Tool::Python);
    }
    if has("Gemfile") {
        out.push(Tool::Ruby);
    }
    if has("Cargo.toml") {
        out.push(Tool::Rust);
    }
    if has("Project.toml") || has("JuliaProject.toml") {
        out.push(Tool::Julia);
    }
    if has("renv.lock") {
        out.push(Tool::R);
    }
    out
}

/// The folder of `venv`'s programs: `Scripts` on Windows, `bin` elsewhere.
fn venv_bin(venv: &Path, windows: bool) -> PathBuf {
    venv.join(if windows { "Scripts" } else { "bin" })
}

/// A Python environment is there: its `pyvenv.cfg`.
fn venv_ready(venv: &Path) -> bool {
    venv.join("pyvenv.cfg").is_file()
}

/// The PC's side of `dir`'s local environments in `env` (built for `kit`'s
/// target): `.venv`'s programs first on the `PATH` and `VIRTUAL_ENV`, only
/// when it exists; Ruby's, Julia's and R's variables. Applied again it
/// changes nothing.
pub fn apply_pc(kit: &Kit, dir: &Path, env: &mut KitEnv) {
    let windows = kit.target().is_windows();
    for tool in tools(dir) {
        match tool {
            Tool::Python => {
                let venv = dir.join(VENV);
                if venv_ready(&venv) {
                    let bin = venv_bin(&venv, windows);
                    crate::editor::terminals::strip(env, std::slice::from_ref(&bin));
                    let rest = env.get("PATH").unwrap_or_default().to_owned();
                    let first = bin.display().to_string();
                    let path = if rest.is_empty() {
                        first
                    } else {
                        format!("{first}{}{rest}", env.path_sep())
                    };
                    env.set("PATH", path);
                    env.set("VIRTUAL_ENV", venv.display().to_string());
                }
            }
            Tool::Ruby => env.set(
                "BUNDLE_PATH",
                dir.join("vendor").join("bundle").display().to_string(),
            ),
            Tool::Rust => {}
            Tool::Julia => env.set("JULIA_PROJECT", "@."),
            Tool::R => env.set(
                "RENV_PATHS_CACHE",
                kit.data().join("cache").join("renv").display().to_string(),
            ),
        }
    }
}

/// The VM's side of `dir`'s local environments, its folder at `guest` in
/// the VM (`vm::guest_path`): the variables only, every path under the
/// project's own folder, never another of the PC. Python's `.venv-linux`
/// is made and put on the `PATH` by the session itself ([`VENV_VAR`]).
pub fn vm_vars(dir: &Path, guest: &str) -> Vec<(String, String)> {
    let under = |rel: &str| format!("{}/{rel}", guest.trim_end_matches('/'));
    let mut out = Vec::new();
    for tool in tools(dir) {
        match tool {
            Tool::Python => {
                out.push(("UV_PROJECT_ENVIRONMENT".into(), under(VENV_LINUX)));
                out.push((VENV_VAR.into(), under(VENV_LINUX)));
            }
            Tool::Ruby => out.push(("BUNDLE_PATH".into(), under("vendor/bundle-linux"))),
            Tool::Rust => out.push(("CARGO_TARGET_DIR".into(), under("target/linux"))),
            Tool::Julia => out.push(("JULIA_PROJECT".into(), "@.".into())),
            // renv keeps its cache in the VM's home.
            Tool::R => {}
        }
    }
    out
}

/// Where a project's Python environment is, as its page shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum LocalEnv {
    /// `.venv` is there.
    Ready,
    /// Not made yet (Atollo makes it in the background).
    Missing,
    /// The last try failed: `reason`, the last line of its output.
    Failed { reason: String },
}

/// The last failure of [`ensure_pc`] per project folder, in this run.
static FAILED: Mutex<Option<HashMap<PathBuf, String>>> = Mutex::new(None);

fn note_failure(dir: &Path, reason: Option<String>) {
    let mut failed = FAILED.lock().unwrap_or_else(PoisonError::into_inner);
    let map = failed.get_or_insert_with(HashMap::new);
    match reason {
        Some(r) => {
            map.insert(dir.to_path_buf(), r);
        }
        None => {
            map.remove(dir);
        }
    }
}

/// The state of `dir`'s Python environment; `None` for a folder that is
/// not a Python project.
pub fn status(dir: &Path) -> Option<LocalEnv> {
    if !tools(dir).contains(&Tool::Python) {
        return None;
    }
    if venv_ready(&dir.join(VENV)) {
        return Some(LocalEnv::Ready);
    }
    let failed = FAILED.lock().unwrap_or_else(PoisonError::into_inner);
    Some(match failed.as_ref().and_then(|m| m.get(dir)) {
        Some(reason) => LocalEnv::Failed {
            reason: reason.clone(),
        },
        None => LocalEnv::Missing,
    })
}

/// The first program `name` (`uv`, `python`) on `env`'s `PATH`.
fn which(env: &KitEnv, name: &str, windows: bool) -> Option<PathBuf> {
    let file = if windows {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    env.path_entries()
        .into_iter()
        .map(|dir| Path::new(dir).join(&file))
        .find(|p| p.is_file())
}

/// What makes an empty `.venv` with `env` (the project's environment on
/// the PC, at its Python line): `uv venv`, else Python's own `venv`; `None`
/// without either program. Nothing of the project runs.
pub fn create_command(env: &KitEnv, windows: bool) -> Option<(PathBuf, Vec<String>)> {
    if let Some(uv) = which(env, "uv", windows) {
        return Some((uv, vec!["venv".into(), VENV.into()]));
    }
    let python = which(env, "python", windows).or_else(|| which(env, "python3", windows))?;
    Some((python, vec!["-m".into(), "venv".into(), VENV.into()]))
}

/// How [`ensure_pc`] runs a program: its path, its words, its folder, its
/// environment; its error text when it fails.
pub type Run<'a> = dyn Fn(&Path, &[String], &Path, &KitEnv) -> Result<(), String> + 'a;

/// Makes `dir`'s `.venv` when it is a Python project without one, with
/// `run` (the program, its words, the folder; its error text): once made,
/// nothing runs again. A failure is kept for [`status`] and goes to the
/// log; the project works without it. `.venv` goes in `.gitignore`.
pub fn ensure_pc(kit: &Kit, dir: &Path, env: &KitEnv, run: &Run<'_>) -> Option<LocalEnv> {
    if !tools(dir).contains(&Tool::Python) {
        return None;
    }
    if !venv_ready(&dir.join(VENV)) {
        let Some((program, args)) = create_command(env, kit.target().is_windows()) else {
            let reason = "uv or Python not installed".to_owned();
            note_failure(dir, Some(reason.clone()));
            return Some(LocalEnv::Failed { reason });
        };
        if let Err(reason) = run(&program, &args, dir, env) {
            crate::common::log::warn(&format!("{}: {VENV}: {reason}", dir.display()));
            note_failure(dir, Some(reason.clone()));
            return Some(LocalEnv::Failed { reason });
        }
        note_failure(dir, None);
    }
    if let Err(e) = ignore(dir, &[VENV, VENV_LINUX]) {
        crate::common::log::warn(&format!("{}: .gitignore: {e:#}", dir.display()));
    }
    status(dir)
}

/// [`ensure_pc`] with the kit's programs, without a window, to their end.
pub fn ensure_pc_now(kit: &Kit, dir: &Path, env: &KitEnv) -> Option<LocalEnv> {
    ensure_pc(kit, dir, env, &|program, args, at, env| {
        let out = crate::platform::jobs::kit_command(program, env)
            .args(args)
            .current_dir(at)
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            return Ok(());
        }
        let text = String::from_utf8_lossy(&out.stderr);
        let last = text.lines().rev().find(|l| !l.trim().is_empty());
        Err(last.map_or_else(
            || format!("exit code {}", out.status.code().unwrap_or(-1)),
            str::to_owned,
        ))
    })
}

/// `names` in the `.gitignore` of `dir` when it is a Git repository and
/// they are not listed there yet (as `/<name>/`).
pub fn ignore(dir: &Path, names: &[&str]) -> anyhow::Result<()> {
    let path = dir.join(".gitignore");
    if !dir.join(".git").exists() || crate::platform::safe_path::is_link(&path) {
        return Ok(());
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let listed = |name: &str| {
        text.lines().any(|l| {
            let l = l.trim().trim_start_matches('/').trim_end_matches('/');
            l == name
        })
    };
    let missing: Vec<&&str> = names.iter().filter(|n| !listed(n)).collect();
    if missing.is_empty() {
        return Ok(());
    }
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = text.clone();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push_str(newline);
    }
    for name in missing {
        out.push_str(&format!("/{name}/{newline}"));
    }
    crate::common::fsx::write_atomic(&path, out.as_bytes())?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/projects/envs.rs"]
mod tests;
