//! The kit's terminals (the Terminals page): consoles with the kit's
//! environment that open in any folder of the PC, each with its own tools
//! and versions.
//!
//! - The default terminal is not saved: every tool on disk, each at its
//!   newest line, that is the kit environment as it is.
//! - The user's terminals are in `data\app\terminals.json`: a name, the
//!   folder they start in (`projects\` when none) and the tools they
//!   include, each at its newest line on disk or at a chosen one (the
//!   rewrites of `versions::swap_lines`). A tool left out leaves the PATH,
//!   and so do the variables that point into its folders. The kit's own
//!   commands, the tools of the Required group (PowerShell, GitHub CLI) and
//!   the system folders are always there.
//! - Only what is on disk: a tool or a line removed meanwhile is left out
//!   (a line falls back to the newest, shown on the page as `replaced`),
//!   never downloaded at the start.
//! - Opened in a project's folder, the project's lines win for the tools
//!   the project chose, the terminal's stay for the others (D-179, Q1 of
//!   LEVELS.md): `levels::env::pc_environment` applies both. Every console
//!   gets `ATOLLO_TERMINAL` (its name), so the kit's profile does not apply
//!   the project's lines again (profile.ps1, bashrc).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::common::fsx::{read_json, write_json};
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::setup::{Catalog, Component, Group};

/// The id of the default terminal.
pub const DEFAULT_ID: &str = "default";
/// The longest name, in characters.
const NAME_MAX: usize = 60;
/// Tools of those groups without a command for a terminal.
const NO_COMMAND: &[&str] = &["phpmyadmin"];

/// One save or removal at a time: each rewrites the whole file.
static STORE: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Terminal {
    /// Empty for a new one.
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// The folder it starts in; `None`: `projects\`.
    #[serde(default)]
    pub dir: Option<String>,
    /// The tools it includes, each at its line (`None`: the newest on disk).
    #[serde(default)]
    pub tools: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    terminals: Vec<Terminal>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalInfo {
    #[serde(flatten)]
    pub terminal: Terminal,
    /// The default terminal: neither changed nor removed.
    pub builtin: bool,
    /// Tools it includes that are not on disk (any more): left out.
    pub missing: Vec<String>,
    /// Lines it keeps that are not on disk (any more): the newest is used
    /// instead (BUG-V4-13).
    pub replaced: Vec<LineSwap>,
}

/// A line of a terminal's tool not on disk, and the one used in its place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineSwap {
    pub id: String,
    pub saved: String,
    pub used: String,
}

/// A tool a terminal can include: installed, with its lines on disk.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOption {
    pub id: String,
    pub name: String,
    pub group: Group,
    /// The lines on disk, newest first; empty for a tool with one version.
    pub lines: Vec<String>,
    /// The version of each line, when known.
    pub versions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalsView {
    pub terminals: Vec<TerminalInfo>,
    pub tools: Vec<ToolOption>,
    /// Where a terminal without a folder starts.
    pub projects: String,
}

fn file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("terminals.json")
}

/// The terminals with their folders as full paths: a folder inside the kit
/// is saved relative to it, so it follows the kit when it moves (D-137).
fn load(kit: &Kit) -> Vec<Terminal> {
    let mut all = read_json::<Stored>(&file(kit))
        .unwrap_or_default()
        .terminals;
    for t in &mut all {
        if let Some(d) = &mut t.dir {
            if std::path::Path::new(d.as_str()).is_relative() {
                *d = crate::common::fsx::join_rel(kit.root(), d)
                    .display()
                    .to_string();
            }
        }
    }
    all
}

/// Writes `all`, a folder inside the kit relative to it.
fn store(kit: &Kit, mut all: Vec<Terminal>) -> Result<()> {
    for t in &mut all {
        if let Some(d) = &mut t.dir {
            if let Ok(rel) = std::path::Path::new(d.as_str()).strip_prefix(kit.root()) {
                *d = rel.to_string_lossy().replace('\\', "/");
            }
        }
    }
    write_json(&file(kit), &Stored { terminals: all })
}

