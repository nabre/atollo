//! The relations matrix (DECISIONS.md D-81): what each component needs or
//! uses, at which versions, what the PC must provide, and the versions the
//! projects declare, with what the kit has and what to download.
//!
//! The knowledge is `catalog/relations.json` (embedded); the state is read
//! from the kit (installed versions and lines, the kit's and the projects'
//! choices, the projects' files). The matrix is saved in
//! `data\app\relations.json` with a fingerprint of everything it was made
//! from: it is built again only when that changes.
//!
//! Here the matrix, what the kit has of each component and the saved copy;
//! the knowledge in `known`, the links and runtimes in `checks`, the
//! projects' declared versions in `projects`.

mod checks;
mod known;
mod projects;

use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::ranges::{Range, Version};
use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Component};
use checks::{links, runtimes};
#[cfg(test)]
pub use known::notes;
use known::{known, KNOWN};
use projects::{project_dirs, projects};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkStatus {
    /// `from` is not installed: nothing to check.
    Inactive,
    Ok,
    /// `to` is missing and `from` needs it.
    Missing,
    /// `to` is missing, `from` works without it.
    Optional,
    /// The version of `to` the kit uses is outside `range`.
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub from: String,
    pub from_name: String,
    pub to: String,
    pub to_name: String,
    /// `needs`, `uses`, `runs-on`, `loads`.
    pub kind: String,
    pub optional: bool,
    pub range: Option<String>,
    pub note: Option<String>,
    pub status: LinkStatus,
    /// The version of `to` the kit uses.
    pub uses: Option<String>,
    /// Lines of `to` on disk outside `range` (the kit or a project may
    /// choose them: `from` will not work there).
    pub outside: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeState {
    pub id: String,
    pub name: String,
    pub note: Option<String>,
    /// The installed components that need it.
    pub needed_by: Vec<String>,
    pub present: bool,
    /// Its version, or where it comes from.
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NeedStatus {
    /// The project's line (or the kit's) fits.
    Ok,
    /// A line on disk fits: choose it for the project.
    Choose,
    /// A line the source offers fits: download it.
    Download,
    /// Nothing the kit has or offers fits.
    Unavailable,
    /// The component is not installed.
    NotInstalled,
    /// The declared text is not a version range the kit reads.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Need {
    pub component: String,
    pub name: String,
    /// `package.json engines.node`.
    pub source: String,
    pub declared: String,
    pub status: NeedStatus,
    /// The line the project uses now.
    pub uses: Option<String>,
    /// For `choose` and `download`: the line that fits.
    pub line: Option<String>,
    /// A database server, one for every project: `choose` is the version
    /// the server runs (Database page), not the project's.
    #[serde(default)]
    pub server: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectNeeds {
    pub project: String,
    pub needs: Vec<Need>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matrix {
    pub fingerprint: String,
    /// Unix seconds.
    pub generated: u64,
    pub platform: String,
    pub links: Vec<Link>,
    pub runtimes: Vec<RuntimeState>,
    pub projects: Vec<ProjectNeeds>,
}

/// What the kit has of a component: installed, and its lines with their
/// full versions (default first), the kit's line.
#[derive(Clone)]
struct Have {
    installed: bool,
    /// `(line, version)`, the default first.
    lines: Vec<(String, Option<String>)>,
    kit: Option<String>,
    /// Lines the source offers (versioned components that download).
    offered: Vec<String>,
}

fn have(kit: &Kit, c: &Component) -> Have {
    let installed = c.is_installed(kit);
    if !c.versioned {
        let version = c.installed_version(kit);
        let line = version.as_deref().and_then(crate::versions::line_of);
        return Have {
            installed,
            lines: line.map(|l| (l, version)).into_iter().collect(),
            kit: None,
            offered: Vec::new(),
        };
    }
    let mut lines = Vec::new();
    if let Some(line) = crate::versions::default_line(kit, c) {
        lines.push((line, c.installed_version(kit)));
    }
    for l in crate::versions::saved(kit, c) {
        lines.push((l.line, l.version));
    }
    let choice = crate::versions::kit_choice(kit);
    let kit_line = choice
        .get(&c.id)
        .filter(|l| lines.iter().any(|(x, _)| x == *l))
        .cloned()
        .or_else(|| lines.first().map(|(l, _)| l.clone()));
    let downloadable = crate::install::installer(&c.id).is_some_and(|i| i.has_lines());
    let offered = if downloadable {
        crate::versions::offered(kit)
            .remove(&c.id)
            .map(|o| o.lines)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Have {
        installed,
        lines,
        kit: kit_line,
        offered,
    }
}

/// `php/composer` → the component (`php`) and the part's name.
fn split(id: &str) -> (&str, Option<&str>) {
    match id.split_once('/') {
        Some((c, part)) => (c, Some(part)),
        None => (id, None),
    }
}

fn part_name(part: &str) -> String {
    let mut chars = part.chars();
    chars
        .next()
        .map(|f| f.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

fn name_of(catalog: &Catalog, id: &str) -> String {
    match split(id) {
        (_, Some(part)) => part_name(part),
        (c, None) => catalog
            .find(c)
            .map_or_else(|| c.to_owned(), |c| c.name.clone()),
    }
}

/// The version text of `line` in `have`.
fn version_of(have: &Have, line: &str) -> String {
    have.lines
        .iter()
        .find(|(l, _)| l == line)
        .and_then(|(_, v)| v.clone())
        .unwrap_or_else(|| line.to_owned())
}

/// `line` in `have` is in the range: its full version when known (a
/// `>=3.12.4` refuses the kit's 3.12.3), else any patch of the line.
fn admits(range: &Range, have: &Have, line: &str) -> bool {
    let full = have
        .lines
        .iter()
        .find(|(l, _)| l == line)
        .and_then(|(_, v)| v.as_deref())
        .filter(|v| Version::parse(v).is_some());
    match full {
        Some(v) => range.admits_text(v),
        None => range.admits_line(line),
    }
}

/// Everything the matrix is made from, hashed: the app and its knowledge,
/// the kit's target and the language, the installed versions and lines, the
/// choices, the lines offered, and each project's files that declare
/// versions (size and time).
fn fingerprint(kit: &Kit, lang: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    env!("CARGO_PKG_VERSION").hash(&mut h);
    KNOWN.hash(&mut h);
    kit.target().as_str().hash(&mut h);
    lang.hash(&mut h);
    let read = |p: &Path| fs::read(p).unwrap_or_default();
    for c in Catalog::get().all() {
        c.is_installed(kit).hash(&mut h);
        c.installed_version(kit).hash(&mut h);
        if c.versioned {
            for l in crate::versions::saved(kit, c) {
                (l.line, l.version).hash(&mut h);
            }
        }
    }
    read(&crate::versions::offered_file(kit)).hash(&mut h);
    crate::fsx::join_rel(kit.apps(), "tools/vcruntime/vcruntime140.dll")
        .is_file()
        .hash(&mut h);
    for (project, dir) in project_dirs(kit) {
        project.hash(&mut h);
        crate::versions::project_choice(kit, &project).hash(&mut h);
        for spec in &known().projects {
            if let Ok(m) = fs::metadata(dir.join(&spec.file)) {
                (&spec.file, m.len(), m.modified().ok()).hash(&mut h);
            }
        }
    }
    format!("{:016x}", h.finish())
}

/// `relations.json` for this PC's system, `relations-<target>.json` for
/// another one chosen on the Tools page: switching does not build again
/// the matrix of the other.
fn file(kit: &Kit) -> std::path::PathBuf {
    if kit.target() == crate::kit::Target::detect() {
        kit.app_dir().join("relations.json")
    } else {
        kit.app_dir()
            .join(format!("relations-{}.json", kit.target().as_str()))
    }
}

/// Alphabetical by the component, then by what it needs.
fn sorted_links(mut links: Vec<Link>) -> Vec<Link> {
    links.sort_by_key(|l| (l.from_name.to_lowercase(), l.to_name.to_lowercase()));
    links
}

/// Builds the matrix from the state of the kit now, for the kit's target:
/// the links between components that target cannot have (PHP's on a
/// system without PHP, Android's on Linux ARM64) are left out.
pub fn build(kit: &Kit, lang: &str) -> Matrix {
    let catalog = Catalog::get();
    let haves: BTreeMap<String, Have> = catalog
        .all()
        .iter()
        .map(|c| (c.id.clone(), have(kit, c)))
        .collect();
    let target = kit.target();
    let here: BTreeMap<String, Have> = haves
        .iter()
        .filter(|(id, _)| catalog.find(id).is_some_and(|c| c.is_available_on(target)))
        .map(|(id, h)| (id.clone(), h.clone()))
        .collect();
    Matrix {
        fingerprint: fingerprint(kit, lang),
        generated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        platform: target.as_str().to_owned(),
        links: sorted_links(links(lang, &here)),
        runtimes: {
            let mut r = runtimes(kit, lang, &haves);
            r.sort_by_key(|x| x.name.to_lowercase());
            r
        },
        projects: projects(kit, &haves),
    }
}

/// A project copied from elsewhere (clone): each version it declares gets
/// the newest line that fits, chosen for the project. Returns the lines to
/// download (chosen already: until they arrive the kit's line is used).
pub fn settle(kit: &Kit, lang: &str, project: &str) -> Vec<(String, String)> {
    let matrix = build(kit, lang);
    let Some(p) = matrix.projects.iter().find(|p| p.project == project) else {
        return Vec::new();
    };
    let mut downloads = Vec::new();
    for need in &p.needs {
        let Some(line) = &need.line else { continue };
        // A server's version is the kit's to choose, for every project.
        if need.server || !matches!(need.status, NeedStatus::Choose | NeedStatus::Download) {
            continue;
        }
        // The first declaration of a component wins (package.json before .nvmrc).
        let chosen = crate::versions::project_choice(kit, project);
        if chosen.contains_key(&need.component) {
            continue;
        }
        if let Err(e) =
            crate::versions::set_project_choice(kit, project, &need.component, Some(line))
        {
            crate::log::warn(&format!("{project}: {} {line}: {e:#}", need.component));
            continue;
        }
        if need.status == NeedStatus::Download {
            downloads.push((need.component.clone(), line.clone()));
        }
    }
    downloads
}

/// The saved matrix while nothing it was made from changed; otherwise (or
/// with `force`) a new one, saved.
pub fn matrix(kit: &Kit, lang: &str, force: bool) -> Result<Matrix> {
    let fp = fingerprint(kit, lang);
    if !force {
        let saved = crate::fsx::read_json::<Matrix>(&file(kit));
        if let Some(m) = saved.filter(|m| m.fingerprint == fp) {
            return Ok(m);
        }
    }
    let m = build(kit, lang);
    crate::fsx::write_json(&file(kit), &m)?;
    Ok(m)
}

#[cfg(test)]
#[path = "../../../../test/rust/versions/relations/mod.rs"]
mod tests;
