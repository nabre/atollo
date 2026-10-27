//! The tools sandbox (D-180): what an installer for the VM's Linux runs
//! (rustup, micromamba, npm, sdkmanager, tlmgr, `tar` for an archive of
//! symbolic links) runs there when this PC cannot run Linux programs
//! itself. One sandbox per kit, `atollo-k-tools`, of the kit's `sbx`:
//!
//! - its workspace is the kit's `data/apps/<linux>` (the VM's Linux,
//!   `vm::guest_target`), mounted writable, and `data/npm-global` too
//!   (pnpm and yarn, D-180); the projects' VMs mount both read-only;
//! - it runs the kit's installers only, never a project's code: no
//!   database port, no GitHub, no agent;
//! - its network is the kit's policy (`balanced`) and, for it alone, the
//!   hosts of [`HOSTS`];
//! - it starts at the first program an installation runs there and stops
//!   when the run is over ([`Guest`] dropped); its disk keeps only caches.
//!
//! The words of a program are the PC's: a path under a mounted folder
//! becomes the VM's (`vm::guest_path`), the rest stays as it is. The work
//! folders of an installation go into the workspace ([`Guest::work_root`]):
//! the PC's temporary folder is not in the VM.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{anyhow, Result};

use crate::env::{Host, KitEnv};
use crate::isolation::{self, vm, Vm};
use crate::kit::{Kit, Target};

/// The tools sandbox's name.
pub const NAME: &str = "atollo-k-tools";

/// The hosts the installers reach from the tools sandbox beyond the kit's
/// policy (`balanced` refuses them, tried with sbx 0.50.0): conda-forge,
/// for Ruby, R, C and C++ (make), MongoDB and SQLite on Linux.
pub const HOSTS: [&str; 1] = ["conda.anaconda.org"];

/// The folder of the work folders in the workspace, out of every
/// component's own.
pub const WORK: &str = ".work";

/// The tools sandbox of a run that prepares the VM's Linux from a PC that
/// cannot run it.
pub struct Guest {
    vm: Vm,
    /// The PC's kit, host and environment `sbx` runs with.
    kit: Kit,
    host: Host,
    env: KitEnv,
    windows: bool,
    /// The folders of the PC the sandbox mounts, writable.
    roots: Vec<PathBuf>,
    started: OnceLock<std::result::Result<(), String>>,
}

impl Guest {
    /// The tools sandbox for preparing `target` from the PC of `kit` (the
    /// PC's kit), when `target` is the VM's Linux, this PC cannot run it
    /// and the VM is on; `None` otherwise (its components are then staged
    /// for a first start on Linux, `router::bootstrap`).
    pub fn for_target(kit: &Kit, host: &Host, env: &KitEnv, target: Target) -> Option<Guest> {
        if !needed_for(kit.target(), target) {
            return None;
        }
        let status = isolation::status(kit, host, env, false);
        if status.level != isolation::Level::Vm {
            crate::common::log::info(&format!(
                "{NAME}: no VM ({:?}): what runs Linux programs is staged",
                status.blockers
            ));
            return None;
        }
        let guest = kit.for_target(target);
        let roots = mounts(&guest);
        let vm = isolation::tools_vm(kit, host, &roots[0], roots[1..].to_vec())?;
        Some(Guest {
            vm,
            kit: kit.clone(),
            host: host.clone(),
            env: env.clone(),
            windows: kit.target().is_windows(),
            roots,
            started: OnceLock::new(),
        })
    }

    /// Where the work folders of an installation go: in the workspace,
    /// which the VM sees.
    pub fn work_root(&self) -> PathBuf {
        self.roots[0].join(WORK)
    }

