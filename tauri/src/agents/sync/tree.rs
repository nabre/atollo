//! Three-way sync of files between the shared copy and one agent (port of
//! `tree.js`). The state of the last sync (size and times of each file on
//! both sides) tells a change from a deletion; when both sides changed the
//! same file the newer wins and the other is saved in `.sync\conflicts`.
//! Writes and deletions never go through a link or junction.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::{alive, read_state, stamp, write_json, Paths, Report};

#[derive(Debug, Clone, Copy)]
pub struct Meta {
    pub size: u64,
    /// Modification time, milliseconds.
    pub mtime: i64,
}

fn meta(path: &Path) -> Option<Meta> {
    let m = fs::metadata(path).ok()?;
    let mtime = m
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64);
    Some(Meta {
        size: m.len(),
        mtime,
    })
}

/// Relative paths (with `/`) of the files under `dir`. Top-level names
/// starting with a dot are skipped (Codex `.system`...), and so are links;
/// temporary files `<name>.tmp-<pid>` of a sync that died are removed.
pub fn list_files(dir: &Path) -> BTreeMap<String, Meta> {
    let mut out = BTreeMap::new();
    let mut stack = vec![(dir.to_path_buf(), String::new())];
    while let Some((abs, rel)) = stack.pop() {
        let Ok(entries) = fs::read_dir(&abs) else {
            continue;
        };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if rel.is_empty() && name.starts_with('.') {
                continue;
            }
            let r = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_symlink() || is_reparse(&e.path()) {
                continue;
            }
            if let Some(pid) = name
                .rsplit_once(".tmp-")
                .and_then(|(_, p)| p.parse::<u32>().ok())
            {
                if ft.is_file() && pid != std::process::id() && !alive(pid) {
                    let _ = fs::remove_file(e.path());
                }
                continue;
            }
            if ft.is_dir() {
                stack.push((e.path(), r));
            } else if ft.is_file() {
                if let Some(m) = meta(&e.path()) {
                    out.insert(r, m);
                }
            }
        }
    }
    out
}

/// A symbolic link (Unix has no junctions).
#[cfg(not(windows))]
pub fn is_reparse(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// A junction or symbolic link (Rust's `is_symlink` misses junctions).
#[cfg(windows)]
pub fn is_reparse(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    fs::symlink_metadata(path)
        .is_ok_and(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
}

fn same_content(a: &Path, b: &Path) -> bool {
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(x), Ok(y)) if x.len() == y.len() => fs::read(a).ok() == fs::read(b).ok(),
        _ => false,
    }
}

/// Copies keeping the modification time, through `<to>.tmp-<pid>`.
pub fn copy_file(from: &Path, to: &Path) -> Result<()> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = PathBuf::from(format!("{}.tmp-{}", to.display(), std::process::id()));
    let _ = fs::remove_file(&tmp);
    fs::copy(from, &tmp).with_context(|| from.display().to_string())?;
    let mtime = fs::metadata(from)?.modified()?;
    let f = fs::OpenOptions::new().write(true).open(&tmp)?;
    f.set_modified(mtime)?;
    drop(f);
    fs::rename(&tmp, to).with_context(|| to.display().to_string())?;
    Ok(())
}

fn remove_file(file: &Path, top: &Path) -> Result<()> {
    match fs::remove_file(file) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
        _ => {}
    }
    // Folders left empty, up to (excluded) the synced folder.
    let mut d = file.parent().map(Path::to_path_buf);
    while let Some(dir) = d {
        if dir == top || !dir.starts_with(top) || fs::remove_dir(&dir).is_err() {
            break;
        }
        d = dir.parent().map(Path::to_path_buf);
    }
    Ok(())
}

/// True if no folder between `top` (excluded) and `file` is a link or
/// junction: writing or deleting `file` then stays inside `top`.
pub fn confined(top: &Path, file: &Path) -> bool {
    let Some(parent) = file.parent() else {
        return false;
    };
    let Ok(rel) = parent.strip_prefix(top) else {
        return false;
    };
    let mut d = top.to_path_buf();
    for c in rel.components() {
        let Component::Normal(part) = c else {
            return false;
        };
        d.push(part);
        match fs::symlink_metadata(&d) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
            Err(_) => return false,
            Ok(m) if !m.is_dir() || is_reparse(&d) => return false,
            Ok(_) => {}
        }
    }
    true
}

fn save_conflict(p: &Paths, file: &Path, label: &str, report: &mut Report) -> Result<()> {
    let dest = p.state.join("conflicts").join(format!(
        "{}-{}",
        stamp(),
        label.replace(['\\', '/', ':'], "_")
    ));
    copy_file(file, &dest)?;
    report.warn(format!(
        "{label} changed on both sides: keeping the newer, the other is in {}",
        dest.strip_prefix(&p.root).unwrap_or(&dest).display()
    ));
    Ok(())
}