/// The components a terminal can leave out or include: agents, local
/// models, languages and databases (the others are always there, or not
/// commands).
fn selectable() -> impl Iterator<Item = &'static Component> {
    Catalog::get().all().iter().filter(|c| {
        matches!(
            c.group,
            Group::Agents | Group::Ai | Group::Languages | Group::Databases
        ) && !NO_COMMAND.contains(&c.id.as_str())
    })
}

/// The tools the kit chose (Linguaggi e strumenti), each at its newest
/// line: what the default terminal and a project have in the VM, where
/// the tools come from the kit's choice for Linux, not from what is on the
/// PC's disk (D-170). Never an agent (LEVELS.md §7, Q11): Claude Code and
/// Codex come with their `sbx` sandboxes, Pi's or Antigravity's build is
/// asked by its own session only (`levels::prepare::wanted_tools`).
pub fn chosen_tools(kit: &Kit) -> BTreeMap<String, Option<String>> {
    let catalog = Catalog::get();
    let choice = crate::setup::catalog::Choice::read_or_defaults(kit, catalog);
    selectable()
        .filter(|c| c.group != Group::Agents && choice.is_chosen(&c.id))
        .map(|c| (c.id.clone(), None))
        .collect()
}

/// The lines of `c` on disk, newest first, with their versions.
fn lines_of(kit: &Kit, c: &Component) -> (Vec<String>, BTreeMap<String, String>) {
    if !c.versioned {
        return (Vec::new(), BTreeMap::new());
    }
    let mut versions = BTreeMap::new();
    let mut lines = Vec::new();
    if let Some(line) = crate::versions::default_line(kit, c) {
        if let Some(v) = c.installed_version(kit) {
            versions.insert(line.clone(), v);
        }
        lines.push(line);
    }
    for l in crate::versions::saved(kit, c) {
        if let Some(v) = l.version {
            versions.insert(l.line.clone(), v);
        }
        lines.push(l.line);
    }
    lines.sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
    lines.dedup();
    (lines, versions)
}

/// The tools on disk a terminal can include, in catalog order: never an
/// agent, which runs only in the VM (D-178).
pub fn tools(kit: &Kit) -> Vec<ToolOption> {
    let target = kit.target();
    selectable()
        .filter(|c| c.group != Group::Agents)
        .filter(|c| c.is_available_on(target) && c.is_installed(kit))
        .map(|c| {
            let (lines, versions) = lines_of(kit, c);
            ToolOption {
                id: c.id.clone(),
                name: c.name.clone(),
                group: c.group,
                lines,
                versions,
            }
        })
        .collect()
}

/// The default terminal: every tool on disk, at its newest line.
fn default_terminal(kit: &Kit, lang: &str) -> Terminal {
    Terminal {
        id: DEFAULT_ID.to_owned(),
        name: crate::common::i18n::t(lang, "Tutti gli strumenti", &[]),
        dir: None,
        tools: tools(kit).into_iter().map(|t| (t.id, None)).collect(),
    }
}

pub fn list(kit: &Kit, lang: &str) -> TerminalsView {
    let tools = tools(kit);
    let on_disk = |id: &str| tools.iter().any(|t| t.id == id);
    // A saved line not among those on disk: `choice_env` keeps the newest.
    let replaced = |t: &Terminal| -> Vec<LineSwap> {
        t.tools
            .iter()
            .filter_map(|(id, line)| {
                let saved = line.as_ref()?;
                let lines = &tools.iter().find(|o| o.id == *id)?.lines;
                let used = lines.first()?;
                (!lines.contains(saved)).then(|| LineSwap {
                    id: id.clone(),
                    saved: saved.clone(),
                    used: used.clone(),
                })
            })
            .collect()
    };
    let mut terminals = vec![TerminalInfo {
        terminal: default_terminal(kit, lang),
        builtin: true,
        missing: Vec::new(),
        replaced: Vec::new(),
    }];
    // An agent a terminal chose before D-178 is not missing: it is left out.
    let agent = |id: &str| crate::agents::IDS.contains(&id);
    terminals.extend(load(kit).into_iter().map(|t| {
        TerminalInfo {
            missing: t
                .tools
                .keys()
                .filter(|id| !on_disk(id) && !agent(id))
                .cloned()
                .collect(),
            replaced: replaced(&t),
            terminal: t,
            builtin: false,
        }
    }));
    TerminalsView {
        terminals,
        tools,
        projects: kit.projects().display().to_string(),
    }
}