    /// `program` with `args`, `envs` and in `dir` when given, run in the
    /// sandbox (`sbx exec`, `-i` when the caller writes to its standard
    /// input): the paths in them as the VM sees them. The sandbox starts
    /// the first time.
    pub fn command(
        &self,
        program: &Path,
        args: &[String],
        envs: &[(String, String)],
        dir: Option<&Path>,
        input: bool,
    ) -> Result<Command> {
        self.start()?;
        let shown = program.display().to_string();
        // A program of the kit, or one of the VM's image (`/usr/bin/tar`).
        let program = self
            .path(&shown)
            .or_else(|| shown.starts_with('/').then(|| shown.clone()))
            .ok_or_else(|| anyhow!("{shown}: not in the tools sandbox"))?;
        let mut words: Vec<String> = vec!["exec".into()];
        if input {
            words.push("-i".into());
        }
        if let Some(dir) = dir {
            let shown = dir.display().to_string();
            let inside = self
                .path(&shown)
                .ok_or_else(|| anyhow!("{shown}: not in the tools sandbox"))?;
            words.extend(["-w".into(), inside]);
        }
        for (k, v) in envs {
            words.push("-e".into());
            words.push(format!("{k}={}", self.word(v)));
        }
        words.push(self.vm.name.clone());
        words.push(program);
        words.extend(args.iter().map(|a| self.word(a)));
        Ok(self.vm.command(&self.env, &words))
    }

    /// Starts the sandbox once ([`Vm::start`]), then lets it reach
    /// [`HOSTS`]; the error of the first try for every later one.
    fn start(&self) -> Result<()> {
        self.started
            .get_or_init(|| {
                let env = &self.env;
                self.vm
                    .start(env, "en", &|| {
                        crate::common::log::info(&format!("{NAME}: made, its image pulled"));
                    })
                    .and_then(|()| self.vm.allow_hosts(env, &HOSTS))
                    .map_err(|e| format!("{e:#}"))
            })
            .clone()
            .map_err(|e| anyhow!("{NAME}: {e}"))
    }

    /// `path` of the PC as the VM sees it, when it is in a mounted folder.
    fn path(&self, path: &str) -> Option<String> {
        guest_word(self.windows, &self.roots, path)
    }

    /// One word for the VM: a path of a mounted folder translated, a
    /// `--name=<path>` too; the rest as it is.
    fn word(&self, word: &str) -> String {
        if let Some(inside) = self.path(word) {
            return inside;
        }
        if let Some((name, value)) = word.split_once('=') {
            if let Some(inside) = self.path(value) {
                return format!("{name}={inside}");
            }
        }
        word.to_owned()
    }
}

impl Drop for Guest {
    /// Stops the sandbox it started: nothing runs there between two runs.
    fn drop(&mut self) {
        if matches!(self.started.get(), Some(Ok(()))) {
            if let Err(e) = isolation::stop_sandbox(&self.kit, &self.host, &self.env, NAME) {
                crate::common::log::warn(&format!("{NAME}: stop: {e:#}"));
            }
        }
    }
}

/// True when preparing `target` from a PC of `host` needs the tools
/// sandbox: `target` is the VM's Linux there, and the PC cannot run it.
pub fn needed_for(host: Target, target: Target) -> bool {
    vm::guest_target(host) == Some(target)
        && host != target
        && !crate::router::target::can_execute_on_host(target)
}

/// The folders of `guest` (the kit scoped to the VM's Linux) the tools
/// sandbox mounts writable, the workspace first: its `data/apps/<linux>`,
/// then `data/npm-global` (pnpm and yarn, whose Linux layout, `bin` and
/// `lib`, sits beside the PC's).
pub fn mounts(guest: &Kit) -> Vec<PathBuf> {
    vec![guest.apps(), guest.data().join("npm-global")]
}

/// `word` as the VM sees it when it is a path under one of `roots` (as
/// `vm::guest_path` puts them); `None` otherwise.
pub fn guest_word(windows: bool, roots: &[PathBuf], word: &str) -> Option<String> {
    if !windows {
        let path = Path::new(word);
        return roots
            .iter()
            .any(|root| path.starts_with(root))
            .then(|| vm::guest_path(false, path))
            .flatten();
    }
    // Windows paths: `\` or `/`, any case, whatever system reads them.
    let key = |text: &str| text.replace('/', "\\").to_lowercase();
    let path = key(word);
    let under = roots.iter().any(|root| {
        let root = key(&root.display().to_string());
        let root = root.trim_end_matches('\\');
        path == root || path.starts_with(&format!("{root}\\"))
    });
    under
        .then(|| vm::guest_path(true, Path::new(&word.replace('/', "\\"))))
        .flatten()
}

#[cfg(test)]
#[path = "../../../test/rust/install/guest.rs"]
mod tests;
