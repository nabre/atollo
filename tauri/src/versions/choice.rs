//! The lines chosen by the kit and by each project, applied by rewriting
//! the kit environment and written for the editor's terminals.

use std::collections::BTreeMap;
use std::fs;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::{default_line, dir_for, find, is_present, saved, valid_line, versioned};
use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::setup::catalog::Component;

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
    // Apache's PHP (`/` paths, not rewritten above) and the web servers
    // follow the version in use.
    for (from, to) in [
        ("CP_PHP", "CP_PHP_FWD"),
        ("CP_APACHE", "CP_APACHE_FWD"),
        ("CP_NGINX", "CP_NGINX_FWD"),
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
        crate::log::warn(&format!("versions of {project}: {e}"));
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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};
    use crate::setup::catalog::Catalog;
    use crate::versions::tests::install_fake;
    use crate::versions::{default_dir, install_dir, remove, saved_dir, write_version, Line};

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
            crate::setup::catalog::kit_path(&kit, "node/node.exe"),
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
        let file = kit.project_data().join(r"app\versions.json");
        assert!(fs::read_to_string(&file).unwrap().contains("swaps"));
        let txt = kit.project_data().join(r"app\versions.txt");
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
}
