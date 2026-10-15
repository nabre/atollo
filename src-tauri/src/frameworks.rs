//! The frameworks of the new-project wizard (D-83): bases (Laravel, Vite,
//! Electron…) and the additions combined with them (React, Tailwind CSS,
//! NativePHP…), each with the commands that install it.
//!
//! The official instructions are `catalog/frameworks.json`, embedded in the
//! exe. The ones changed or added in Settings are kept in
//! `data\app\frameworks.json`: a piece there replaces the official one with
//! the same id, a new id is added. A file that cannot be read is never
//! overwritten, as for the settings.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use crate::env::KitEnv;
use crate::jobs::{self, Job};
use crate::kit::{write_atomic, Kit};
use crate::run::split_lines;

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
        serde_json::from_str::<File>(include_str!("../catalog/frameworks.json"))
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
    let (own, problem) = match fs::read(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Vec::new(), None),
        Err(e) => (Vec::new(), Some(format!("{}: {e}", path.display()))),
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            match serde_json::from_str::<File>(text.trim_start_matches('\u{feff}')) {
                Ok(f) => (f.pieces, None),
                Err(e) => (Vec::new(), Some(format!("{}: {e}", path.display()))),
            }
        }
    };
    Loaded {
        pieces: merge(official(), own),
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

/// A problem: the Italian source and its argument (translated by the caller).
pub type Problem = (&'static str, Vec<String>);

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn check_commands(lines: &[String]) -> Result<(), Problem> {
    match lines.iter().find(|l| l.chars().any(char::is_control)) {
        Some(l) => Err(("Comando non valido: {0}", vec![l.clone()])),
        None => Ok(()),
    }
}

fn check_actions(actions: &[Action]) -> Result<(), Problem> {
    for a in actions {
        if a.name.is_empty() {
            return Err(("Azione senza nome: {0}", vec![a.command.clone()]));
        }
        if a.command.is_empty() {
            return Err(("Azione senza comando: {0}", vec![a.name.clone()]));
        }
        if a.name.len() > 60 || a.name.chars().any(char::is_control) {
            return Err(("Valore non valido: {0}", vec![a.name.clone()]));
        }
        check_commands(std::slice::from_ref(&a.command))?;
    }
    Ok(())
}

fn valid_env_key(key: &str) -> bool {
    key.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && key.len() <= 80
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Variables of `.env`: a name of letters, digits and `_`, a value on one line.
pub fn check_env(env: &[EnvVar]) -> Result<(), Problem> {
    for v in env {
        if !valid_env_key(&v.key) {
            return Err(("Variabile non valida: {0}", vec![v.key.clone()]));
        }
        if v.value.chars().any(char::is_control) {
            return Err(("Valore non valido: {0}", vec![v.key.clone()]));
        }
    }
    Ok(())
}

fn check_needs(needs: &[String]) -> Result<(), Problem> {
    let catalog = crate::catalog::Catalog::get();
    match needs.iter().find(|n| catalog.find(n).is_none()) {
        Some(n) => Err(("Strumento del kit sconosciuto: {0}", vec![n.clone()])),
        None => Ok(()),
    }
}

/// Checks a whole list (cleaned) as it would be saved.
pub fn validate(pieces: &[Piece]) -> Result<(), Problem> {
    let mut ids = BTreeSet::new();
    for p in pieces {
        if !valid_id(&p.id) {
            return Err(("Identificativo non valido: {0}", vec![p.id.clone()]));
        }
        if !ids.insert(p.id.as_str()) {
            return Err(("Identificativo ripetuto: {0}", vec![p.id.clone()]));
        }
        if p.name.is_empty() || p.name.len() > 60 {
            return Err(("Nome non valido: {0}", vec![p.id.clone()]));
        }
        if !p.home.is_empty() && !p.home.starts_with("https://") {
            return Err(("Indirizzo non valido: {0}", vec![p.home.clone()]));
        }
    }
    let bases: BTreeSet<&str> = pieces
        .iter()
        .filter(|p| p.kind == Kind::Base)
        .map(|p| p.id.as_str())
        .collect();
    for p in pieces {
        check_needs(&p.needs)?;
        check_commands(&p.create)?;
        check_commands(&p.steps)?;
        check_actions(&p.actions)?;
        check_env(&p.env)?;
        match p.kind {
            Kind::Base => {
                if p.create.is_empty() {
                    return Err((
                        "{0}: manca il comando che crea il progetto.",
                        vec![p.name.clone()],
                    ));
                }
            }
            Kind::Addon => {
                if p.on.is_empty() {
                    return Err(("{0}: scegli almeno una base.", vec![p.name.clone()]));
                }
                for (base, v) in &p.on {
                    if !bases.contains(base.as_str()) {
                        return Err(("Base sconosciuta: {0}", vec![base.clone()]));
                    }
                    check_needs(&v.needs)?;
                    if let Some(db) = &v.database {
                        if !crate::services::SERVERS.contains(&db.as_str()) {
                            return Err(("Server di database sconosciuto: {0}", vec![db.clone()]));
                        }
                    }
                    check_commands(&v.create)?;
                    check_commands(&v.steps)?;
                    check_actions(&v.actions)?;
                    check_env(&v.env)?;
                }
            }
        }
    }
    Ok(())
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

/// What a combination does, in order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// Components of the kit to install first.
    pub needs: Vec<String>,
    /// In `projects\`.
    pub create: Vec<String>,
    /// Inside the project.
    pub steps: Vec<String>,
    pub actions: Vec<Action>,
    /// The kit's database server where the project gets its database.
    pub database: Option<String>,
    /// Variables of `.env` (`{name}`, `{package}` not filled in yet).
    pub env: Vec<EnvVar>,
    /// Italian sources of the notes.
    pub notes: Vec<String>,
}

fn add_actions(out: &mut Vec<Action>, more: &[Action]) {
    for a in more {
        match out.iter_mut().find(|o| o.name == a.name) {
            Some(o) => o.command.clone_from(&a.command),
            None => out.push(a.clone()),
        }
    }
}

fn add_needs(out: &mut Vec<String>, more: &[String]) {
    for n in more {
        if !out.contains(n) {
            out.push(n.clone());
        }
    }
}

/// The base with the additions: the creation (the base's, or the one of the
/// addition that replaces it), the base's steps, then those of each
/// addition in the order of the list.
pub fn plan(pieces: &[Piece], base: &str, addons: &[String]) -> Result<Plan, Problem> {
    let b = pieces
        .iter()
        .find(|p| p.id == base && p.kind == Kind::Base)
        .ok_or_else(|| ("Base sconosciuta: {0}", vec![base.to_owned()]))?;
    let mut chosen: Vec<(&Piece, &Variant)> = Vec::new();
    for p in pieces.iter().filter(|p| p.kind == Kind::Addon) {
        if !addons.contains(&p.id) {
            continue;
        }
        let v =
            p.on.get(base)
                .ok_or_else(|| ("{0} non si usa con questa base.", vec![p.name.clone()]))?;
        chosen.push((p, v));
    }
    if let Some(missing) = addons
        .iter()
        .find(|a| chosen.iter().all(|(p, _)| &p.id != *a))
    {
        return Err(("Aggiunta sconosciuta: {0}", vec![missing.clone()]));
    }
    let mut slots: BTreeMap<&str, &str> = BTreeMap::new();
    for (p, _) in &chosen {
        if let Some(slot) = p.slot.as_deref() {
            if let Some(other) = slots.insert(slot, &p.name) {
                return Err((
                    "{0} e {1} non vanno insieme.",
                    vec![other.to_owned(), p.name.clone()],
                ));
            }
        }
    }
    let replacing: Vec<&(&Piece, &Variant)> = chosen
        .iter()
        .filter(|(_, v)| !v.create.is_empty())
        .collect();
    if replacing.len() > 1 {
        return Err((
            "{0} e {1} non vanno insieme.",
            vec![replacing[0].0.name.clone(), replacing[1].0.name.clone()],
        ));
    }
    let mut out = Plan {
        needs: b.needs.clone(),
        create: replacing
            .first()
            .map_or_else(|| b.create.clone(), |(_, v)| v.create.clone()),
        steps: b.steps.clone(),
        actions: b.actions.clone(),
        database: None,
        env: b.env.clone(),
        notes: b.note.iter().cloned().collect(),
    };
    for (p, v) in &chosen {
        if let Some(db) = &v.database {
            if let Some(other) = &out.database {
                if other != db {
                    return Err((
                        "{0} e {1} non vanno insieme.",
                        vec![other.clone(), p.name.clone()],
                    ));
                }
            }
            out.database = Some(db.clone());
            add_needs(&mut out.needs, std::slice::from_ref(db));
        }
        add_needs(&mut out.needs, &v.needs);
        for e in &v.env {
            match out.env.iter_mut().find(|o| o.key == e.key) {
                Some(o) => o.value.clone_from(&e.value),
                None => out.env.push(e.clone()),
            }
        }
        out.steps.extend(v.steps.iter().cloned());
        add_actions(&mut out.actions, &v.actions);
        out.notes.extend(v.note.iter().cloned());
    }
    Ok(out)
}

/// `{name}` and `{package}` of a project: the package id is
/// `com.example.` with the name's letters and digits in lower case.
pub fn package_of(name: &str) -> String {
    let mut id: String = name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if id.is_empty() || id.starts_with(|c: char| c.is_ascii_digit()) {
        id.insert(0, 'a');
    }
    format!("com.example.{id}")
}

/// What is already in `projects\<name>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Existing {
    /// No folder: the creation makes it.
    New,
    /// An empty folder: the framework goes in it.
    Empty,
    /// Files but no Git: the framework's files are added, the others stay.
    Files,
    /// A Git repository: never used.
    Git,
}

fn existing_at(dir: &Path) -> Existing {
    if !dir.exists() {
        Existing::New
    } else if dir.join(".git").exists() {
        Existing::Git
    } else if fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none()) {
        Existing::Empty
    } else {
        Existing::Files
    }
}

/// A folder of `projects\` and what is in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub state: Existing,
}

