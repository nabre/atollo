//! The levels at the app's commands (D-179, LEVELS.md §4.1, step 3 of
//! §12.2): every command that starts a terminal, an action, a site, a pool
//! or a scaffold builds its `Request`, reads the facts, asks
//! `levels::decide`, and then starts on the PC with
//! `levels::env::environment`, keeps its VM path (step 4 moves it into
//! `levels::prepare`), or returns the refusal with its reason and fix.

use std::path::PathBuf;

use crate::editor::terminals::Terminal;
use crate::env::{Host, KitEnv};
use crate::kit::Kit;
use crate::levels::env::{environment, Environment};
use crate::levels::{decide, facts, Facts, Placement, Request};

use super::{CmdError, CmdResult};

/// What a command does after `decide`. One per command, at once taken
/// apart: the size of its `Pc` variant costs nothing.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Placed {
    /// On the PC, in `dir`, with the environment of the placement.
    Pc {
        placement: Placement,
        dir: PathBuf,
        env: KitEnv,
    },
    /// In the VM: the command's VM path, as before (step 4 of §12.2);
    /// `workdir` the folder in the VM.
    Vm { workdir: String },
}

/// `decide(req, facts)`, a refusal as the command's error: its text in
/// `lang`, its reason and its fix (LEVELS.md §4.1).
pub(crate) fn admitted(req: &Request, facts: &Facts, lang: &str) -> CmdResult<Placement> {
    match decide(req, facts) {
        Placement::Refused { reason, fix, .. } => Err(CmdError::refused(reason, fix, lang)),
        placement => Ok(placement),
    }
}

/// The environment of the `Pc` placement `placement`, built from the kit's
/// `base` by `levels::env::environment` (`terminal`: a kit terminal's tool
/// choice); an error for any other placement.
pub(crate) fn pc_env(
    kit: &Kit,
    base: &KitEnv,
    placement: &Placement,
    terminal: Option<&Terminal>,
    lang: &str,
    host: &Host,
) -> CmdResult<KitEnv> {
    match environment(kit, base, placement, terminal, lang, host) {
        Some(Environment::Pc(env)) => Ok(env),
        _ => Err("levels: not a placement on the PC".into()),
    }
}

/// `req` placed with `facts`: [`admitted`], then a `Pc` placement with its
/// environment ([`pc_env`] from `base`), a `Vm` one as it is.
pub(crate) fn placed(
    kit: &Kit,
    base: &KitEnv,
    req: &Request,
    facts: &Facts,
    terminal: Option<&Terminal>,
    lang: &str,
    host: &Host,
) -> CmdResult<Placed> {
    let placement = admitted(req, facts, lang)?;
    match &placement {
        Placement::Vm { workdir, .. } => Ok(Placed::Vm {
            workdir: workdir.clone(),
        }),
        Placement::Pc { dir, .. } => {
            let dir = dir.clone();
            let env = pc_env(kit, base, &placement, terminal, lang, host)?;
            Ok(Placed::Pc {
                placement,
                dir,
                env,
            })
        }
        Placement::Refused { .. } => Err("levels: refused".into()),
    }
}

/// The facts of `req` read now (`levels::facts::facts`: the VM's status at
/// the moment of the request; `env` the kit's environment that asks `sbx`).
pub(crate) fn facts_now(kit: &Kit, host: &Host, env: &KitEnv, req: &Request) -> Facts {
    facts::facts(kit, host, env, req)
}

/// [`placed`] with the facts read now.
pub(crate) fn place_now(
    kit: &Kit,
    base: &KitEnv,
    req: &Request,
    terminal: Option<&Terminal>,
    lang: &str,
    host: &Host,
) -> CmdResult<Placed> {
    let facts = facts_now(kit, host, base, req);
    placed(kit, base, req, &facts, terminal, lang, host)
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/levels.rs"]
mod tests;
