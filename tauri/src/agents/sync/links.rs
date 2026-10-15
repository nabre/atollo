//! Global skills: one junction per skill from each agent's skills folder to
//! `data\agents\skills`, the only real copy (port of `links.js`).
//!
//! At every sync:
//! - a junction removed by the agent since the last sync (skill deleted
//!   there) moves the skill to `.sync\deleted` (recoverable);
//! - a real folder with `SKILL.md` (made or installed by the agent) moves to
//!   `data\agents\skills` and becomes a junction; if one with that name and
//!   different content exists, the older is kept in `.sync\conflicts`;
//! - a junction pointing elsewhere (the kit moved: junctions keep absolute
//!   paths) or to a skill that no longer exists is made again or removed.
//!
//! A disk without junctions (FAT32, exFAT): `link_skills` returns false and
//! the caller copies the skills instead.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

use super::tree::{is_reparse, list_files};
use super::{read_state, stamp, write_json, Paths, Report, Tool};

fn same(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| {
        let s = p.display().to_string();
        s.trim_start_matches(r"\\?\")
            .trim_end_matches(['\\', '/'])
            .to_lowercase()
    };
    norm(a) == norm(b)
}

fn can_link(dir: &Path, target: &Path) -> bool {
    let probe = dir.join(format!(".atollo-link-{}", std::process::id()));
    let _ = junction::delete(&probe);
    let _ = fs::remove_dir(&probe);
    let ok = junction::create(target, &probe).is_ok();
    let _ = junction::delete(&probe);
    let _ = fs::remove_dir(&probe);
    ok
}

fn same_tree(a: &Path, b: &Path) -> bool {
    let (fa, fb) = (list_files(a), list_files(b));
    fa.len() == fb.len()
        && fa.iter().all(|(rel, st)| {
            fb.get(rel).is_some_and(|o| o.size == st.size)
                && fs::read(a.join(rel.replace('/', "\\"))).ok()
                    == fs::read(b.join(rel.replace('/', "\\"))).ok()
        })
}

fn newest(dir: &Path) -> i64 {
    list_files(dir).values().map(|m| m.mtime).max().unwrap_or(0)
}

/// Moved under `.sync\<kind>\`, never deleted.
fn keep(p: &Paths, dir: &Path, kind: &str, label: &str) -> Result<PathBuf> {
    let dest = p.state.join(kind).join(format!("{}-{label}", stamp()));
    fs::create_dir_all(dest.parent().unwrap_or(&p.state))?;
    fs::rename(dir, &dest)?;
    Ok(dest)
}

fn adopt(p: &Paths, tool: &Tool, name: &str, dir: &Path, report: &mut Report) -> Result<()> {
    let shared_root = p.agents.join("skills");
    let shared = shared_root.join(name);
    if !shared.exists() {
        fs::create_dir_all(&shared_root)?;
        fs::rename(dir, &shared)?;
        report.note(format!(
            "skill {name} of {} moved to data\\agents\\skills",
            tool.name
        ));
    } else if same_tree(dir, &shared) {
        fs::remove_dir_all(dir)?;
    } else if newest(dir) > newest(&shared) {
        let kept = keep(p, &shared, "conflicts", name)?;
        fs::rename(dir, &shared)?;
        report.warn(format!("skill {name} differs in {} and data\\agents: keeping the newer ({}), the other is in {}", tool.name, tool.name, kept.display()));
    } else {
        let kept = keep(p, dir, "conflicts", &format!("{}-{name}", tool.id))?;
        report.warn(format!("skill {name} differs in {} and data\\agents: keeping the newer (data\\agents), the other is in {}", tool.name, kept.display()));
    }
    Ok(())
}

fn state_file(p: &Paths, tool: &Tool) -> PathBuf {
    p.state.join(format!("links-{}.json", tool.id))
}

/// First pass: skills deleted from an agent since the last sync.
pub fn remove_deleted(p: &Paths, tool: &Tool, report: &mut Report) -> Result<()> {
    let state = read_state(&state_file(p, tool));
    let before: Vec<String> = state
        .get("linked")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    for name in before {
        let shared = p.agents.join("skills").join(&name);
        if fs::symlink_metadata(tool.skills.join(&name)).is_ok() || !shared.exists() {
            continue;
        }
        let kept = keep(p, &shared, "deleted", &name)?;
        report.note(format!(
            "skill {name} deleted by {}: moved to {}",
            tool.name,
            kept.display()
        ));
    }
    Ok(())
}

/// Second pass: every shared skill linked in the agent's folder. `Ok(false)`
/// on a disk without junctions.
pub fn link_skills(p: &Paths, tool: &Tool, report: &mut Report) -> Result<bool> {
    let dir = &tool.skills;
    let shared_root = p.agents.join("skills");
    fs::create_dir_all(dir)?;
    fs::create_dir_all(&shared_root)?;
    if !can_link(dir, &shared_root) {
        let _ = fs::remove_file(state_file(p, tool));
        report.note(format!(
            "{}: the disk has no junctions, skills copied",
            dir.display()
        ));
        return Ok(false);
    }
    for e in fs::read_dir(dir)?.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let path = e.path();
        let shared = shared_root.join(&name);
        if is_reparse(&path) {
            let target = junction::get_target(&path).ok();
            if shared.exists() && target.as_deref().is_some_and(|t| same(t, &shared)) {
                continue;
            }
            let _ = junction::delete(&path);
            let _ = fs::remove_dir(&path);
        } else if path.is_dir() && path.join("SKILL.md").is_file() {
            adopt(p, tool, &name, &path, report)?;
        }
    }
    let mut linked = Vec::new();
    let mut names: Vec<String> = fs::read_dir(&shared_root)?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect();
    names.sort();
    for name in names {
        let link = dir.join(&name);
        if fs::symlink_metadata(&link).is_err() {
            if let Err(e) = junction::create(shared_root.join(&name), &link) {
                report.warn(format!("{}: link failed ({e})", link.display()));
                continue;
            }
        } else if !is_reparse(&link) {
            report.warn(format!(
                "{} exists and is not a skill: skill {name} not linked",
                link.display()
            ));
            continue;
        }
        linked.push(name);
    }
    write_json(&state_file(p, tool), &json!({ "linked": linked }))?;
    // State of the copies (a disk without links earlier).
    let _ = fs::remove_file(p.state.join(format!("{}.json", tool.skills_key)));
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::sync::tests::kit;
    use crate::agents::sync::{run, tools};

    #[test]
    fn skills_become_junctions_and_deletions_are_kept() {
        let (_t, kit, p) = kit("links");
        let mine = p.codex.join(r"skills\review");
        fs::create_dir_all(&mine).unwrap();
        fs::write(mine.join("SKILL.md"), "review").unwrap();
        fs::create_dir_all(p.codex.join(r"skills\.system")).unwrap();
        let r = run(&kit, None);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
        assert_eq!(
            fs::read_to_string(p.agents.join(r"skills\review\SKILL.md")).unwrap(),
            "review"
        );
        for t in tools(&p) {
            let link = t.skills.join("review");
            assert!(is_reparse(&link), "{} linked", t.name);
            assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), "review");
        }
        assert!(
            p.codex.join(r"skills\.system").is_dir(),
            "dot folders untouched"
        );

        // Claude deletes it: kept in .sync\deleted, links gone everywhere.
        junction::delete(p.claude.join(r"skills\review")).unwrap();
        fs::remove_dir(p.claude.join(r"skills\review")).unwrap();
        run(&kit, None);
        assert!(!p.agents.join(r"skills\review").exists());
        assert_eq!(fs::read_dir(p.state.join("deleted")).unwrap().count(), 1);
        assert!(fs::symlink_metadata(p.codex.join(r"skills\review")).is_err());
    }

    #[test]
    fn junction_to_old_place_is_remade() {
        let (t, kit, p) = kit("links-moved");
        fs::create_dir_all(p.agents.join(r"skills\s1")).unwrap();
        fs::write(p.agents.join(r"skills\s1\SKILL.md"), "x").unwrap();
        let old = t.0.join("old-kit-skill");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(p.claude.join("skills")).unwrap();
        junction::create(&old, p.claude.join(r"skills\s1")).unwrap();
        run(&kit, None);
        let target = junction::get_target(p.claude.join(r"skills\s1")).unwrap();
        assert!(
            same(&target, &p.agents.join(r"skills\s1")),
            "{}",
            target.display()
        );
    }
}
