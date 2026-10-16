//! The frameworks of the new-project wizard (D-83): bases (Laravel, Vite,
//! Electron…) and the additions combined with them (React, Tailwind CSS,
//! NativePHP…), each with the commands that install it.
//!
//! The official instructions are `catalog/frameworks.json`, embedded in the
//! exe. The ones changed or added in Settings are kept in
//! `data\app\frameworks.json`: a piece there replaces the official one with
//! the same id, a new id is added. A file that cannot be read is never
//! overwritten, as for the settings.
//!
//! Here the pieces and their loading and saving; the checks in `validate`,
//! the combinations in `plan`, the project's folder in `target`, running
//! the lines in `runner`.

mod plan;
mod runner;
mod target;
mod validate;

pub use plan::{env_raw, env_value, fill, password_of, plan, Plan};
pub use runner::{begin, cancel, cancelled, run_line};
pub use target::{existing, merge_into, targets, Existing, Target};
pub use validate::{check_env, validate, Problem};

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::fsx::write_atomic;
use crate::kit::Kit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Base,
    Addon,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub command: String,
}

/// A line of `.env`; `{name}` and `{package}` are filled in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

/// What an addition does with one base.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Variant {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
    /// Replaces the base's creation (a starter kit, another template).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub create: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
    /// A server of the kit (`mariadb`, `mongodb`): the project gets its
    /// database there and the connection in `.env`, before the steps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    /// Variables written in the project's `.env`, before the steps.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    pub id: String,
    pub kind: Kind,
    pub name: String,
    /// Italian; translated when shown.
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub home: String,
    /// Additions of the same slot do not go together (one frontend…).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
    /// Run in `projects\`: makes the folder `{name}`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub create: Vec<String>,
    /// Run inside the project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
    /// Variables written in the project's `.env`, before the steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVar>,
    /// Of a base that cannot do without one (a CMS): the kit's server
    /// where the project gets its database, as an addition's `database`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Of an addition: what it does with each base, by base id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub on: BTreeMap<String, Variant>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct File {
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    about: String,
    pieces: Vec<Piece>,
}

/// The official pieces, in the catalog's order.
pub fn official() -> &'static [Piece] {
    static PIECES: OnceLock<Vec<Piece>> = OnceLock::new();
    PIECES.get_or_init(|| {
        // A broken catalog fails the tests; at run time the list is empty.
        serde_json::from_str::<File>(include_str!("../../../catalog/frameworks.json"))
            .map(|f| f.pieces)
            .unwrap_or_default()
    })
}

fn user_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("frameworks.json")
}

/// The pieces in use plus the reason the user's file could not be read.
pub struct Loaded {
    pub pieces: Vec<Piece>,
    pub problem: Option<String>,
}

/// Official pieces with the user's changes, then the user's own pieces.
pub fn load(kit: &Kit) -> Loaded {
    let path = user_file(kit);
    let (own, problem) = crate::fsx::load_json::<File>(&path);
    Loaded {
        pieces: merge(official(), own.pieces),
        problem,
    }
}

fn merge(official: &[Piece], mut own: Vec<Piece>) -> Vec<Piece> {
    let mut out: Vec<Piece> = official
        .iter()
        .map(|p| match own.iter().position(|o| o.id == p.id) {
            Some(i) => own.remove(i),
            None => p.clone(),
        })
        .collect();
    out.extend(own);
    out
}

fn clean_lines(lines: &mut Vec<String>) {
    for l in lines.iter_mut() {
        *l = l.trim().to_owned();
    }
    lines.retain(|l| !l.is_empty());
}

fn clean_actions(actions: &mut Vec<Action>) {
    for a in actions.iter_mut() {
        a.name = a.name.trim().to_owned();
        a.command = a.command.trim().to_owned();
    }
    actions.retain(|a| !a.name.is_empty() || !a.command.is_empty());
}

fn clean_env(env: &mut Vec<EnvVar>) {
    for v in env.iter_mut() {
        v.key = v.key.trim().to_owned();
        v.value = v.value.trim().to_owned();
    }
    env.retain(|v| !v.key.is_empty() || !v.value.is_empty());
}

fn clean_note(note: &mut Option<String>) {
    if note.as_deref().is_some_and(|n| n.trim().is_empty()) {
        *note = None;
    }
}

/// Trims the texts and drops empty lines, as they are saved.
pub fn clean(p: &mut Piece) {
    p.id = p.id.trim().to_owned();
    p.name = p.name.trim().to_owned();
    p.desc = p.desc.trim().to_owned();
    p.home = p.home.trim().to_owned();
    if p.slot.as_deref().is_some_and(|s| s.trim().is_empty()) {
        p.slot = None;
    }
    clean_lines(&mut p.needs);
    clean_lines(&mut p.create);
    clean_lines(&mut p.steps);
    clean_actions(&mut p.actions);
    clean_env(&mut p.env);
    clean_note(&mut p.database);
    clean_note(&mut p.note);
    for v in p.on.values_mut() {
        clean_lines(&mut v.needs);
        clean_lines(&mut v.create);
        clean_lines(&mut v.steps);
        clean_actions(&mut v.actions);
        clean_env(&mut v.env);
        clean_note(&mut v.note);
        clean_note(&mut v.database);
    }
}

/// Saves what differs from the official instructions: changed official
/// pieces and new ones. An official piece left out stays as it is.
pub fn save(kit: &Kit, mut pieces: Vec<Piece>) -> Result<(), Problem> {
    pieces.iter_mut().for_each(clean);
    let full = merge(official(), pieces);
    validate(&full)?;
    let own: Vec<Piece> = full
        .into_iter()
        .filter(|p| official().iter().all(|o| o != p))
        .collect();
    let path = user_file(kit);
    let result = if own.is_empty() {
        match fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    } else {
        let json = serde_json::to_string_pretty(&File {
            about: String::new(),
            pieces: own,
        })
        .unwrap_or_default();
        fs::create_dir_all(kit.app_dir()).and_then(|()| write_atomic(&path, json.as_bytes()))
    };
    result.map_err(|e| ("Salvataggio non riuscito: {0}", vec![e.to_string()]))
}

#[cfg(test)]
#[path = "../../../../test/rust/projects/frameworks/mod.rs"]
mod tests;
