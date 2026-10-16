//! The kit moved (another drive letter or folder): the old root, written by
//! the agents, the editor and the Python tools into their files, is replaced
//! with the current one (port of nabre/dev-kit `setup/agents/moved.js`).
//!
//! `kit.rs` notices the move at start and saves the old root in
//! `data\app\old-root.txt`; it is removed here once the paths are updated,
//! so a move is never lost if this step does not run at that start.
//! Spellings replaced: `E:\kit`, `E:\\kit` (JSON), `E:/kit`, and
//! `file:///e%3A/kit` URIs, only as a whole path, keeping the drive letter's
//! case. Claude's per-folder data (`.claude\projects\E--kit-...`) is renamed
//! (merged if the new name exists). Programs (a NUL byte inside) are changed
//! only when the new root has the same length; otherwise they are named in
//! a warning, to be reinstalled or recreated.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::bytes::{Captures, Regex};
use serde_json::Value;

use super::sync::{read_json, write_json, Paths, Report};
use crate::kit::Kit;

const TEXT_MAX: u64 = 8 * 1024 * 1024;
const BINARY_MAX: u64 = 32 * 1024 * 1024;

fn uri_path(p: &str) -> String {
    let p = p.replace('\\', "/");
    let mut parts: Vec<&str> = p.split('/').collect();
    let mut out = String::new();
    if parts
        .first()
        .is_some_and(|d| d.len() == 2 && d.ends_with(':') && d.as_bytes()[0].is_ascii_alphabetic())
    {
        let d = parts.remove(0);
        out.push_str(&format!("/{}%3A/", d[..1].to_ascii_lowercase()));
    }
    let enc = |s: &str| {
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z'
                | b'a'..=b'z'
                | b'0'..=b'9'
                | b'-'
                | b'_'
                | b'.'
                | b'~'
                | b'!'
                | b'*'
                | b'\''
                | b'('
                | b')' => (b as char).to_string(),
                _ => format!("%{b:02X}"),
            })
            .collect::<String>()
    };
    out.push_str(&parts.iter().map(|s| enc(s)).collect::<Vec<_>>().join("/"));
    out
}