/// The terminal `id`, the default one included.
pub fn find(kit: &Kit, id: &str, lang: &str) -> Option<Terminal> {
    if id == DEFAULT_ID {
        return Some(default_terminal(kit, lang));
    }
    load(kit).into_iter().find(|t| t.id == id)
}

/// `name` as an id: lower case letters, digits, `-`.
fn slug(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-');
    if out.is_empty() {
        "terminal".to_owned()
    } else {
        out.to_owned()
    }
}

/// An id for `name` that no terminal has (nor the default).
fn new_id(name: &str, taken: &[Terminal]) -> String {
    let base = slug(name);
    let free = |id: &str| id != DEFAULT_ID && !taken.iter().any(|t| t.id == id);
    if free(&base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|id| free(id))
        .unwrap_or(base)
}

/// The folder a terminal starts in, checked: an absolute path to a folder.
pub fn checked_dir(dir: &str, lang: &str) -> Result<PathBuf> {
    let path = PathBuf::from(dir.trim());
    if !path.is_absolute() || !path.is_dir() {
        bail!(
            "{}",
            crate::common::i18n::t(lang, "La cartella {0} non esiste.", &[dir.trim()])
        );
    }
    Ok(path)
}

/// `input` as saved: a name, a folder that exists or none, only tools a
/// terminal can include and valid lines (of versioned tools).
fn clean(input: Terminal, lang: &str) -> Result<Terminal> {
    let name = input.name.trim().to_owned();
    if name.is_empty() {
        bail!(
            "{}",
            crate::common::i18n::t(lang, "Scrivi un nome per il terminale.", &[])
        );
    }
    if name.chars().count() > NAME_MAX {
        bail!(
            "{}",
            crate::common::i18n::t(
                lang,
                "Il nome del terminale è troppo lungo (al massimo {0} caratteri).",
                &[&NAME_MAX.to_string()]
            )
        );
    }
    let dir = match input.dir.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(d) => Some(checked_dir(d, lang)?.display().to_string()),
    };
    let tools = input
        .tools
        .into_iter()
        .filter_map(|(id, line)| {
            let c = selectable().find(|c| c.id == id)?;
            let line = line.filter(|l| c.versioned && crate::versions::valid_line(l));
            Some((id, line))
        })
        .collect();
    Ok(Terminal {
        id: input.id,
        name,
        dir,
        tools,
    })
}

/// Saves a new terminal (empty id) or the changes of one; returns its id.
pub fn save(kit: &Kit, input: Terminal, lang: &str) -> Result<String> {
    if input.id == DEFAULT_ID {
        bail!(
            "{}",
            crate::common::i18n::t(lang, "Il terminale predefinito non si modifica.", &[])
        );
    }
    let mut terminal = clean(input, lang)?;
    let _lock = STORE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut all = load(kit);
    if terminal.id.is_empty() {
        terminal.id = new_id(&terminal.name, &all);
        all.push(terminal.clone());
    } else {
        let Some(existing) = all.iter_mut().find(|t| t.id == terminal.id) else {
            bail!(
                "{}",
                crate::common::i18n::t(lang, "Terminale non trovato: {0}", &[&terminal.id])
            );
        };
        *existing = terminal.clone();
    }
    store(kit, all)?;
    Ok(terminal.id)
}

pub fn remove(kit: &Kit, id: &str) -> Result<()> {
    let _lock = STORE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut all = load(kit);
    let before = all.len();
    all.retain(|t| t.id != id);
    if all.len() != before {
        store(kit, all)?;
    }
    Ok(())
}

/// Variables whose folder belongs to a tool besides its own folder in
/// `data\apps`: what it installs for the user (global packages, tools).
fn data_vars(id: &str) -> &'static [&'static str] {
    match id {
        "python" => &["PYTHONUSERBASE"],
        "node" => &["NPM_CONFIG_PREFIX"],
        "pnpm" => &["PNPM_HOME"],
        "uv" => &["UV_TOOL_DIR", "UV_TOOL_BIN_DIR"],
        "php" => &["COMPOSER_HOME"],
        "go" => &["GOPATH", "GOBIN"],
        "dotnet" => &["DOTNET_CLI_HOME"],
        "dart" => &["PUB_CACHE"],
        "bun" => &["BUN_INSTALL"],
        "deno" => &["DENO_INSTALL_ROOT"],
        _ => &[],
    }
}

