//! Saved versions of the languages and tools (DECISIONS.md D-80).
//!
//! Every version has its folder inside the component's own, one per minor
//! line (`major.minor`): `data\apps\<platform>\node\<line>`, with the full
//! version in `.atollo-version`. The default is the folder of the installed
//! version (`data\app\versions\<id>.txt`), the one updates replace:
//! - an update that changes the minor line installs next to the old one,
//!   which stays as a saved line (a new patch of the same line replaces);
//! - a line can be downloaded there from the Tools page;
//! - the kit uses the newest line on disk; a project the one in its
//!   `dev-settings.json` (`versions`), the newest when it was created;
//! - the lines each source offers are kept in `data\app\version-lines.json`,
//!   so the lists propose them also offline; choosing one that is not saved
//!   downloads it.
//!
//! A choice is applied by rewriting the kit environment: every value that
//! points into the default folder (PATH, `JAVA_HOME`, `DOTNET_ROOT`,
//! `CP_PHP`) points into the chosen one instead. The editor's terminals
//! start from the shared editor server, so each project with a choice gets
//! `data\app\projects\<project>\versions.json` (the rewrites), which the
//! kit's PowerShell profile applies.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, Component};
use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::Kit;

/// The file with the full version, inside a saved folder.
const VERSION_FILE: &str = ".atollo-version";

/// `major.minor` of a version: `v22.11.0` → `22.11`, `8.4.13 (Composer 2.8.1)`
/// → `8.4`, `21.0.5+11` → `21.0`. `None` without two numbers.
pub fn line_of(version: &str) -> Option<String> {
    let start = version.find(|c: char| c.is_ascii_digit())?;
    let mut parts = version[start..]
        .split(|c: char| !c.is_ascii_digit() && c != '.')
        .next()?
        .split('.');
    let (major, minor) = (parts.next()?, parts.next()?);
    if major.is_empty() || minor.is_empty() {
        return None;
    }
    Some(format!("{major}.{minor}"))
}

/// A line as the interface and the files write it: `22.11`.
pub fn valid_line(line: &str) -> bool {
    line_of(line).as_deref() == Some(line)
}

/// The versioned components of the catalog.
pub fn versioned() -> impl Iterator<Item = &'static Component> {
    Catalog::get().all().iter().filter(|c| c.versioned)
}

fn find(id: &str) -> Option<&'static Component> {
    Catalog::get().find(id).filter(|c| c.versioned)
}

/// `data\apps\<platform>\<folder>`: the folder of every version.
fn store(kit: &Kit, c: &Component) -> PathBuf {
    c.folder_path(kit).unwrap_or_else(|| kit.apps().join(&c.id))
}

/// The folder of `version`: its line, or the version itself (safe
/// characters only) when it has no `major.minor`.
pub fn dir_name(version: &str) -> String {
    line_of(version).unwrap_or_else(|| {
        version
            .trim()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' {
                    ch
                } else {
                    '_'
                }
            })
            .collect()
    })
}

/// Where `line` of `c` is (or goes).
pub fn saved_dir(kit: &Kit, c: &Component, line: &str) -> PathBuf {
    store(kit, c).join(line)
}

/// Where `version` of `c` is installed as the default.
pub fn install_dir(kit: &Kit, c: &Component, version: &str) -> PathBuf {
    saved_dir(kit, c, &dir_name(version))
}

/// The default folder of `c`: that of the installed version; the
/// component's folder itself while it has the layout of an older kit
/// (`migrate` not done yet) or nothing is installed.
pub fn default_dir(kit: &Kit, c: &Component) -> PathBuf {
    let store = store(kit, c);
    if let Some(version) = c.installed_version(kit) {
        let dir = store.join(dir_name(&version));
        if has_key(c, &dir) {
            return dir;
        }
    }
    store
}

/// `rel` of the catalog (`node/node.exe`) inside the default folder of the
/// versioned component it starts with; `None` for the others.
pub fn resolve(kit: &Kit, rel: &str) -> Option<PathBuf> {
    let rel = rel.replace('/', "\\");
    let (head, rest) = rel.split_once('\\').unwrap_or((rel.as_str(), ""));
    let c = versioned().find(|c| {
        c.folder
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case(head))
    })?;
    let dir = default_dir(kit, c);
    Some(if rest.is_empty() { dir } else { dir.join(rest) })
}

/// The key of `c` is in `dir`, a copy of its own folder.
pub fn has_key(c: &Component, dir: &Path) -> bool {
    let Some(folder) = &c.folder else {
        return false;
    };
    let rel = c
        .key
        .strip_prefix(folder.as_str())
        .unwrap_or(&c.key)
        .trim_start_matches('/');
    let path = dir.join(rel.replace('/', "\\"));
    path.is_file() || (rel.is_empty() && fs::read_dir(dir).is_ok_and(|mut d| d.next().is_some()))
}