/// Old and new root in every spelling, longest first.
pub struct Replacer {
    re: Regex,
    pairs: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Replacer {
    pub fn new(from: &str, to: &str) -> Replacer {
        let mut pairs: Vec<(String, String)> = vec![
            (from.to_owned(), to.to_owned()),
            (uri_path(from), uri_path(to)),
            (from.replace('\\', "/"), to.replace('\\', "/")),
            (from.replace('\\', "\\\\"), to.replace('\\', "\\\\")),
        ];
        let mut seen = std::collections::HashSet::new();
        pairs.retain(|(a, _)| seen.insert(a.to_lowercase()));
        pairs.sort_by_key(|(a, _)| std::cmp::Reverse(a.len()));
        let alts: Vec<String> = pairs
            .iter()
            .map(|(a, _)| format!("({})", regex::escape(a)))
            .collect();
        // (?-u) byte mode: files are read as bytes; no look-around in the
        // regex crate, so the boundaries are checked by hand in `replace`.
        let re = Regex::new(&format!("(?i){}", alts.join("|")))
            .unwrap_or_else(|_| Regex::new("$^").unwrap_or_else(|_| unreachable!()));
        Replacer {
            re,
            pairs: pairs
                .into_iter()
                .map(|(a, b)| (a.into_bytes(), b.into_bytes()))
                .collect(),
        }
    }

    fn boundary(b: Option<&u8>) -> bool {
        !b.is_some_and(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-'))
    }

    pub fn replace(&self, text: &[u8]) -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(text.len());
        let mut last = 0;
        let mut changed = false;
        for caps in self.re.captures_iter(text) {
            let Some(m) = caps.get(0) else { continue };
            if !Self::boundary(m.start().checked_sub(1).and_then(|i| text.get(i)))
                || !Self::boundary(text.get(m.end()))
            {
                continue;
            }
            out.extend_from_slice(&text[last..m.start()]);
            out.extend_from_slice(&self.substitute(&caps, m.as_bytes()));
            last = m.end();
            changed = true;
        }
        if !changed {
            return None;
        }
        out.extend_from_slice(&text[last..]);
        Some(out)
    }

    /// The new spelling, with the drive letter in the case found.
    fn substitute(&self, caps: &Captures, found: &[u8]) -> Vec<u8> {
        let i = (1..caps.len())
            .find(|i| caps.get(*i).is_some())
            .unwrap_or(1)
            - 1;
        let mut new = self.pairs[i].1.clone();
        let at = usize::from(new.first() == Some(&b'/'));
        if let (Some(f), Some(n)) = (found.get(at), new.get_mut(at)) {
            if f.is_ascii_alphabetic() && n.is_ascii_alphabetic() {
                *n = if f.is_ascii_lowercase() {
                    n.to_ascii_lowercase()
                } else {
                    n.to_ascii_uppercase()
                };
            }
        }
        new
    }

    fn has_length_change(&self, text: &[u8]) -> bool {
        let lower = text.to_ascii_lowercase();
        self.pairs.iter().any(|(a, b)| {
            a.len() != b.len()
                && lower
                    .windows(a.len())
                    .any(|w| w == a.to_ascii_lowercase().as_slice())
        })
    }
}

#[derive(Default)]
struct Moved {
    changed: usize,
    programs: Vec<PathBuf>,
    failed: Vec<String>,
}

fn relocate_file(file: &Path, r: &Replacer, report: &mut Moved) {
    let Ok(meta) = fs::metadata(file) else { return };
    if !meta.is_file() {
        return;
    }
    let Ok(bytes) = fs::read(file) else { return };
    let binary = bytes.iter().take(8192).any(|b| *b == 0);
    if meta.len() > if binary { BINARY_MAX } else { TEXT_MAX } {
        return;
    }
    let Some(mut out) = r.replace(&bytes) else {
        return;
    };
    if binary && r.has_length_change(&bytes) {
        report.programs.push(file.to_path_buf());
        return;
    }
    if file.file_name().is_some_and(|n| n == "config.toml") {
        out = dedupe_toml(&String::from_utf8_lossy(&out)).into_bytes();
    }
    let readonly = meta.permissions().readonly();
    let result = (|| -> std::io::Result<()> {
        if readonly {
            let mut p = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            p.set_readonly(false);
            fs::set_permissions(file, p)?;
        }
        fs::write(file, &out)?;
        if readonly {
            fs::set_permissions(file, meta.permissions())?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => report.changed += 1,
        Err(e) => report.failed.push(format!("{} ({e})", file.display())),
    }
}

/// Codex refuses the same `[projects.x]` table twice: after the change an
/// old key may equal one already there; the later one goes.
fn dedupe_toml(text: &str) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut skip = false;
    text.split_inclusive('\n')
        .filter(|line| {
            let t = line.trim();
            if let Some(key) = t
                .strip_prefix("[projects.")
                .and_then(|r| r.strip_suffix(']'))
            {
                skip = !seen.insert(key.trim().to_lowercase());
            } else if t.starts_with('[') || t.starts_with("# >>>") || t.starts_with("# <<<") {
                skip = false;
            }
            !skip
        })
        .collect()
}

/// Claude's settings: keys of "projects" are folders. An old key that now
/// equals one already written at the new place is merged into it (the new
/// one wins). Written only if Claude did not save meanwhile.
fn relocate_claude_json(p: &Paths, r: &Replacer, report: &mut Moved) -> Result<()> {
    let file = p.claude_json();
    let Some(cfg) = read_json(&file)? else {
        return Ok(());
    };
    let mut changed = false;
    fn rep(r: &Replacer, s: &str, changed: &mut bool) -> String {
        match r.replace(s.as_bytes()) {
            Some(b) => {
                *changed = true;
                String::from_utf8_lossy(&b).into_owned()
            }
            None => s.to_owned(),
        }
    }
    fn walk(v: &Value, r: &Replacer, changed: &mut bool) -> Value {
        match v {
            Value::String(s) => Value::String(rep(r, s, changed)),
            Value::Array(a) => Value::Array(a.iter().map(|x| walk(x, r, changed)).collect()),
            Value::Object(o) => {
                let mut out = serde_json::Map::new();
                for (k, val) in o {
                    let mut moved = false;
                    let key = rep(r, k, &mut moved);
                    *changed |= moved;
                    let value = walk(val, r, changed);
                    match out.remove(&key) {
                        None => {
                            out.insert(key, value);
                        }
                        Some(existing) => {
                            let (older, newer) = if moved {
                                (value, existing)
                            } else {
                                (existing, value)
                            };
                            let merged = match (older, newer) {
                                (Value::Object(mut a), Value::Object(b)) => {
                                    a.extend(b);
                                    Value::Object(a)
                                }
                                (_, n) => n,
                            };
                            out.insert(key, merged);
                        }
                    }
                }
                Value::Object(out)
            }
            other => other.clone(),
        }
    }
    let out = walk(&cfg, r, &mut changed);
    if !changed || read_json(&file)? != Some(cfg) {
        return Ok(());
    }
    write_json(&file, &out)?;
    report.changed += 1;
    Ok(())
}

fn claude_folder_name(p: &str) -> String {
    p.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn merge_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for e in fs::read_dir(src)?.flatten() {
        let (a, b) = (e.path(), dst.join(e.file_name()));
        if !b.exists() {
            fs::rename(&a, &b)?;
            continue;
        }
        let (ma, mb) = (fs::metadata(&a)?, fs::metadata(&b)?);
        if ma.is_dir() && mb.is_dir() {
            merge_dir(&a, &b)?;
        } else if ma.is_file() && mb.is_file() && ma.modified()? > mb.modified()? {
            fs::remove_file(&b)?;
            fs::rename(&a, &b)?;
        }
    }
    let _ = fs::remove_dir(src);
    Ok(())
}

fn relocate_claude_projects(p: &Paths, from: &str, to: &str, report: &mut Moved) {
    let dir = p.claude.join("projects");
    let (old, new) = (claude_folder_name(from), claude_folder_name(to));
    for e in fs::read_dir(&dir).into_iter().flatten().flatten() {
        if !e.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        if name.len() < old.len() || !name[..old.len()].eq_ignore_ascii_case(&old) {
            continue;
        }
        let rest = &name[old.len()..];
        if !rest.is_empty() && !rest.starts_with('-') {
            continue;
        }
        let mut target = format!("{new}{rest}");
        if name.starts_with(|c: char| c.is_ascii_lowercase()) {
            target = format!("{}{}", target[..1].to_ascii_lowercase(), &target[1..]);
        }
        if target == name {
            continue;
        }
        let (src, dst) = (e.path(), dir.join(&target));
        let r = if dst.exists() {
            merge_dir(&src, &dst)
        } else {
            fs::rename(&src, &dst)
        };
        match r {
            Ok(()) => report.changed += 1,
            Err(err) => report.failed.push(format!("{} ({err})", src.display())),
        }
    }
}

fn files_in(dir: &Path, filter: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_type().is_ok_and(|t| t.is_file()) && filter(&e.file_name().to_string_lossy())
        })
        .map(|e| e.path())
        .collect()
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect()
}

