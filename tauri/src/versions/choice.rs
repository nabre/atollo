//! The lines chosen by the kit and by each project, applied by rewriting
//! the kit environment and written for the editor's terminals.

use std::collections::BTreeMap;
use std::fs;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::{default_line, dir_for, find, is_present, saved, valid_line, versioned};
use crate::common::fsx::write_atomic;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::setup::Component;

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

/// The rewrites from the folders of `base` to those of `choice`. PHP's
/// line also moves `PHPRC` to that line's `php.ini` (`data\php\<line>`).
fn swaps(kit: &Kit, base: &Choice, choice: &Choice) -> Vec<Swap> {
    let swap = |from: &std::path::Path, to: &std::path::Path| Swap {
        from: from.display().to_string(),
        to: to.display().to_string(),
    };
    let data = kit.data();
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
            (from != to).then(|| {
                let mut out = vec![swap(&from, &to)];
                if c.id == "php" {
                    let (ini_from, ini_to) = (
                        crate::install::php_ini_dir(&data, &from),
                        crate::install::php_ini_dir(&data, &to),
                    );
                    if ini_from != ini_to {
                        out.push(swap(&ini_from, &ini_to));
                    }
                }
                // Off Windows each Ruby line has its own GEM_HOME (and its bin
                // on PATH, moved by the same swap since it sits under it).
                if c.id == "ruby" {
                    let target = kit.target();
                    if let (Some(a), Some(b)) = (
                        crate::env::gem_home_for_dir(&data, target, &from),
                        crate::env::gem_home_for_dir(&data, target, &to),
                    ) {
                        if a != b {
                            out.push(swap(&a, &b));
                        }
                    }
                }
                out
            })
        })
        .flatten()
        .collect()
}

/// `value` with every entry (`sep`-separated) under `from` moved under `to`.
fn rewrite_value(value: &str, swaps: &[Swap], sep: char) -> String {
    let mut sep_buf = [0u8; 4];
    let sep_str = sep.encode_utf8(&mut sep_buf);
    crate::env::split_path(value, sep)
        .into_iter()
        .map(|entry| {
            for s in swaps {
                let lower = entry.to_ascii_lowercase();
                let from = s.from.to_ascii_lowercase();
                if lower == from
                    || lower.starts_with(&format!("{from}\\"))
                    || lower.starts_with(&format!("{from}/"))
                {
                    return format!("{}{}", s.to, &entry[s.from.len()..]);
                }
            }
            entry.to_owned()
        })
        .collect::<Vec<_>>()
        .join(sep_str)
}

fn apply(env: &mut KitEnv, swaps: &[Swap]) {
    if swaps.is_empty() {
        return;
    }
    let sep = env.path_sep();
    let changed: Vec<(String, String)> = env
        .iter()
        .filter_map(|(k, v)| {
            let new = rewrite_value(v, swaps, sep);
            (new != v).then(|| (k.to_owned(), new))
        })
        .collect();
    for (k, v) in changed {
        env.set(&k, v);
    }
    // Apache's PHP (`/` paths, not rewritten above) and the web servers
    // follow the version in use.
    for (from, to) in [
        ("CP_PHP", "CP_PHP_FWD"),
        ("PHPRC", "CP_PHPRC_FWD"),
        ("CP_APACHE", "CP_APACHE_FWD"),
    ] {
        if let Some(p) = env.get(from).map(|p| p.replace('\\', "/")) {
            env.set(to, p);
        }
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

/// `env`, on the lines `lines` (the kit's, [`kit_choice`], or a kit
/// terminal's on top of them), moved to the lines of `choice` (cleaned);
/// the lines it is on afterwards: `lines` with those of `choice` that are
/// on disk (a line not on disk leaves the tool where it was). The steps 2
/// and 3 of `levels::env::pc_environment` (LEVELS.md §3.5).
pub fn swap_lines(kit: &Kit, env: &mut KitEnv, lines: &Choice, choice: &Choice) -> Choice {
    let choice = clean(choice.clone());
    apply(env, &swaps(kit, lines, &choice));
    let mut now = lines.clone();
    for (id, line) in choice {
        if find(&id).is_some_and(|c| is_present(kit, c, &line)) {
            now.insert(id, line);
        }
    }
    now
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
    let Some(state) = crate::projects::state::ProjectState::of(kit, project) else {
        return;
    };
    let dir = state.dir();
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
    let result = fs::create_dir_all(dir)
        .and_then(|()| write_atomic(&json, text.as_bytes()))
        .and_then(|()| write_atomic(&txt, lines.as_bytes()));
    if let Err(e) = result {
        crate::common::log::warn(&format!("versions of {project}: {e}"));
    }
}

/// Every project's `versions.json`, after a change of the kit's lines or of
/// what is installed.
pub fn write_project_files(kit: &Kit) {
    for project in crate::projects::state::all(kit) {
        write_project_file(kit, &project);
    }
}

#[cfg(test)]
#[path = "../../../test/rust/versions/choice.rs"]
mod tests;
