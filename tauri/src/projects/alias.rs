//! Path aliases and `.localhost` domains of a served site (`/media` ->
//! `storage/uploads`, `/shared` -> `@common/assets`). They come from the
//! project's panel.json, a file of its repository: here they are parsed
//! (plain names only, `@` for another project) and resolved on disk without
//! following links (DECISIONS.md D-77) before a web server configuration may
//! contain them. The Apache and Caddy generators take only [`SiteAlias`], so
//! an alias that was not checked cannot reach a configuration.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::kit::validate_project_name;
use crate::os::safe_path;

/// RFC 6761 `.localhost` domain (`mio-sito.localhost`).
pub fn valid_domain(d: &str) -> bool {
    let d = d.trim();
    let Some(prefix) = d.strip_suffix(".localhost") else {
        return false;
    };
    !prefix.is_empty()
        && d.len() <= 120
        && prefix.split('.').all(|part| {
            !part.is_empty()
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        })
}

/// URL path prefix of an alias (`/media`, `/uploads`): `/`, then ASCII
/// letters, digits and `-_./`; no `..`, no trailing `/`.
pub fn valid_url(u: &str) -> bool {
    let u = u.trim();
    u.starts_with('/')
        && u.len() >= 2
        && u.len() <= 80
        && !u.ends_with('/')
        && !u.contains("..")
        && u.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.'))
}

/// Where an alias points, parsed from `storage/uploads`, `.` or `@common/assets`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A folder of the project itself (no parts: its root).
    Project(Vec<String>),
    /// A folder of another project of `projects\`.
    Other { project: String, parts: Vec<String> },
}

/// A name that is safe as it is in any configuration: ASCII letters, digits
/// and `-_.`, not starting with `.` (so never `..`).
fn plain(part: &str) -> bool {
    !part.is_empty()
        && !part.starts_with('.')
        && part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

impl Target {
    /// `None` for an absolute path, a drive, `..`, or any character outside
    /// the plain names (quotes, line breaks, `$`, `{`, `;`, spaces).
    pub fn parse(target: &str) -> Option<Target> {
        let t = target.trim().replace('\\', "/");
        if t.is_empty()
            || t.len() > 160
            || t.starts_with('/')
            || t.contains(':')
            || t.contains("..")
        {
            return None;
        }
        let (other, body) = match t.strip_prefix('@') {
            Some(rest) => {
                let (project, body) = rest.split_once('/').unwrap_or((rest, "."));
                (Some(project), body)
            }
            None => (None, t.as_str()),
        };
        let mut parts = Vec::new();
        for part in body.split('/') {
            if part == "." {
                continue;
            }
            if !plain(part) {
                return None;
            }
            parts.push(part.to_owned());
        }
        match other {
            None => Some(Target::Project(parts)),
            Some(project) if plain(project) && validate_project_name(project).is_ok() => {
                Some(Target::Other {
                    project: project.to_owned(),
                    parts,
                })
            }
            Some(_) => None,
        }
    }
}

/// An alias checked on disk, ready for a configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteAlias {
    /// `/media`
    pub url: String,
    /// `None`: a folder of the project itself.
    other: Option<String>,
    parts: Vec<String>,
    /// The folder on disk.
    pub dir: PathBuf,
}

impl SiteAlias {
    /// The folder for Apache: under `own` (the expression of the project's
    /// folder, `${CP_PROJECT_FWD}`) or under `${CP_PROJECTS_FWD}` for another
    /// project; made only of checked names, so portable and safe in quotes.
    pub fn apache_path(&self, own: &str) -> String {
        let mut s = match &self.other {
            None => own.to_owned(),
            Some(project) => format!("${{CP_PROJECTS_FWD}}/{project}"),
        };
        for part in &self.parts {
            s.push('/');
            s.push_str(part);
        }
        s
    }
}

/// The aliases of a site that can be served: URL and target parsed, the
/// folder existing under `projects_root` with no link or junction on the way
/// (checked as text, never through `canonicalize`, which follows links). The
/// others are left out, each with a warning in the log.
pub fn resolve(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<SiteAlias> {
    aliases
        .iter()
        .filter_map(|(url, target)| {
            let found = resolve_one(project_root, projects_root, url, target);
            if found.is_none() {
                crate::common::log::warn(&format!(
                    "{}: alias {url} -> {target} not served (not a folder of the projects, or through a link)",
                    project_root.display()
                ));
            }
            found
        })
        .collect()
}

/// The URLs of the aliases [`resolve`] leaves out, for the interface to say
/// so (without the log warning: `resolve` already wrote it).
pub fn skipped(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<String> {
    aliases
        .iter()
        .filter(|(url, target)| resolve_one(project_root, projects_root, url, target).is_none())
        .map(|(url, _)| url.trim().to_owned())
        .collect()
}

/// How deep and how wide [`look_inside`] goes: deeper and wider than the
/// project walk of detection (3 levels, 3000 files), since an alias is often
/// a folder of uploads by year and month.
pub const INSIDE_DEPTH: usize = 8;
pub const INSIDE_ENTRIES: usize = 20_000;

/// What [`look_inside`] found in an alias' folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inside {
    /// No link or junction anywhere in it.
    Clean,
    /// This entry is a link or a junction (or could not be read).
    Link(PathBuf),
    /// Deeper than [`INSIDE_DEPTH`] or with more than [`INSIDE_ENTRIES`]
    /// entries: what was not looked at is not trusted.
    TooLarge,
}

/// Looks for links and junctions inside `dir` (itself already checked by
/// [`resolve`]) without following any. A folder or entry that cannot be read
/// counts as a link: Caddy might read what the scan could not.
pub fn look_inside(dir: &Path) -> Inside {
    fn walk(d: &Path, depth: usize, seen: &mut usize) -> Inside {
        let Ok(entries) = std::fs::read_dir(d) else {
            return Inside::Link(d.to_path_buf());
        };
        for e in entries {
            let Ok(e) = e else {
                return Inside::Link(d.to_path_buf());
            };
            *seen += 1;
            if *seen > INSIDE_ENTRIES {
                return Inside::TooLarge;
            }
            // A DirEntry's metadata is the entry's own, never its target's.
            let Ok(meta) = e.metadata() else {
                return Inside::Link(e.path());
            };
            if safe_path::is_reparse(&meta) {
                return Inside::Link(e.path());
            }
            if meta.is_dir() {
                if depth >= INSIDE_DEPTH {
                    return Inside::TooLarge;
                }
                match walk(&e.path(), depth + 1, seen) {
                    Inside::Clean => {}
                    other => return other,
                }
            }
        }
        Inside::Clean
    }
    let mut seen = 0;
    walk(dir, 0, &mut seen)
}

fn resolve_one(
    project_root: &Path,
    projects_root: &Path,
    url: &str,
    target: &str,
) -> Option<SiteAlias> {
    let url = url.trim();
    if !valid_url(url) {
        return None;
    }
    let (other, parts) = match Target::parse(target)? {
        Target::Project(parts) => (None, parts),
        Target::Other { project, parts } => (Some(project), parts),
    };
    let base = match &other {
        None => project_root.to_path_buf(),
        Some(project) => projects_root.join(project),
    };
    let dir = safe_path::lexical(&parts.iter().fold(base, |d, p| d.join(p)))?;
    let root = safe_path::lexical(projects_root)?;
    if !safe_path::is_dir(&root, &dir) {
        return None;
    }
    Some(SiteAlias {
        url: url.to_owned(),
        other,
        parts,
        dir,
    })
}

#[cfg(test)]
#[path = "../../../test/rust/projects/alias.rs"]
mod tests;