/// The folders of `c`: its own in `data\apps` (every line), the other ones
/// it owns there (Maven and Gradle of Java...) and those of `data_vars`.
fn owned_dirs(kit: &Kit, c: &Component, env: &KitEnv) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = c.folder_path(kit).into_iter().collect();
    out.extend(
        c.remove
            .iter()
            .filter(|r| !r.starts_with('@') && !r.contains(['/', '\\']))
            .map(|r| kit.apps().join(r)),
    );
    out.extend(
        data_vars(&c.id)
            .iter()
            .filter_map(|v| env.get(v))
            .map(PathBuf::from),
    );
    out
}

/// `p` with one separator, compared as the PC's file system does.
fn norm(p: &str) -> String {
    crate::platform::processes::path_key(p.replace('/', "\\").trim_end_matches('\\'))
}

/// `path` is one of `dirs` or inside one.
fn under(path: &str, dirs: &[String]) -> bool {
    let p = norm(path);
    !p.is_empty()
        && dirs.iter().any(|d| {
            p == *d
                || p.strip_prefix(d.as_str())
                    .is_some_and(|r| r.starts_with('\\'))
        })
}

/// `env` without what is inside `dirs`: the PATH entries, and the
/// variables that point there.
pub fn strip(env: &mut KitEnv, dirs: &[PathBuf]) {
    let dirs: Vec<String> = dirs
        .iter()
        .map(|d| norm(&d.display().to_string()))
        .filter(|d| !d.is_empty())
        .collect();
    if dirs.is_empty() {
        return;
    }
    let sep = env.path_sep();
    let mut sep_buf = [0u8; 4];
    let sep_str: &str = sep.encode_utf8(&mut sep_buf);
    let path = env
        .path_entries()
        .into_iter()
        .filter(|e| !under(e, &dirs))
        .collect::<Vec<_>>()
        .join(sep_str);
    let gone: Vec<String> = env
        .iter()
        .filter(|(k, v)| !k.eq_ignore_ascii_case("PATH") && under(v, &dirs))
        .map(|(k, _)| k.to_owned())
        .collect();
    for name in gone {
        env.remove(&name);
    }
    env.set("PATH", path);
}

/// `terminal`'s tools in `env` (a kit environment on the kit's lines,
/// [`crate::versions::kit_choice`]): the lines it chose
/// ([`crate::versions::swap_lines`]) and the tools it left out stripped
/// ([`strip`]); the lines `env` is on afterwards. The default terminal
/// changes nothing. Step 2 of `levels::env::pc_environment` (LEVELS.md
/// §3.5), which gives a terminal its whole environment: the project's
/// lines in a project's folder, sealed, without the agents, with
/// `ATOLLO_TERMINAL`.
pub fn apply_tools(kit: &Kit, terminal: &Terminal, env: &mut KitEnv) -> crate::versions::Choice {
    let lines = crate::versions::kit_choice(kit);
    if terminal.id == DEFAULT_ID {
        return lines;
    }
    let choice: crate::versions::Choice = terminal
        .tools
        .iter()
        .filter_map(|(id, line)| Some((id.clone(), line.clone()?)))
        .collect();
    let lines = crate::versions::swap_lines(kit, env, &lines, &choice);
    let left_out: Vec<PathBuf> = selectable()
        .filter(|c| !terminal.tools.contains_key(&c.id))
        .flat_map(|c| owned_dirs(kit, c, env))
        .collect();
    strip(env, &left_out);
    lines
}

/// The folder `terminal` starts in: `dir` when given, its own, or
/// `projects\`.
pub fn start_dir(kit: &Kit, terminal: &Terminal, dir: Option<&str>, lang: &str) -> Result<PathBuf> {
    match dir.or(terminal.dir.as_deref()) {
        Some(d) => checked_dir(d, lang),
        None => Ok(kit.projects()),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/editor/terminals.rs"]
mod tests;
