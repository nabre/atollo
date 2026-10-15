//! Saved versions of the languages and tools (DECISIONS.md D-80).
//!
//! The component's own folder (`data\apps\<platform>\node`) stays the kit's
//! default and is what updates replace. Other versions are kept one per
//! minor line (`major.minor`) in `data\apps\<platform>\versions\<id>\<line>`,
//! with the full version in `.devkit-version`:
//! - an update that changes the minor line moves the old folder there
//!   instead of replacing it (a new patch of the same line still replaces);
//! - a line can be downloaded there from the Tools page;
//! - the preferred line of the kit is in `data\app\tool-versions.json`, the
//!   one of a project in its `dev-settings.json` (`versions`).
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
use crate::kit::{write_atomic, Kit};

/// The file with the full version, inside a saved folder.
const VERSION_FILE: &str = ".devkit-version";

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

/// `data\apps\<platform>\versions\<id>`.
fn store(kit: &Kit, c: &Component) -> PathBuf {
    kit.apps().join("versions").join(&c.id)
}

/// Where `line` of `c` is (or goes) when it is not the default.
pub fn saved_dir(kit: &Kit, c: &Component, line: &str) -> PathBuf {
    store(kit, c).join(line)
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

/// The saved lines of `c`, newest first.
pub fn saved(kit: &Kit, c: &Component) -> Vec<Line> {
    let mut out: Vec<Line> = fs::read_dir(store(kit, c))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let line = e.file_name().to_string_lossy().into_owned();
            (valid_line(&line) && has_key(c, &e.path())).then(|| Line {
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
    let main = c.folder_path(kit)?;
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

fn prefs_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("tool-versions.json")
}

/// The lines preferred for the whole kit.
pub fn kit_choice(kit: &Kit) -> Choice {
    fs::read_to_string(prefs_file(kit))
        .ok()
        .and_then(|t| serde_json::from_str::<Choice>(t.trim_start_matches('\u{feff}')).ok())
        .map(clean)
        .unwrap_or_default()
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

pub fn set_kit_choice(kit: &Kit, id: &str, line: Option<&str>) -> Result<()> {
    check(id, line)?;
    let mut choice = kit_choice(kit);
    match line {
        Some(l) => choice.insert(id.to_owned(), l.to_owned()),
        None => choice.remove(id),
    };
    let text = serde_json::to_string_pretty(&choice)?;
    write_atomic(&prefs_file(kit), text.as_bytes())?;
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
}

/// The kit's preferred lines, applied to an environment built on the
/// default folders.
pub fn apply_kit_choice(kit: &Kit, env: &mut KitEnv) {
    apply(env, &swaps(kit, &Choice::new(), &kit_choice(kit)));
}

/// The lines chosen in `project`.
pub fn project_choice(kit: &Kit, project: &str) -> Choice {
    clean(crate::project_settings::load(kit, project).versions)
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
    crate::project_settings::update(kit, project, |s| {
        match line {
            Some(l) => s.versions.insert(id.to_owned(), l.to_owned()),
            None => s.versions.remove(id),
        };
    });
    write_project_file(kit, project);
    Ok(())
}

/// `versions.json` of `project` for the editor's terminals (profile.ps1);
/// removed when the project has no rewrite.
pub fn write_project_file(kit: &Kit, project: &str) {
    let Some(dir) = crate::project_settings::dir(kit, project) else {
        return;
    };
    let file = dir.join("versions.json");
    let swaps = project_swaps(kit, project);
    if swaps.is_empty() {
        let _ = fs::remove_file(file);
        return;
    }
    let text =
        serde_json::to_string_pretty(&serde_json::json!({ "swaps": swaps })).unwrap_or_default();
    let result = fs::create_dir_all(&dir).and_then(|()| write_atomic(&file, text.as_bytes()));
    if let Err(e) = result {
        crate::log::warn(&format!("versions of {project}: {e}"));
    }
}

/// Every project's `versions.json`, after a change of the kit's lines or of
/// what is installed.
pub fn write_project_files(kit: &Kit) {
    let projects = kit.app_dir().join("projects");
    for entry in fs::read_dir(projects).into_iter().flatten().flatten() {
        write_project_file(kit, &entry.file_name().to_string_lossy());
    }
}

/// Removes a saved line (never the default folder).
pub fn remove(kit: &Kit, id: &str, line: &str) -> Result<()> {
    check(id, Some(line))?;
    let Some(c) = find(id) else { return Ok(()) };
    crate::archive::remove_dir(&saved_dir(kit, c, line))
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
    /// The kit's preferred line; `None` = the default folder.
    pub preferred: Option<String>,
    /// Lines can be downloaded (the installer knows the older ones).
    pub downloadable: bool,
}

pub fn tools(kit: &Kit) -> Vec<Tool> {
    let choice = kit_choice(kit);
    versioned()
        .map(|c| Tool {
            id: c.id.clone(),
            name: c.name.clone(),
            installed: c.is_installed(kit),
            default: default_line(kit, c).map(|line| Line {
                line,
                version: c.installed_version(kit),
            }),
            saved: saved(kit, c),
            preferred: choice.get(&c.id).cloned(),
            downloadable: crate::install::installer(&c.id).is_some_and(|i| i.has_lines()),
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
        let main = node.folder_path(&kit).unwrap();
        install_fake(&kit, node, &main);
        node.save_version(&kit, "v22.11.0").unwrap();
        assert_eq!(default_line(&kit, node).as_deref(), Some("22.11"));
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

        // The kit prefers 20.18.
        set_kit_choice(&kit, "node", Some("20.18")).unwrap();
        apply_kit_choice(&kit, &mut env);
        assert_eq!(
            env.get("PATH").unwrap(),
            format!(
                "{};{}\\x;{}x;C:\\W",
                old.display(),
                old.display(),
                main.display()
            ),
            "only entries under the folder"
        );
        // A project goes back to the default.
        crate::project_settings::update(&kit, "app", |s| {
            s.versions.insert("node".into(), "22.11".into());
        });
        let p = project_env(&kit, "app", env.clone());
        assert_eq!(p.get("PATH"), base.get("PATH"));
        write_project_file(&kit, "app");
        let file = kit.app_dir().join(r"projects\app\versions.json");
        assert!(fs::read_to_string(&file).unwrap().contains("swaps"));
        // Same as the kit: no rewrite, no file.
        set_project_choice(&kit, "app", "node", Some("20.18")).unwrap();
        assert!(!file.exists());

        assert!(set_kit_choice(&kit, "claude", Some("1.0")).is_err());
        assert!(set_kit_choice(&kit, "node", Some(r"..\x")).is_err());
        set_kit_choice(&kit, "node", None).unwrap();
        assert!(kit_choice(&kit).is_empty());
        remove(&kit, "node", "20.18").unwrap();
        assert!(saved(&kit, node).is_empty());
    }
}