/// The projects' folders with their state, for the wizard's name.
pub fn targets(kit: &Kit) -> Vec<Target> {
    crate::projects::folders(kit)
        .iter()
        .filter_map(|d| {
            Some(Target {
                name: d.file_name()?.to_string_lossy().into_owned(),
                state: existing_at(d),
            })
        })
        .collect()
}

/// The state of `projects\<name>`; a Git repository or a link is refused.
pub fn existing(kit: &Kit, name: &str) -> Result<Existing, Problem> {
    crate::kit::validate_project_name(name)
        .map_err(|e| ("Nome non valido: {0}", vec![e.to_string()]))?;
    let dir = kit.projects().join(name);
    if dir.exists() && !crate::safe_path::is_dir(&kit.projects(), &dir) {
        return Err((
            "{0} non è una cartella dei progetti.",
            vec![name.to_owned()],
        ));
    }
    match existing_at(&dir) {
        Existing::Git => Err((
            "{0} è già un repository Git: scegli un altro nome.",
            vec![name.to_owned()],
        )),
        state => Ok(state),
    }
}

/// Moves what is in `from` into `to`: what `to` already has stays as it is
/// (its path relative to `to` goes in `kept`), folders are merged.
pub fn merge_into(from: &Path, to: &Path, kept: &mut Vec<String>) -> std::io::Result<()> {
    merge_at(from, to, Path::new(""), kept)
}

