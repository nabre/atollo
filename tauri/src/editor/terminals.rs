//! The kit's terminals (the Terminals page): consoles with the kit's
//! environment that open in any folder of the PC, each with its own tools
//! and versions.
//!
//! - The default terminal is not saved: every tool on disk, each at its
//!   newest line, that is the kit environment as it is.
//! - The user's terminals are in `data\app\terminals.json`: a name, the
//!   folder they start in (`projects\` when none) and the tools they
//!   include, each at its newest line on disk or at a chosen one (the
//!   rewrites of `versions::choice_env`). A tool left out leaves the PATH,
//!   and so do the variables that point into its folders. The kit's own
//!   commands, the tools of the Required group (PowerShell, GitHub CLI) and
//!   the system folders are always there.
//! - Only what is on disk: a tool or a line removed meanwhile is left out
//!   (a line falls back to the newest), never downloaded at the start.
//! - Every console gets `DEVKIT_TERMINAL` (its name): the kit's profile
//!   then keeps the terminal's versions in a project folder too, instead of
//!   the project's (profile.ps1, bashrc).

use std::collections::BTreeMap;
#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::env::KitEnv;
use crate::fsx::{read_json, write_json};
use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Component, Group};

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

fn load(kit: &Kit) -> Vec<Terminal> {
    read_json::<Stored>(&file(kit))
        .unwrap_or_default()
        .terminals
}

/// The components a terminal can leave out or include: agents, languages
/// and databases (the others are always there, or not commands).
fn selectable() -> impl Iterator<Item = &'static Component> {
    Catalog::get().all().iter().filter(|c| {
        matches!(c.group, Group::Agents | Group::Languages | Group::Databases)
            && !NO_COMMAND.contains(&c.id.as_str())
    })
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

/// The tools on disk a terminal can include, in catalog order.
pub fn tools(kit: &Kit) -> Vec<ToolOption> {
    let target = kit.target();
    selectable()
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
        name: crate::i18n::t(lang, "Tutti gli strumenti", &[]),
        dir: None,
        tools: tools(kit).into_iter().map(|t| (t.id, None)).collect(),
    }
}

pub fn list(kit: &Kit, lang: &str) -> TerminalsView {
    let tools = tools(kit);
    let on_disk = |id: &str| tools.iter().any(|t| t.id == id);
    let mut terminals = vec![TerminalInfo {
        terminal: default_terminal(kit, lang),
        builtin: true,
        missing: Vec::new(),
    }];
    terminals.extend(load(kit).into_iter().map(|t| TerminalInfo {
        missing: t.tools.keys().filter(|id| !on_disk(id)).cloned().collect(),
        terminal: t,
        builtin: false,
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
            crate::i18n::t(lang, "La cartella {0} non esiste.", &[dir.trim()])
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
            crate::i18n::t(lang, "Scrivi un nome per il terminale.", &[])
        );
    }
    if name.chars().count() > NAME_MAX {
        bail!(
            "{}",
            crate::i18n::t(
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
            crate::i18n::t(lang, "Il terminale predefinito non si modifica.", &[])
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
                crate::i18n::t(lang, "Terminale non trovato: {0}", &[&terminal.id])
            );
        };
        *existing = terminal.clone();
    }
    write_json(&file(kit), &Stored { terminals: all })?;
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
        write_json(&file(kit), &Stored { terminals: all })?;
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
    crate::os::processes::path_key(p.replace('/', "\\").trim_end_matches('\\'))
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

/// The environment of `terminal`, from the kit's (`base`).
pub fn env(kit: &Kit, terminal: &Terminal, base: KitEnv) -> KitEnv {
    let mut env = if terminal.id == DEFAULT_ID {
        base
    } else {
        let choice: crate::versions::Choice = terminal
            .tools
            .iter()
            .filter_map(|(id, line)| Some((id.clone(), line.clone()?)))
            .collect();
        let mut env = crate::versions::choice_env(kit, &choice, base);
        let left_out: Vec<PathBuf> = selectable()
            .filter(|c| !terminal.tools.contains_key(&c.id))
            .flat_map(|c| owned_dirs(kit, c, &env))
            .collect();
        strip(&mut env, &left_out);
        env
    };
    env.set("DEVKIT_TERMINAL", terminal.name.clone());
    env
}

/// The folder `terminal` starts in: `dir` when given, its own, or
/// `projects\`.
pub fn start_dir(kit: &Kit, terminal: &Terminal, dir: Option<&str>, lang: &str) -> Result<PathBuf> {
    match dir.or(terminal.dir.as_deref()) {
        Some(d) => checked_dir(d, lang),
        None => Ok(kit.projects()),
    }
}

/// Windows' "Select folder" window over `owner` (a window handle), starting
/// in `start`. `None` when closed without a choice. macOS and Linux:
/// `editor::picker::folder`.
#[cfg(windows)]
pub fn pick_dir(owner: isize, title: &str, start: Option<&Path>) -> Result<Option<PathBuf>> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Foundation::{ERROR_CANCELLED, HWND};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileOpenDialog, IShellItem, SHCreateItemFromParsingName,
        FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
    };
    // SAFETY: as `setup::downloads::pick`: COM initialised on this thread
    // for the dialog and released after every COM object of the closure is
    // dropped; the strings COM allocates are freed once read.
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        let result = (|| -> Result<Option<PathBuf>> {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(
                dialog.GetOptions()? | FOS_PICKFOLDERS | FOS_PATHMUSTEXIST | FOS_FORCEFILESYSTEM,
            )?;
            dialog.SetTitle(&HSTRING::from(title))?;
            if let Some(start) = start.filter(|s| s.is_dir()) {
                let start = HSTRING::from(start.as_os_str());
                let folder: windows::core::Result<IShellItem> =
                    SHCreateItemFromParsingName(PCWSTR(start.as_ptr()), None);
                if let Ok(folder) = folder {
                    let _ = dialog.SetFolder(&folder);
                }
            }
            if let Err(e) = dialog.Show(Some(HWND(owner as _))) {
                if e.code() == ERROR_CANCELLED.to_hresult() {
                    return Ok(None);
                }
                return Err(e.into());
            }
            let path = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let text = path.to_string();
            CoTaskMemFree(Some(path.0 as _));
            Ok(Some(PathBuf::from(text?)))
        })();
        if init.is_ok() {
            CoUninitialize();
        }
        result
    }
}

#[cfg(test)]
#[path = "../../../test/rust/editor/terminals.rs"]
mod tests;