/// The line of the default folder, when it is installed.
pub fn default_line(kit: &Kit, c: &Component) -> Option<String> {
    if !c.is_installed(kit) {
        return None;
    }
    c.installed_version(kit).as_deref().and_then(line_of)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub line: String,
    /// The full version, when known.
    pub version: Option<String>,
}

/// The saved lines of `c` (not the default), newest first.
pub fn saved(kit: &Kit, c: &Component) -> Vec<Line> {
    let default = default_line(kit, c);
    let mut out: Vec<Line> = fs::read_dir(store(kit, c))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let line = e.file_name().to_string_lossy().into_owned();
            let other = valid_line(&line) && default.as_deref() != Some(line.as_str());
            (other && has_key(c, &e.path())).then(|| Line {
                version: fs::read_to_string(e.path().join(VERSION_FILE))
                    .ok()
                    .map(|v| v.trim().to_owned())
                    .filter(|v| !v.is_empty()),
                line,
            })
        })
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(&l.line)));
    out
}

pub fn write_version(dir: &Path, version: &str) -> Result<()> {
    write_atomic(&dir.join(VERSION_FILE), version.as_bytes())
        .with_context(|| dir.display().to_string())
}

/// The folder used for `line` of `c`: the saved one, or the default folder
/// when `line` is the default's, is not chosen or is not saved (any more).
pub fn dir_for(kit: &Kit, c: &Component, line: Option<&str>) -> Option<PathBuf> {
    c.folder.as_ref()?;
    let main = default_dir(kit, c);
    Some(match line {
        Some(l) if Some(l) != default_line(kit, c).as_deref() => {
            let dir = saved_dir(kit, c, l);
            if has_key(c, &dir) {
                dir
            } else {
                main
            }
        }
        _ => main,
    })
}

/// Lines chosen per component id.
pub type Choice = BTreeMap<String, String>;

/// The newest line of `c` on disk (the default or a saved one).
pub fn newest_line(kit: &Kit, c: &Component) -> Option<String> {
    default_line(kit, c)
        .into_iter()
        .chain(saved(kit, c).into_iter().map(|l| l.line))
        .max_by_key(|l| crate::install::version_key(l))
}

/// The lines the whole kit uses: for each versioned component, the newest
/// on disk where it is not the default folder (a newer line downloaded
/// with "+"). No choice of the user (15/10/2026): the kit always uses the
/// newest, each project its own.
pub fn kit_choice(kit: &Kit) -> Choice {
    versioned()
        .filter_map(|c| {
            let newest = newest_line(kit, c)?;
            (default_line(kit, c).as_deref() != Some(newest.as_str()))
                .then(|| (c.id.clone(), newest))
        })
        .collect()
}

/// A new project keeps the newest version on disk of every language: newer
/// versions installed later do not change it (its code was written for
/// these). Languages installed later follow the kit until chosen.
pub fn pin_newest(kit: &Kit, project: &str) {
    let lines: Choice = versioned()
        .filter_map(|c| Some((c.id.clone(), newest_line(kit, c)?)))
        .collect();
    if lines.is_empty() {
        return;
    }
    crate::projects::settings::update(kit, project, |s| {
        for (id, line) in &lines {
            s.versions.entry(id.clone()).or_insert_with(|| line.clone());
        }
    });
    write_project_file(kit, project);
}

/// Only versioned components and valid lines.
pub fn clean(choice: Choice) -> Choice {
    choice
        .into_iter()
        .filter(|(id, line)| find(id).is_some() && valid_line(line))
        .collect()
}

/// Checks a choice from the interface.
pub fn check(id: &str, line: Option<&str>) -> Result<()> {
    if find(id).is_none() {
        bail!("not a versioned component: {id}");
    }
    if let Some(l) = line {
        if !valid_line(l) {
            bail!("not a version line: {l}");
        }
    }
    Ok(())
}

/// A rewrite of the environment: values in `from` point into `to`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Swap {
    pub from: String,
    pub to: String,
}

/// The rewrites from the folders of `base` to those of `choice`.
fn swaps(kit: &Kit, base: &Choice, choice: &Choice) -> Vec<Swap> {
    choice
        .iter()
        .filter_map(|(id, line)| {
            let c = find(id)?;
            // A line chosen and not on disk (removed, or still downloading):
            // the base stays, the kit's default for a project.
            if !is_present(kit, c, line) {
                return None;
            }
            let from = dir_for(kit, c, base.get(id).map(String::as_str))?;
            let to = dir_for(kit, c, Some(line))?;
            (from != to).then(|| Swap {
                from: from.display().to_string(),
                to: to.display().to_string(),
            })
        })
        .collect()
}