fn merge_at(from: &Path, to: &Path, rel: &Path, kept: &mut Vec<String>) -> std::io::Result<()> {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        let rel = rel.join(entry.file_name());
        match fs::symlink_metadata(&dst) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::rename(&src, &dst)?,
            Err(e) => return Err(e),
            Ok(m)
                if m.is_dir()
                    && !crate::safe_path::is_reparse(&m)
                    && entry.file_type()?.is_dir() =>
            {
                merge_at(&src, &dst, &rel, kept)?;
            }
            Ok(_) => kept.push(rel.to_string_lossy().into_owned()),
        }
    }
    Ok(())
}

/// A line with `{package}` and `{name}` filled in; the name goes in
/// PowerShell quotes, so spaces and `$` stay as they are.
pub fn fill(line: &str, name: &str) -> String {
    let quoted = format!("'{}'", name.replace('\'', "''"));
    line.replace("{package}", &package_of(name))
        .replace("{name}", &quoted)
}

/// A value of `.env` with `{name}` and `{package}` filled in, in double
/// quotes when it has spaces, quotes, `#` or `$`.
pub fn env_value(value: &str, name: &str) -> String {
    let v = value
        .replace("{package}", &package_of(name))
        .replace("{name}", name);
    if v.contains([' ', '"', '#', '\'', '$']) {
        format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        v
    }
}

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
/// input (`CI=1`: the tools do not ask). Every line of output goes to
/// `out`. A failure returns its last line of output.
pub fn run_line(
    kit: &Kit,
    env: &KitEnv,
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
    let mut cmd = jobs::kit_command(kit.apps().join(r"pwsh\pwsh.exe"), env);
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
                let mut emit = |l: String| {
                    if !l.trim().is_empty() {
                        out(&l);
                        *last
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = l;
                    }
                };
                while let Ok(n @ 1..) = pipe.read(&mut buf) {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    split_lines(&mut rest, &chunk)
                        .into_iter()
                        .for_each(&mut emit);
                }
                split_lines(&mut rest, "\n").into_iter().for_each(&mut emit);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn official_catalog_is_valid() {
        let pieces = official();
        assert!(pieces.len() >= 15, "catalog did not load");
        validate(pieces).unwrap();
        for p in pieces {
            let mut c = p.clone();
            clean(&mut c);
            assert_eq!(&c, p, "{}: not clean", p.id);
            assert!(
                !p.desc.is_empty() && p.home.starts_with("https://"),
                "{}",
                p.id
            );
            assert_eq!(p.kind == Kind::Addon, p.slot.is_some(), "{}: slot", p.id);
            if p.kind == Kind::Base {
                assert!(p.on.is_empty(), "{}", p.id);
                assert!(p.create.iter().all(|l| l.contains("{name}")), "{}", p.id);
            }
            for v in p.on.values() {
                assert!(v.create.iter().all(|l| l.contains("{name}")), "{}", p.id);
            }
        }
    }

    #[test]
    fn combinations() {
        let p = official();
        let laravel_react = plan(p, "laravel", &ids(&["react", "tailwind", "nativephp"])).unwrap();
        assert!(laravel_react.create[0].contains("laravel/react-starter-kit"));
        assert_eq!(laravel_react.steps[0], "npm install");
        assert!(laravel_react
            .steps
            .iter()
            .any(|s| s.contains("nativephp/desktop")));
        assert!(laravel_react
            .actions
            .iter()
            .any(|a| a.name == "App desktop"));
        assert!(laravel_react.notes.len() >= 2);

        let electron_vue = plan(p, "electron", &ids(&["vue"])).unwrap();
        assert!(electron_vue.create[0].contains("@quick-start/electron"));
        let start = electron_vue
            .actions
            .iter()
            .find(|a| a.name == "Avvia")
            .unwrap();
        assert_eq!(
            start.command, "npm run dev",
            "the addition replaces the action"
        );
        assert_eq!(electron_vue.actions.len(), 2);

        let tauri = plan(p, "tauri", &[]).unwrap();
        assert_eq!(tauri.needs, ["rust", "node"]);

        let mysql = plan(p, "laravel", &ids(&["mysql", "bootstrap"])).unwrap();
        assert_eq!(mysql.database.as_deref(), Some("mariadb"));
        assert!(mysql.needs.contains(&"mariadb".to_owned()));
        assert!(
            plan(p, "laravel", &ids(&["mysql", "mongodb"])).is_err(),
            "one database"
        );
        for base in ["laravel", "symfony", "vite", "nextjs", "electron", "tauri"] {
            for css in ["tailwind", "bootstrap", "bulma"] {
                plan(p, base, &ids(&[css])).unwrap_or_else(|_| panic!("{css} with {base}"));
            }
        }
        assert!(plan(p, "symfony", &ids(&["vite-bundle"]))
            .unwrap()
            .needs
            .contains(&"node".to_owned()));

        let web_app = plan(p, "vite", &ids(&["react", "tailwind", "capacitor"])).unwrap();
        assert!(web_app.needs.contains(&"android".to_owned()));
        assert!(web_app.steps.iter().any(|s| s.contains("cap add android")));

        let tauri_android = plan(p, "tauri", &ids(&["react", "capacitor"])).unwrap();
        assert!(tauri_android
            .steps
            .iter()
            .any(|s| s.contains("tauri android init")));
        assert!(tauri_android.needs.contains(&"android".to_owned()));

        let mobile = plan(p, "laravel", &ids(&["nativephp-mobile"])).unwrap();
        assert_eq!(mobile.needs, ["php", "node", "java", "android"]);

        assert_eq!(
            plan(p, "laravel", &ids(&["react", "vue"])).unwrap_err().0,
            "{0} e {1} non vanno insieme."
        );
        assert_eq!(
            plan(p, "symfony", &ids(&["react"])).unwrap_err().0,
            "{0} non si usa con questa base."
        );
        assert!(plan(p, "react", &[]).is_err(), "an addition is not a base");
        assert!(plan(p, "vite", &ids(&["nothing"])).is_err());
    }

    #[test]
    fn validation() {
        let base = official().iter().find(|p| p.id == "vite").unwrap().clone();
        let bad = |f: fn(&mut Piece)| {
            let mut list = official().to_vec();
            let mut p = base.clone();
            p.id = "mine".into();
            f(&mut p);
            list.push(p);
            validate(&list).is_err()
        };
        assert!(!bad(|_| ()));
        assert!(bad(|p| p.id = "Mine".into()));
        assert!(bad(|p| p.id = "vite".into()));
        assert!(bad(|p| p.name.clear()));
        assert!(bad(|p| p.create.clear()));
        assert!(bad(|p| p.needs.push("nothing".into())));
        assert!(bad(|p| p.steps.push("a\u{7}b".into())));
        assert!(bad(|p| p.actions[0].name.clear()));
        assert!(bad(|p| p.home = "http://x".into()));
        assert!(bad(|p| {
            p.kind = Kind::Addon;
            p.on.insert("nothing".into(), Variant::default());
        }));
        assert!(bad(|p| p.kind = Kind::Addon));
        assert!(bad(|p| {
            p.kind = Kind::Addon;
            p.on.insert(
                "vite".into(),
                Variant {
                    database: Some("oracle".into()),
                    ..Variant::default()
                },
            );
        }));
    }

    #[test]
    fn saves_only_the_changes() {
        let tmp = TempDir::new("frameworks");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        assert_eq!(load(&kit).pieces, official());

        let mut all = official().to_vec();
        all[0].steps.push("  npm run build ".into());
        all[0].steps.push(String::new());
        let mut mine = all[2].clone();
        mine.id = "mine".into();
        all.push(mine);
        save(&kit, all).unwrap();
        let saved: File =
            serde_json::from_str(&fs::read_to_string(user_file(&kit)).unwrap()).unwrap();
        assert_eq!(
            saved
                .pieces
                .iter()
                .map(|p| p.id.as_str())
                .collect::<Vec<_>>(),
            ["laravel", "mine"]
        );
        let loaded = load(&kit).pieces;
        assert_eq!(loaded[0].steps, ["npm install", "npm run build"]);
        assert_eq!(loaded.len(), official().len() + 1);

        // Back to the official instructions: the file goes away.
        save(&kit, official().to_vec()).unwrap();
        assert!(!user_file(&kit).exists());
        assert!(save(
            &kit,
            vec![Piece {
                name: String::new(),
                ..official()[0].clone()
            }]
        )
        .is_err());
    }

    #[test]
    fn broken_file_is_reported_not_replaced() {
        let tmp = TempDir::new("frameworks-bad");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        fs::write(user_file(&kit), "{ \"pieces\": ").unwrap();
        let loaded = load(&kit);
        assert!(loaded.problem.is_some());
        assert_eq!(loaded.pieces, official());
    }

    #[test]
    fn existing_folders() {
        let tmp = TempDir::new("frameworks-targets");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let p = kit.projects();
        fs::create_dir_all(p.join("empty")).unwrap();
        fs::create_dir_all(p.join("files")).unwrap();
        fs::write(p.join("files").join("notes.txt"), "x").unwrap();
        fs::create_dir_all(p.join("repo").join(".git")).unwrap();
        let states: Vec<(String, Existing)> = targets(&kit)
            .into_iter()
            .map(|t| (t.name, t.state))
            .collect();
        assert_eq!(
            states,
            [
                ("empty".to_owned(), Existing::Empty),
                ("files".to_owned(), Existing::Files),
                ("repo".to_owned(), Existing::Git)
            ]
        );
        assert_eq!(existing(&kit, "new").unwrap(), Existing::New);
        assert_eq!(existing(&kit, "files").unwrap(), Existing::Files);
        assert!(existing(&kit, "repo").is_err());
        assert!(existing(&kit, "..").is_err());
    }

    #[test]
    fn merging_keeps_what_is_there() {
        let tmp = TempDir::new("frameworks-merge");
        let (from, to) = (tmp.0.join("from"), tmp.0.join("to"));
        fs::create_dir_all(from.join("src")).unwrap();
        fs::create_dir_all(to.join("src")).unwrap();
        fs::write(from.join("package.json"), "new").unwrap();
        fs::write(from.join("README.md"), "new").unwrap();
        fs::write(from.join("src").join("main.ts"), "new").unwrap();
        fs::write(from.join("src").join("app.css"), "new").unwrap();
        fs::write(to.join("README.md"), "mine").unwrap();
        fs::write(to.join("src").join("app.css"), "mine").unwrap();
        let mut kept = Vec::new();
        merge_into(&from, &to, &mut kept).unwrap();
        kept.sort();
        assert_eq!(kept, ["README.md", r"src\app.css"]);
        assert_eq!(fs::read_to_string(to.join("README.md")).unwrap(), "mine");
        assert_eq!(fs::read_to_string(to.join("package.json")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(to.join("src").join("main.ts")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(to.join("src").join("app.css")).unwrap(),
            "mine"
        );
    }

    #[test]
    fn env_values() {
        assert_eq!(env_value("{name}", "shop"), "shop");
        assert_eq!(env_value("{name}", "my shop"), "\"my shop\"");
        assert_eq!(env_value("{package}", "my shop"), "com.example.myshop");
        let p = official();
        let plan = plan(p, "laravel", &ids(&["nativephp"])).unwrap();
        let keys: Vec<&str> = plan.env.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["APP_NAME", "NATIVEPHP_APP_ID"]);
        let bad = vec![EnvVar {
            key: "1A".into(),
            value: String::new(),
        }];
        assert!(check_env(&bad).is_err());
    }

    #[test]
    fn filled_lines() {
        assert_eq!(
            fill(
                "npm create vite@9 {name} -- --identifier {package}",
                "it's mine"
            ),
            "npm create vite@9 'it''s mine' -- --identifier com.example.itsmine"
        );
    }

    #[test]
    fn package_ids() {
        assert_eq!(package_of("My App 2"), "com.example.myapp2");
        assert_eq!(package_of("2048"), "com.example.a2048");
        assert_eq!(package_of("è"), "com.example.a");
    }
}