fn venv_files(venv: &Path) -> Vec<PathBuf> {
    if !venv.join("pyvenv.cfg").is_file() {
        return Vec::new();
    }
    let mut out = vec![venv.join("pyvenv.cfg")];
    out.extend(files_in(&venv.join("Scripts"), |_| true));
    out.extend(files_in(&venv.join(r"Lib\site-packages"), |n| {
        n.ends_with(".pth") || (n.starts_with("__editable__") && n.ends_with(".py"))
    }));
    out
}

/// What the kit's programs wrote with the root in it (not logs, caches or
/// session histories).
fn targets(kit: &Kit, p: &Paths) -> Vec<PathBuf> {
    let data = kit.data();
    let mut out = vec![
        p.claude.join("settings.json"),
        p.mcp_file(),
        p.codex.join("config.toml"),
    ];
    out.extend(files_in(&p.claude.join("plugins"), |n| {
        n.ends_with(".json")
    }));
    out.extend(files_in(&p.claude.join("chrome"), |_| true));
    out.extend(files_in(
        &kit.home()
            .join(r"AppData\Roaming\Claude Code\ChromeNativeHost"),
        |_| true,
    ));
    out.extend(files_in(&kit.home(), |_| true));
    for d in [
        p.gemini.clone(),
        p.gemini.join("config"),
        p.gemini.join("antigravity-cli"),
    ] {
        out.extend(files_in(&d, |n| n.ends_with(".json")));
    }
    // The integrated editor.
    let server = data.join("vscodium-server");
    out.push(server.join(r"data\Machine\settings.json"));
    out.push(server.join(r"extensions\extensions.json"));
    out.extend(files_in(&server.join(r"data\User"), |n| {
        n.ends_with(".json")
    }));
    out.extend(files_in(&server.join(r"data\User\globalStorage"), |n| {
        n.ends_with(".json")
    }));
    // Python: pip --user and uv tool commands, and the projects' venvs.
    for d in subdirs(&data.join("python-user")) {
        out.extend(files_in(&d.join("Scripts"), |_| true));
    }
    out.extend(files_in(&data.join(r"uv\bin"), |_| true));
    for d in subdirs(&data.join(r"uv\tools")) {
        out.extend(venv_files(&d));
    }
    for proj in subdirs(&kit.projects()) {
        if proj
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        {
            continue;
        }
        for v in [".venv", "venv"] {
            out.extend(venv_files(&proj.join(v)));
        }
        for sub in subdirs(&proj) {
            let n = sub
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if n.starts_with('.') || n == "node_modules" {
                continue;
            }
            for v in [".venv", "venv"] {
                out.extend(venv_files(&sub.join(v)));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Replaces the old root (from `data\app\old-root.txt`) everywhere.
pub fn relocate(kit: &Kit, report: &mut Report) -> Result<()> {
    let pending = kit.app_dir().join("old-root.txt");
    let Ok(from) = fs::read_to_string(&pending) else {
        return Ok(());
    };
    let from = from
        .trim()
        .trim_matches('"')
        .trim_end_matches(['\\', '/'])
        .to_owned();
    let to = kit.root().display().to_string();
    if !from.is_empty() && !from.eq_ignore_ascii_case(&to) {
        let p = Paths::of(kit);
        let r = Replacer::new(&from, &to);
        let mut moved = Moved::default();
        if let Err(e) = relocate_claude_json(&p, &r, &mut moved) {
            moved
                .failed
                .push(format!("{} ({e:#})", p.claude_json().display()));
        }
        relocate_claude_projects(&p, &from, &to, &mut moved);
        for f in targets(kit, &p) {
            relocate_file(&f, &r, &mut moved);
        }
        report.note(format!(
            "kit moved from \"{from}\" to \"{to}\": {} files and folders updated",
            moved.changed
        ));
        for f in &moved.programs {
            report.warn(format!("{}: holds the old path and cannot be updated (different length): reinstall it or recreate the .venv (uv sync)", f.display()));
        }
        for f in &moved.failed {
            report.warn(format!("{f}: path not updated"));
        }
        crate::log::info(&format!(
            "relocate {from} -> {to}: {} changed, {} programs, {} failed",
            moved.changed,
            moved.programs.len(),
            moved.failed.len()
        ));
    }
    let _ = fs::remove_file(&pending);
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/agents/relocate.rs"]
mod tests;