/// `value` with every entry (`;`-separated) under `from` moved under `to`.
fn rewrite_value(value: &str, swaps: &[Swap]) -> String {
    value
        .split(';')
        .map(|entry| {
            for s in swaps {
                let lower = entry.to_ascii_lowercase();
                let from = s.from.to_ascii_lowercase();
                if lower == from || lower.starts_with(&format!("{from}\\")) {
                    return format!("{}{}", s.to, &entry[s.from.len()..]);
                }
            }
            entry.to_owned()
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn apply(env: &mut KitEnv, swaps: &[Swap]) {
    if swaps.is_empty() {
        return;
    }
    let changed: Vec<(String, String)> = env
        .iter()
        .filter_map(|(k, v)| {
            let new = rewrite_value(v, swaps);
            (new != v).then(|| (k.to_owned(), new))
        })
        .collect();
    for (k, v) in changed {
        env.set(&k, v);
    }
    // Apache's PHP (`/` paths, not rewritten above) is the PHP in use: its
    // module and php.ini's `${CP_PHP}\ext` must be the same version.
    if let Some(php) = env.get("CP_PHP").map(|p| p.replace('\\', "/")) {
        env.set("CP_PHP_FWD", php);
    }
}

/// The kit's lines (the newest on disk), applied to an environment built
/// on the default folders.
pub fn apply_kit_choice(kit: &Kit, env: &mut KitEnv) {
    apply(env, &swaps(kit, &Choice::new(), &kit_choice(kit)));
}

/// The lines chosen in `project`.
pub fn project_choice(kit: &Kit, project: &str) -> Choice {
    clean(crate::projects::settings::load(kit, project).versions)
}

fn project_swaps(kit: &Kit, project: &str) -> Vec<Swap> {
    swaps(kit, &kit_choice(kit), &project_choice(kit, project))
}

/// The kit environment (with the kit's lines) for a program of `project`.
pub fn project_env(kit: &Kit, project: &str, mut env: KitEnv) -> KitEnv {
    apply(&mut env, &project_swaps(kit, project));
    env
}

pub fn set_project_choice(kit: &Kit, project: &str, id: &str, line: Option<&str>) -> Result<()> {
    check(id, line)?;
    crate::projects::settings::update(kit, project, |s| {
        match line {
            Some(l) => s.versions.insert(id.to_owned(), l.to_owned()),
            None => s.versions.remove(id),
        };
    });
    write_project_file(kit, project);
    Ok(())
}

/// `versions.json` of `project` for the editor's PowerShell terminals
/// (profile.ps1), and `versions.txt` (`from<TAB>to` lines) for Git Bash
/// (bashrc); removed when the project has no rewrite.
pub fn write_project_file(kit: &Kit, project: &str) {
    let Some(dir) = crate::projects::settings::dir(kit, project) else {
        return;
    };
    let (json, txt) = (dir.join("versions.json"), dir.join("versions.txt"));
    let swaps = project_swaps(kit, project);
    if swaps.is_empty() {
        let _ = fs::remove_file(json);
        let _ = fs::remove_file(txt);
        return;
    }
    let text =
        serde_json::to_string_pretty(&serde_json::json!({ "swaps": swaps })).unwrap_or_default();
    let lines: String = swaps
        .iter()
        .map(|s| format!("{}\t{}\n", s.from, s.to))
        .collect();
    let result = fs::create_dir_all(&dir)
        .and_then(|()| write_atomic(&json, text.as_bytes()))
        .and_then(|()| write_atomic(&txt, lines.as_bytes()));
    if let Err(e) = result {
        crate::log::warn(&format!("versions of {project}: {e}"));
    }
}

/// Every project's `versions.json`, after a change of the kit's lines or of
/// what is installed.
pub fn write_project_files(kit: &Kit) {
    for entry in fs::read_dir(kit.project_states())
        .into_iter()
        .flatten()
        .flatten()
    {
        write_project_file(kit, &entry.file_name().to_string_lossy());
    }
}

/// Removes a line. The default goes only while another one stays, which
/// becomes the default (the newest); the last one goes with the component
/// (the Tools page, which follows what needs it).
pub fn remove(kit: &Kit, id: &str, line: &str) -> Result<()> {
    check(id, Some(line))?;
    let Some(c) = find(id) else { return Ok(()) };
    // Locked: the projects that use it keep it.
    if let Some(users) = project_users(kit).get(&(id.to_owned(), line.to_owned())) {
        bail!("{id} {line}: used by {}", users.join(", "));
    }
    if default_line(kit, c).as_deref() != Some(line) {
        return crate::archive::remove_dir(&saved_dir(kit, c, line));
    }
    let Some(next) = saved(kit, c).into_iter().next() else {
        bail!("{id} {line}: the only version, it goes with the component");
    };
    let dir = default_dir(kit, c);
    // Never the component's folder (an older kit's layout, not moved yet):
    // it holds every version.
    if dir != saved_dir(kit, c, line) {
        bail!("{id} {line}: not in its own folder yet");
    }
    crate::archive::remove_dir(&dir)?;
    promote(kit, c, &next)
}

/// `line` becomes the default: the installed version is its own.
fn promote(kit: &Kit, c: &Component, line: &Line) -> Result<()> {
    let version = line.version.clone().unwrap_or_else(|| line.line.clone());
    c.save_version(kit, &version)?;
    crate::log::info(&format!("{}: {version} is the default", c.id));
    Ok(())
}

/// What was deleted by hand from `data\apps` (D-81), when the Tools page
/// opens: a component installed before (its version is known) of which
/// nothing is on disk is no longer chosen (with what needs it) and its
/// version is forgotten; a versioned one whose default folder went takes
/// the newest line left. Required components stay chosen: they come back
/// at the next start. Returns the names no longer chosen.
pub fn sync_with_disk(kit: &Kit) -> Result<Vec<String>> {
    let catalog = Catalog::get();
    for c in versioned() {
        if let Err(e) = repair_default(kit, c) {
            crate::log::warn(&format!("{}: no default version ({e:#})", c.id));
        }
    }
    let Some(mut choice) = crate::catalog::Choice::read(kit, catalog) else {
        return Ok(Vec::new());
    };
    let gone: Vec<&Component> = catalog
        .all()
        .iter()
        .filter(|c| c.installed_version(kit).is_some() && !c.is_present(kit))
        .collect();
    let before = choice.clone();
    for c in &gone {
        c.forget_version(kit);
        if !catalog.is_locked(&c.id) {
            choice.set(catalog, &c.id, false);
        }
    }
    if choice == before {
        return Ok(Vec::new());
    }
    choice.write(kit, catalog)?;
    let off: Vec<String> = catalog
        .all()
        .iter()
        .filter(|c| before.is_chosen(&c.id) && !choice.is_chosen(&c.id))
        .map(|c| c.name.clone())
        .collect();
    crate::log::info(&format!(
        "deleted by hand, no longer chosen: {}",
        off.join(", ")
    ));
    Ok(off)
}

/// The default's folder is gone (removed by hand) and other lines are
/// there: the newest becomes the default.
fn repair_default(kit: &Kit, c: &Component) -> Result<()> {
    if c.installed_version(kit).is_none() || has_key(c, &default_dir(kit, c)) {
        return Ok(());
    }
    match saved(kit, c).into_iter().next() {
        Some(next) => promote(kit, c, &next),
        None => Ok(()),
    }
}

/// Brings the folders of an older kit to one folder per version, at start
/// before anything uses them: the default `node\` into `node\<line>`, the
/// saved `versions\<id>\<line>` into `node\<line>`. The saved lines of
/// components no longer versioned go. A folder in use stays as it is (the
/// kit works with it) and is moved at the next start.
pub fn migrate(kit: &Kit) {
    for c in versioned() {
        if let Err(e) = migrate_default(kit, c) {
            crate::log::warn(&format!("{}: default folder not moved ({e:#})", c.id));
        }
    }
    let old = kit.apps().join("versions");
    for entry in fs::read_dir(&old).into_iter().flatten().flatten() {
        let id = entry.file_name().to_string_lossy().into_owned();
        let result = match find(&id) {
            Some(c) => migrate_saved(kit, c, &entry.path()),
            None => crate::archive::remove_dir(&entry.path()),
        };
        if let Err(e) = result {
            crate::log::warn(&format!("{id}: saved versions not moved ({e:#})"));
        }
    }
    let _ = fs::remove_dir(&old);
    for c in versioned() {
        if let Err(e) = repair_default(kit, c) {
            crate::log::warn(&format!("{}: no default version ({e:#})", c.id));
        }
    }
    // The databases' data, one folder per version too.
    crate::services::migrate_data(kit);
    if let Err(e) = crate::install::adopt_composer(kit) {
        crate::log::warn(&format!("Composer of the older kit: {e:#}"));
    }
    if let Err(e) = crate::projects::apache::fix_kit_conf(kit) {
        crate::log::warn(&format!("Apache kit.conf: {e:#}"));
    }
}

/// `node\node.exe` → `node\<line>\node.exe`, through `node.atollo-move`
/// (left by an interrupted start: taken back first).
fn migrate_default(kit: &Kit, c: &Component) -> Result<()> {
    if c.folder.is_none() {
        return Ok(());
    }
    let store = store(kit, c);
    let mut moving = store.clone().into_os_string();
    moving.push(".atollo-move");
    let moving = PathBuf::from(moving);
    if moving.exists() && !store.exists() {
        fs::rename(&moving, &store).context("taking back the folder being moved")?;
    }
    if !has_key(c, &store) {
        return Ok(());
    }
    let Some(version) = c.installed_version(kit) else {
        return Ok(()); // no version to name its folder: it stays
    };
    let to = install_dir(kit, c, &version);
    fs::rename(&store, &moving).with_context(|| store.display().to_string())?;
    let moved = fs::create_dir_all(&store).and_then(|()| fs::rename(&moving, &to));
    if let Err(e) = moved {
        let _ = fs::remove_dir(&store);
        let _ = fs::rename(&moving, &store);
        return Err(e.into());
    }
    write_version(&to, &version)?;
    crate::log::info(&format!("{}: {version} moved to {}", c.id, to.display()));
    Ok(())
}

/// `versions\<id>\<line>` → `<folder>\<line>`; a line already there wins.
fn migrate_saved(kit: &Kit, c: &Component, old: &Path) -> Result<()> {
    for entry in fs::read_dir(old)?.flatten() {
        let line = entry.file_name().to_string_lossy().into_owned();
        let to = saved_dir(kit, c, &line);
        if !valid_line(&line) || has_key(c, &to) {
            crate::archive::remove_dir(&entry.path())?;
            continue;
        }
        fs::create_dir_all(store(kit, c))?;
        fs::rename(entry.path(), &to).with_context(|| to.display().to_string())?;
    }
    fs::remove_dir(old)?;
    Ok(())
}

/// The lines a source offered for a component, and when (Unix seconds).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offered {
    pub lines: Vec<String>,
    pub checked: u64,
}

pub(crate) fn offered_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("version-lines.json")
}

/// Every component's offered lines (only valid ones, newest first).
pub fn offered(kit: &Kit) -> BTreeMap<String, Offered> {
    let mut all: BTreeMap<String, Offered> =
        crate::fsx::read_json(&offered_file(kit)).unwrap_or_default();
    all.retain(|id, _| find(id).is_some());
    for o in all.values_mut() {
        o.lines.retain(|l| valid_line(l));
        o.lines
            .sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
        o.lines.dedup();
    }
    all
}

/// Keeps what the source of `id` offers now.
pub fn save_offered(kit: &Kit, id: &str, lines: &[String]) -> Result<()> {
    // Lists of several components can be asked for at the same time.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut all = offered(kit);
    let checked = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    all.insert(
        id.to_owned(),
        Offered {
            lines: lines.iter().filter(|l| valid_line(l)).cloned().collect(),
            checked,
        },
    );
    crate::fsx::write_json(&offered_file(kit), &all)
}

/// `line` of `c` is on disk: the default folder or a saved one.
pub fn is_present(kit: &Kit, c: &Component, line: &str) -> bool {
    default_line(kit, c).as_deref() == Some(line) || has_key(c, &saved_dir(kit, c, line))
}

/// `line` of `id` has to be downloaded before it can be used: the component
/// is installed, the line is not on disk and its installer can fetch it.
pub fn to_download(kit: &Kit, id: &str, line: &str) -> Result<bool> {
    check(id, Some(line))?;
    let Some(c) = find(id) else { return Ok(false) };
    if !c.is_installed(kit) || is_present(kit, c, line) {
        return Ok(false);
    }
    if !crate::install::installer(id).is_some_and(|i| i.has_lines()) {
        bail!("{id} {line}: not saved and it cannot be downloaded");
    }
    Ok(true)
}

/// The projects that chose a line, per component id and line.
fn project_users(kit: &Kit) -> BTreeMap<(String, String), Vec<String>> {
    let mut out: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for entry in fs::read_dir(kit.project_states())
        .into_iter()
        .flatten()
        .flatten()
    {
        let project = entry.file_name().to_string_lossy().into_owned();
        for (id, line) in project_choice(kit, &project) {
            out.entry((id, line)).or_default().push(project.clone());
        }
    }
    out
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub installed: bool,
    /// The default folder.
    pub default: Option<Line>,
    pub saved: Vec<Line>,
    /// The line the kit uses when newer than the default folder's.
    pub preferred: Option<String>,
    /// Lines can be downloaded (the installer knows the older ones).
    pub downloadable: bool,
    /// The lines its source offered the last time it was asked, newest
    /// first (also those already on disk).
    pub available: Vec<String>,
    /// When the source was asked (Unix seconds); `None`: never.
    pub checked: Option<u64>,
    /// The projects that chose each line.
    pub projects: BTreeMap<String, Vec<String>>,
}

pub fn tools(kit: &Kit) -> Vec<Tool> {
    let choice = kit_choice(kit);
    let mut offered = offered(kit);
    let users = project_users(kit);
    versioned()
        .map(|c| {
            let downloadable = crate::install::installer(&c.id).is_some_and(|i| i.has_lines());
            let offer = offered.remove(&c.id).filter(|_| downloadable);
            Tool {
                id: c.id.clone(),
                name: c.name.clone(),
                installed: c.is_installed(kit),
                default: default_line(kit, c).map(|line| Line {
                    line,
                    version: c.installed_version(kit),
                }),
                saved: saved(kit, c),
                preferred: choice.get(&c.id).cloned(),
                downloadable,
                checked: offer.as_ref().map(|o| o.checked),
                available: offer.map(|o| o.lines).unwrap_or_default(),
                projects: users
                    .iter()
                    .filter(|((id, _), _)| *id == c.id)
                    .map(|((_, line), p)| (line.clone(), p.clone()))
                    .collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn lines() {
        for (v, l) in [
            ("v22.11.0", Some("22.11")),
            ("3.13.10", Some("3.13")),
            ("8.4.13 (Composer 2.8.12)", Some("8.4")),
            ("21.0.5+11 (Maven 3.9.11, Gradle 9.1.0)", Some("21.0")),
            ("1.25", Some("1.25")),
            ("0.15.1", Some("0.15")),
            ("go1.25.3", Some("1.25")),
            ("22", None),
            ("", None),
        ] {
            assert_eq!(line_of(v).as_deref(), l, "{v}");
        }
        assert!(valid_line("22.11") && !valid_line("22.11.0") && !valid_line("x"));
        assert!(!valid_line(r"..\x") && !valid_line("1.2;3"));
    }

    fn install_fake(kit: &Kit, c: &Component, dir: &Path) {
        let rel = c.key.strip_prefix(c.folder.as_deref().unwrap()).unwrap();
        let key = dir.join(rel.trim_start_matches('/').replace('/', "\\"));
        fs::create_dir_all(key.parent().unwrap()).unwrap();
        fs::write(key, b"").unwrap();
        let _ = kit;
    }

    #[test]
    fn choices_rewrite_the_environment() {
        let tmp = TempDir::new("versions");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let node = Catalog::get().find("node").unwrap();
        let main = install_dir(&kit, node, "v22.11.0");
        assert!(main.ends_with(r"node\22.11"));
        install_fake(&kit, node, &main);
        node.save_version(&kit, "v22.11.0").unwrap();
        assert_eq!(default_line(&kit, node).as_deref(), Some("22.11"));
        assert_eq!(default_dir(&kit, node), main);
        assert_eq!(
            crate::catalog::kit_path(&kit, "node/node.exe"),
            main.join("node.exe")
        );
        assert!(node.is_installed(&kit));
        let old = saved_dir(&kit, node, "20.18");
        install_fake(&kit, node, &old);
        write_version(&old, "v20.18.1").unwrap();
        fs::create_dir_all(saved_dir(&kit, node, "18.1")).unwrap(); // no key
        assert_eq!(
            saved(&kit, node),
            [Line {
                line: "20.18".into(),
                version: Some("v20.18.1".into())
            }]
        );
        assert_eq!(dir_for(&kit, node, Some("22.11")), Some(main.clone()));
        assert_eq!(dir_for(&kit, node, Some("18.1")), Some(main.clone()));
        assert_eq!(dir_for(&kit, node, Some("20.18")), Some(old.clone()));

        let mut env = KitEnv::default();
        let path = format!(
            "{};{}\\x;{}x;C:\\W",
            main.display(),
            main.display(),
            main.display()
        );
        env.set("PATH", path);
        let base = env.clone();

        // Apache's PHP follows the PHP in use (its module and php.ini's
        // ext folder must be the same version).
        let mut php_env = KitEnv::default();
        php_env.set("CP_PHP", main.display().to_string());
        apply(
            &mut php_env,
            &[Swap {
                from: main.display().to_string(),
                to: old.display().to_string(),
            }],
        );
        assert_eq!(
            php_env.get("CP_PHP_FWD"),
            Some(old.display().to_string().replace('\\', "/").as_str())
        );

        // The kit uses the newest line on disk: the default, no rewrite.
        assert!(kit_choice(&kit).is_empty());
        apply_kit_choice(&kit, &mut env);
        assert_eq!(env.get("PATH"), base.get("PATH"));

        // A project on 20.18.
        set_project_choice(&kit, "app", "node", Some("20.18")).unwrap();
        let p = project_env(&kit, "app", env.clone());
        assert_eq!(
            p.get("PATH").unwrap(),
            format!(
                "{};{}\\x;{}x;C:\\W",
                old.display(),
                old.display(),
                main.display()
            ),
            "only entries under the folder"
        );
        let file = kit.app_dir().join(r"projects\app\versions.json");
        assert!(fs::read_to_string(&file).unwrap().contains("swaps"));
        let txt = kit.app_dir().join(r"projects\app\versions.txt");
        assert_eq!(
            fs::read_to_string(&txt).unwrap(),
            format!("{}\t{}\n", main.display(), old.display())
        );
        // Back to the kit's: no rewrite, no file.
        set_project_choice(&kit, "app", "node", None).unwrap();
        assert!(!file.exists() && !txt.exists());

        // A newer line downloaded with "+": the kit uses it, and a new
        // project keeps it even when newer ones come.
        let newer = saved_dir(&kit, node, "24.1");
        install_fake(&kit, node, &newer);
        write_version(&newer, "v24.1.0").unwrap();
        assert_eq!(
            kit_choice(&kit).get("node").map(String::as_str),
            Some("24.1")
        );
        pin_newest(&kit, "fresh");
        assert_eq!(
            project_choice(&kit, "fresh")
                .get("node")
                .map(String::as_str),
            Some("24.1")
        );
        // Locked by the project that uses it; free once it chooses another.
        assert!(remove(&kit, "node", "24.1").is_err());
        set_project_choice(&kit, "fresh", "node", None).unwrap();
        remove(&kit, "node", "24.1").unwrap();
        assert!(kit_choice(&kit).is_empty());

        assert!(check("claude", Some("1.0")).is_err());
        assert!(check("node", Some(r"..\x")).is_err());
        // The default goes while another stays: the newest left takes over.
        remove(&kit, "node", "22.11").unwrap();
        assert_eq!(default_line(&kit, node).as_deref(), Some("20.18"));
        assert_eq!(node.installed_version(&kit).as_deref(), Some("v20.18.1"));
        assert_eq!(default_dir(&kit, node), old);
        assert!(saved(&kit, node).is_empty() && !main.exists());
        assert!(
            remove(&kit, "node", "20.18").is_err(),
            "the last goes with the component"
        );
    }

    #[test]
    fn deleted_by_hand_is_no_longer_chosen() {
        let tmp = TempDir::new("versions-sync");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let catalog = Catalog::get();
        let node = catalog.find("node").unwrap();
        let main = install_dir(&kit, node, "v22.11.0");
        install_fake(&kit, node, &main);
        node.save_version(&kit, "v22.11.0").unwrap();
        let mut choice = crate::catalog::Choice::defaults(catalog);
        choice.set(catalog, "pnpm", true);
        choice.write(&kit, catalog).unwrap();
        assert!(sync_with_disk(&kit).unwrap().is_empty(), "all there");

        // The user deletes Node.js: it and pnpm (which needs it) go off.
        fs::remove_dir_all(node.folder_path(&kit).unwrap()).unwrap();
        let off = sync_with_disk(&kit).unwrap();
        assert!(off.contains(&"Node.js".to_owned()) && off.contains(&"pnpm, yarn".to_owned()));
        let now = crate::catalog::Choice::read(&kit, catalog).unwrap();
        assert!(!now.is_chosen("node") && !now.is_chosen("pnpm"));
        assert!(node.installed_version(&kit).is_none());
        assert!(sync_with_disk(&kit).unwrap().is_empty(), "once");

        // Chosen and never installed (waiting for Apply): left alone.
        let mut choice = crate::catalog::Choice::read(&kit, catalog).unwrap();
        choice.set(catalog, "go", true);
        choice.write(&kit, catalog).unwrap();
        assert!(sync_with_disk(&kit).unwrap().is_empty());
    }

    #[test]
    fn offered_lines_and_downloads() {
        let tmp = TempDir::new("versions-offered");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let node = Catalog::get().find("node").unwrap();

        // Nothing installed: nothing to download.
        assert!(!to_download(&kit, "node", "20.18").unwrap());

        let main = install_dir(&kit, node, "v22.11.0");
        install_fake(&kit, node, &main);
        node.save_version(&kit, "v22.11.0").unwrap();
        let old = saved_dir(&kit, node, "20.18");
        install_fake(&kit, node, &old);
        assert!(!to_download(&kit, "node", "22.11").unwrap(), "the default");
        assert!(!to_download(&kit, "node", "20.18").unwrap(), "saved");
        assert!(to_download(&kit, "node", "18.20").unwrap());
        assert!(to_download(&kit, "node", "x").is_err());

        // Kept per component, valid lines only, newest first.
        save_offered(&kit, "node", &["20.18".into(), "24.1".into(), "bad".into()]).unwrap();
        save_offered(&kit, "claude", &["1.0".into()]).unwrap();
        let all = offered(&kit);
        assert_eq!(all.keys().collect::<Vec<_>>(), ["node"]);
        assert_eq!(all["node"].lines, ["24.1", "20.18"]);
        assert!(all["node"].checked > 0);

        crate::projects::settings::update(&kit, "app", |s| {
            s.versions.insert("node".into(), "20.18".into());
        });
        let tool = tools(&kit).into_iter().find(|t| t.id == "node").unwrap();
        assert_eq!(tool.available, ["24.1", "20.18"]);
        assert!(tool.checked.is_some());
        assert_eq!(tool.projects.get("20.18").map(Vec::len), Some(1));
    }

    #[test]
    fn folders_of_an_older_kit_move() {
        let tmp = TempDir::new("versions-migrate");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let node = Catalog::get().find("node").unwrap();
        // Older kit: node\node.exe the default, versions\node\20.18 saved,
        // versions\typst\0.13 of a component no longer versioned.
        let store = node.folder_path(&kit).unwrap();
        install_fake(&kit, node, &store);
        node.save_version(&kit, "v22.11.0").unwrap();
        install_fake(&kit, node, &kit.apps().join(r"versions\node\20.18"));
        fs::create_dir_all(kit.apps().join(r"versions\typst\0.13")).unwrap();
        assert_eq!(default_dir(&kit, node), store, "works before moving");
        assert!(node.is_installed(&kit));
        let conf = kit.data().join(r"apache\kit.conf");
        fs::create_dir_all(conf.parent().unwrap()).unwrap();
        fs::write(&conf, "LoadFile \"${CP_APPS_FWD}/php/php8ts.dll\"\r\n").unwrap();

        migrate(&kit);
        let main = store.join("22.11");
        assert_eq!(default_dir(&kit, node), main);
        assert!(main.join("node.exe").is_file() && !store.join("node.exe").exists());
        assert_eq!(
            fs::read_to_string(main.join(VERSION_FILE)).unwrap(),
            "v22.11.0"
        );
        assert_eq!(saved(&kit, node).len(), 1);
        assert!(has_key(node, &store.join("20.18")));
        assert!(!kit.apps().join("versions").exists());
        assert_eq!(
            fs::read_to_string(&conf).unwrap(),
            "LoadFile \"${CP_PHP_FWD}/php8ts.dll\"\r\n"
        );
        // Nothing left to do the second time.
        migrate(&kit);
        assert_eq!(default_dir(&kit, node), main);
        // The default removed by hand: the newest line left takes over.
        fs::remove_dir_all(&main).unwrap();
        migrate(&kit);
        assert_eq!(default_dir(&kit, node), store.join("20.18"));
        assert!(node.is_installed(&kit));

        // The older kit's Composer, one in tools\: into each folder of PHP.
        let php = Catalog::get().find("php").unwrap();
        let version = "8.4.13 (Composer 2.8.12)";
        let php_dir = install_dir(&kit, php, version);
        install_fake(&kit, php, &php_dir);
        php.save_version(&kit, version).unwrap();
        let tools = kit.apps().join("tools");
        fs::create_dir_all(&tools).unwrap();
        fs::write(tools.join("composer.phar"), b"phar").unwrap();
        assert!(!php.is_installed(&kit), "no Composer in its folder");
        migrate(&kit);
        assert!(php_dir.join("composer.phar").is_file() && php_dir.join("composer.cmd").is_file());
        assert!(!tools.join("composer.phar").exists());
        assert!(php.is_installed(&kit));

        // Interrupted after the first rename: taken back, then moved.
        let python = Catalog::get().find("python").unwrap();
        let py = python.folder_path(&kit).unwrap();
        let mut moving = py.clone().into_os_string();
        moving.push(".atollo-move");
        install_fake(&kit, python, &PathBuf::from(&moving));
        python.save_version(&kit, "3.13.10").unwrap();
        migrate(&kit);
        assert!(has_key(python, &py.join("3.13")));
        assert!(!PathBuf::from(moving).exists());

        assert_eq!(dir_name("20251104 build"), "20251104_build");
    }
}