/// `master`/`replica`: two folders (`is_dir`) or two files. `key`: name of
/// the state file in `.sync`.
pub fn sync_tree(
    p: &Paths,
    master: &Path,
    replica: &Path,
    key: &str,
    is_dir: bool,
    label: &str,
    report: &mut Report,
) -> Result<()> {
    let state_file = p.state.join(format!("{key}.json"));
    let base = read_state(&state_file);
    let single = |abs: &Path| -> BTreeMap<String, Meta> {
        meta(abs)
            .filter(|_| abs.is_file())
            .map(|m| BTreeMap::from([(String::new(), m)]))
            .unwrap_or_default()
    };
    let m = if is_dir {
        list_files(master)
    } else {
        single(master)
    };
    let r = if is_dir {
        list_files(replica)
    } else {
        single(replica)
    };
    let abs = |top: &Path, rel: &str| {
        if rel.is_empty() {
            top.to_path_buf()
        } else {
            top.join(rel.replace('/', "\\"))
        }
    };
    let name = |rel: &str| {
        if rel.is_empty() {
            label.to_owned()
        } else {
            format!("{label}/{rel}")
        }
    };
    let changed = |st: &Meta, b: Option<&Value>, side: &str| match b {
        None => true,
        Some(b) => {
            b.get("size").and_then(Value::as_u64) != Some(st.size)
                || b.get(side).and_then(Value::as_i64) != Some(st.mtime)
        }
    };
    let guard = |file: &Path, top: &Path, report: &mut Report| {
        if !is_dir || confined(top, file) {
            true
        } else {
            report.warn(format!(
                "{}: folder linked elsewhere, skipped",
                file.display()
            ));
            false
        }
    };

    let keys: BTreeSet<&String> = m.keys().chain(r.keys()).collect();
    for rel in keys {
        let (ma, ra) = (abs(master, rel), abs(replica, rel));
        let b = base.get(rel.as_str());
        match (m.get(rel), r.get(rel)) {
            (Some(ms), Some(rs)) => {
                let (mc, rc) = (changed(ms, b, "m"), changed(rs, b, "r"));
                if (!mc && !rc) || same_content(&ma, &ra) {
                    continue;
                }
                let replica_wins = rc && (!mc || rs.mtime > ms.mtime);
                if mc && rc {
                    save_conflict(p, if replica_wins { &ma } else { &ra }, &name(rel), report)?;
                }
                if replica_wins {
                    if guard(&ma, master, report) {
                        copy_file(&ra, &ma)?;
                        report.note(format!("{} <- {}", name(rel), replica.display()));
                    }
                } else if guard(&ra, replica, report) {
                    copy_file(&ma, &ra)?;
                    report.note(format!("{} -> {}", name(rel), replica.display()));
                }
            }
            (Some(ms), None) => {
                // Only in the shared copy: deleted on the agent's side, or new.
                if b.is_some() && !changed(ms, b, "m") {
                    if guard(&ma, master, report) {
                        remove_file(&ma, master)?;
                        report.note(format!("{} deleted", name(rel)));
                    }
                } else {
                    if b.is_some() {
                        report.warn(format!(
                            "{} deleted in {} but changed elsewhere: restored",
                            name(rel),
                            replica.display()
                        ));
                    }
                    if guard(&ra, replica, report) {
                        copy_file(&ma, &ra)?;
                    }
                }
            }
            (None, Some(rs)) => {
                if b.is_some() && !changed(rs, b, "r") {
                    if guard(&ra, replica, report) {
                        remove_file(&ra, replica)?;
                        report.note(format!("{} deleted in {}", name(rel), replica.display()));
                    }
                } else {
                    if b.is_some() {
                        report.warn(format!(
                            "{} deleted elsewhere but changed in {}: restored",
                            name(rel),
                            replica.display()
                        ));
                    }
                    if guard(&ma, master, report) {
                        copy_file(&ra, &ma)?;
                    }
                }
            }
            (None, None) => {}
        }
    }

    // State after the sync: the files are now equal on both sides.
    let m2 = if is_dir {
        list_files(master)
    } else {
        single(master)
    };
    let r2 = if is_dir {
        list_files(replica)
    } else {
        single(replica)
    };
    let mut now = serde_json::Map::new();
    for (rel, ms) in &m2 {
        if let Some(rs) = r2.get(rel) {
            now.insert(
                rel.clone(),
                json!({ "size": ms.size, "m": ms.mtime, "r": rs.mtime }),
            );
        }
    }
    write_json(&state_file, &Value::Object(now))
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/sync/tree.rs"]
mod tests;
