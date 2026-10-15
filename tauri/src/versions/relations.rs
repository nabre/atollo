//! The relations matrix (DECISIONS.md D-81): what each component needs or
//! uses, at which versions, what the PC must provide, and the versions the
//! projects declare, with what the kit has and what to download.
//!
//! The knowledge is `catalog/relations.json` (embedded); the state is read
//! from the kit (installed versions and lines, the kit's and the projects'
//! choices, the projects' files). The matrix is saved in
//! `data\app\relations.json` with a fingerprint of everything it was made
//! from: it is built again only when that changes.

use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ranges::{Bare, Range, Version};
use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Component};

const KNOWN: &str = include_str!("../../catalog/relations.json");

#[derive(Debug, Deserialize)]
struct Known {
    links: Vec<LinkSpec>,
    runtimes: Vec<RuntimeSpec>,
    projects: Vec<ProjectSpec>,
}

#[derive(Debug, Deserialize)]
struct LinkSpec {
    from: String,
    to: String,
    kind: String,
    #[serde(default)]
    range: Option<String>,
    #[serde(default)]
    optional: bool,
    /// Only the version of `to` the kit uses counts: `from` picks it
    /// whatever a project uses (Gradle on the newest JDK, Composer per PHP).
    #[serde(default)]
    kit: bool,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RuntimeSpec {
    id: String,
    name: String,
    /// Component ids; `@app` = the app itself, `@x64Only` = every
    /// component without an ARM64 build.
    #[serde(rename = "for")]
    for_: Vec<String>,
    /// Only on ARM64 kits.
    #[serde(default)]
    arm64: bool,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectSpec {
    component: String,
    file: String,
    /// A dotted path in a JSON file (`engines.node`).
    #[serde(default)]
    json: Option<String>,
    /// Group 1 of a regex over the file.
    #[serde(default)]
    regex: Option<String>,
    /// The first line of the file.
    #[serde(default)]
    whole: bool,
    /// A version that names a line (`3.12.4` = any 3.12).
    #[serde(default)]
    line: bool,
    /// A bare version is a minimum (`go 1.21`).
    #[serde(default)]
    minimum: bool,
}

/// The notes of the knowledge (Italian), translated by the catalogs.
#[cfg(test)]
pub fn notes() -> Vec<&'static str> {
    let k = known();
    k.links
        .iter()
        .filter_map(|l| l.note.as_deref())
        .chain(k.runtimes.iter().filter_map(|r| r.note.as_deref()))
        .collect()
}

fn known() -> &'static Known {
    static KNOWN_DATA: std::sync::OnceLock<Known> = std::sync::OnceLock::new();
    KNOWN_DATA.get_or_init(|| {
        // Validated by the tests: a broken file ships an empty matrix.
        serde_json::from_str(KNOWN).unwrap_or(Known {
            links: Vec::new(),
            runtimes: Vec::new(),
            projects: Vec::new(),
        })
    })
}

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

fn admits(range: &Range, have: &Have, line: &str) -> bool {
    range.admits_text(&version_of(have, line)) || range.admits_line(line)
}

fn links(lang: &str, haves: &BTreeMap<String, Have>) -> Vec<Link> {
    let catalog = Catalog::get();
    known()
        .links
        .iter()
        .filter(|s| haves.contains_key(split(&s.from).0) && haves.contains_key(&s.to))
        .map(|s| {
            let from = &haves[split(&s.from).0];
            let to = &haves[&s.to];
            let range = s
                .range
                .as_deref()
                .and_then(|r| Range::parse(r, Bare::Exact));
            // Only the kit's version counts: no lines "outside".
            let per_line = s.kit;
            let uses = to
                .kit
                .clone()
                .or_else(|| to.lines.first().map(|(l, _)| l.clone()));
            let outside: Vec<String> = match (&range, per_line) {
                (Some(r), false) => to
                    .lines
                    .iter()
                    .map(|(l, _)| l.clone())
                    .filter(|l| !admits(r, to, l))
                    .collect(),
                _ => Vec::new(),
            };
            let status = if !from.installed {
                LinkStatus::Inactive
            } else if !to.installed {
                if s.optional {
                    LinkStatus::Optional
                } else {
                    LinkStatus::Missing
                }
            } else if uses.as_deref().is_some_and(|l| match &range {
                // Composer is chosen for each PHP folder: it always fits.
                _ if s.from == "php/composer" => false,
                Some(r) => !admits(r, to, l),
                None => false,
            }) {
                LinkStatus::Incompatible
            } else {
                LinkStatus::Ok
            };
            Link {
                from: s.from.clone(),
                from_name: name_of(catalog, &s.from),
                to: s.to.clone(),
                to_name: name_of(catalog, &s.to),
                kind: s.kind.clone(),
                optional: s.optional,
                range: s.range.clone(),
                note: s.note.as_deref().map(|n| crate::i18n::t(lang, n, &[])),
                status,
                uses: uses.map(|l| version_of(to, &l)),
                outside,
            }
        })
        .collect()
}

fn runtimes(kit: &Kit, lang: &str, haves: &BTreeMap<String, Have>) -> Vec<RuntimeState> {
    let catalog = Catalog::get();
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    let installed = |id: &str| haves.get(id).is_some_and(|h| h.installed);
    known()
        .runtimes
        .iter()
        .filter(|s| !s.arm64 || arm64)
        .map(|s| {
            let mut needed_by: Vec<String> = Vec::new();
            for f in &s.for_ {
                match f.as_str() {
                    "@app" => needed_by.push("Atollo".into()),
                    "@x64Only" => needed_by.extend(
                        catalog
                            .all()
                            .iter()
                            .filter(|c| c.x64_only && installed(&c.id))
                            .map(|c| c.name.clone()),
                    ),
                    id if installed(id) => needed_by.push(name_of(catalog, id)),
                    _ => {}
                }
            }
            let (present, detail) = match s.id.as_str() {
                "vcruntime" => {
                    let dll = kit.apps().join(r"tools\vcruntime\vcruntime140.dll");
                    (dll.is_file(), None)
                }
                "webview2" => match crate::os::webview::found(kit) {
                    Some(crate::os::webview::Runtime::System(v)) => (true, Some(v)),
                    Some(crate::os::webview::Runtime::Fixed { version, .. }) => {
                        (true, Some(version))
                    }
                    None => (false, None),
                },
                "x64-emulation" => {
                    let build = crate::editor::wterm::windows_build();
                    (
                        build.is_none_or(|b| b >= 22000),
                        build.map(|b| format!("Windows build {b}")),
                    )
                }
                _ => (false, None),
            };
            RuntimeState {
                id: s.id.clone(),
                name: s.name.clone(),
                note: s.note.as_deref().map(|n| crate::i18n::t(lang, n, &[])),
                needed_by,
                present,
                detail,
            }
        })
        .collect()
}

/// The version text `spec` reads in `dir`, if the file says one.
fn declared(dir: &Path, spec: &ProjectSpec) -> Option<String> {
    let text = crate::fsx::read_text(&dir.join(&spec.file)).ok()?;
    let text = text.as_str();
    let found = if let Some(path) = &spec.json {
        let v: Value = serde_json::from_str(text).ok()?;
        let pointer = format!("/{}", path.replace('.', "/"));
        v.pointer(&pointer)?.as_str()?.to_owned()
    } else if let Some(re) = &spec.regex {
        regex::Regex::new(re)
            .ok()?
            .captures(text)?
            .get(1)?
            .as_str()
            .to_owned()
    } else if spec.whole {
        text.lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with('#'))?
            .to_owned()
    } else {
        return None;
    };
    let found = found.trim().to_owned();
    (!found.is_empty()).then_some(found)
}

/// `3.12.4` → `3.12` for a version that names a line; `22` stays.
fn as_line(text: &str) -> String {
    let start = text.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
    let digits: String = text[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    digits
        .split('.')
        .filter(|p| !p.is_empty())
        .take(2)
        .collect::<Vec<_>>()
        .join(".")
}

fn need(
    kit: &Kit,
    project: &str,
    spec: &ProjectSpec,
    text: String,
    haves: &BTreeMap<String, Have>,
) -> Option<Need> {
    let catalog = Catalog::get();
    let c = catalog.find(&spec.component)?;
    let have = haves.get(&c.id)?;
    let source = match (&spec.json, spec.regex.is_some()) {
        (Some(path), _) => format!("{} {path}", spec.file),
        _ => spec.file.clone(),
    };
    let wanted = if spec.line {
        as_line(&text)
    } else {
        text.clone()
    };
    let bare = if spec.minimum {
        Bare::Minimum
    } else {
        Bare::Exact
    };
    let mut need = Need {
        component: c.id.clone(),
        name: c.name.clone(),
        source,
        declared: text,
        status: NeedStatus::Unknown,
        uses: None,
        line: None,
        server: crate::db::services::SERVERS.contains(&c.id.as_str()),
    };
    let Some(range) = Range::parse(&wanted, bare) else {
        return Some(need);
    };
    let fits = |l: &String| admits(&range, have, l);
    let chosen = crate::versions::project_choice(kit, project).remove(&c.id);
    need.uses = if need.server {
        crate::db::services::server_line(kit, &c.id)
    } else {
        chosen
            .clone()
            .filter(|l| have.lines.iter().any(|(x, _)| x == l))
            .or_else(|| have.kit.clone())
            .or_else(|| have.lines.first().map(|(l, _)| l.clone()))
    };
    let newest = |lines: &mut dyn Iterator<Item = String>| {
        let mut v: Vec<String> = lines.collect();
        v.sort_by_key(|l| std::cmp::Reverse(Version::parse(l)));
        v.into_iter().next()
    };
    let offered = newest(&mut have.offered.iter().filter(|l| fits(l)).cloned());
    (need.status, need.line) = if !have.installed {
        (NeedStatus::NotInstalled, offered)
    } else if need.uses.as_ref().is_some_and(fits) {
        (NeedStatus::Ok, None)
    } else if let Some(l) = newest(&mut have.lines.iter().map(|(l, _)| l.clone()).filter(fits)) {
        (NeedStatus::Choose, Some(l))
    } else if let Some(l) = offered.filter(|_| c.versioned) {
        (NeedStatus::Download, Some(l))
    } else {
        (NeedStatus::Unavailable, None)
    };
    Some(need)
}

fn projects(kit: &Kit, haves: &BTreeMap<String, Have>) -> Vec<ProjectNeeds> {
    let mut out: Vec<ProjectNeeds> = project_dirs(kit)
        .into_iter()
        .map(|(project, dir)| ProjectNeeds {
            needs: known()
                .projects
                .iter()
                .filter_map(|spec| {
                    let text = declared(&dir, spec)?;
                    need(kit, &project, spec, text, haves)
                })
                .collect(),
            project,
        })
        .filter(|p| !p.needs.is_empty())
        .collect();
    // Alphabetical, as every list of the page.
    out.sort_by_key(|p| p.project.to_lowercase());
    for p in &mut out {
        // By component; for one component, the order of the knowledge
        // (package.json before .nvmrc: the first declaration wins).
        p.needs.sort_by_key(|n| n.name.to_lowercase());
    }
    out
}

fn project_dirs(kit: &Kit) -> Vec<(String, std::path::PathBuf)> {
    fs::read_dir(kit.projects())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect()
}

/// Everything the matrix is made from, hashed: the app and its knowledge,
/// the platform and language, the installed versions and lines, the
/// choices, the lines offered, and each project's files that declare
/// versions (size and time).
fn fingerprint(kit: &Kit, lang: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    env!("CARGO_PKG_VERSION").hash(&mut h);
    KNOWN.hash(&mut h);
    kit.arch().platform().hash(&mut h);
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
    read(&kit.app_dir().join("tool-versions.json")).hash(&mut h);
    read(&crate::versions::offered_file(kit)).hash(&mut h);
    kit.apps()
        .join(r"tools\vcruntime\vcruntime140.dll")
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

fn file(kit: &Kit) -> std::path::PathBuf {
    kit.app_dir().join("relations.json")
}

/// Alphabetical by the component, then by what it needs.
fn sorted_links(mut links: Vec<Link>) -> Vec<Link> {
    links.sort_by_key(|l| (l.from_name.to_lowercase(), l.to_name.to_lowercase()));
    links
}

/// Builds the matrix from the state of the kit now.
pub fn build(kit: &Kit, lang: &str) -> Matrix {
    let catalog = Catalog::get();
    let haves: BTreeMap<String, Have> = catalog
        .all()
        .iter()
        .map(|c| (c.id.clone(), have(kit, c)))
        .collect();
    Matrix {
        fingerprint: fingerprint(kit, lang),
        generated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        platform: kit.arch().platform(),
        links: sorted_links(links(lang, &haves)),
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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn knowledge_is_valid() {
        let k: Known = serde_json::from_str(KNOWN).unwrap();
        let catalog = Catalog::get();
        let parts = ["composer", "gradle", "maven"];
        for l in &k.links {
            let (c, part) = split(&l.from);
            assert!(catalog.find(c).is_some(), "{}", l.from);
            assert!(part.is_none_or(|p| parts.contains(&p)), "{}", l.from);
            assert!(catalog.find(&l.to).is_some(), "{}", l.to);
            if let Some(r) = &l.range {
                assert!(Range::parse(r, Bare::Exact).is_some(), "{r}");
            }
            assert!(["needs", "uses", "runs-on", "loads"].contains(&l.kind.as_str()));
        }
        for r in &k.runtimes {
            for f in &r.for_ {
                assert!(f.starts_with('@') || catalog.find(f).is_some(), "{f}");
            }
        }
        for p in &k.projects {
            assert!(catalog.find(&p.component).is_some(), "{}", p.component);
            assert!(
                p.json.is_some() || p.regex.is_some() || p.whole,
                "{}",
                p.file
            );
            if let Some(re) = &p.regex {
                regex::Regex::new(re).unwrap();
            }
        }
    }

    /// `id` installed at `version`: its key files, in the folder of its line.
    fn fake(kit: &Kit, id: &str, version: &str) {
        let c = Catalog::get().find(id).unwrap();
        c.save_version(kit, version).unwrap();
        for k in c.key_paths(kit) {
            fs::create_dir_all(k.parent().unwrap()).unwrap();
            fs::write(&k, b"").unwrap();
        }
    }

    #[test]
    fn links_and_projects() {
        let tmp = TempDir::new("relations");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        fake(&kit, "java", "11.0.25+9 (Maven 3.9.11, Gradle 9.1.0)");
        fake(&kit, "node", "v20.19.5");
        let node = Catalog::get().find("node").unwrap();
        let saved = crate::versions::saved_dir(&kit, node, "22.21");
        fs::create_dir_all(&saved).unwrap();
        fs::write(saved.join("node.exe"), b"").unwrap();
        crate::versions::write_version(&saved, "v22.21.1").unwrap();
        crate::versions::save_offered(
            &kit,
            "node",
            &["24.11".into(), "22.21".into(), "20.19".into()],
        )
        .unwrap();

        let app = kit.projects().join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.json"), r#"{"engines": {"node": ">=22"}}"#).unwrap();
        fs::write(app.join(".nvmrc"), "24\n").unwrap();
        fs::write(app.join("go.mod"), "module x\n\ngo 1.22.1\n").unwrap();
        // The project was made with Node.js 20.19 (the kit now uses 22.21).
        crate::versions::set_project_choice(&kit, "app", "node", Some("20.19")).unwrap();

        let m = build(&kit, "it");
        let gradle = m.links.iter().find(|l| l.from == "java/gradle").unwrap();
        assert_eq!(
            gradle.status,
            LinkStatus::Incompatible,
            "Gradle 9 on Java 11"
        );
        assert!(gradle.outside.is_empty(), "the kit's JDK only counts");
        let kotlin = m.links.iter().find(|l| l.from == "kotlin").unwrap();
        assert_eq!(kotlin.status, LinkStatus::Inactive);
        let pnpm = m.links.iter().find(|l| l.from == "pnpm").unwrap();
        assert_eq!(pnpm.status, LinkStatus::Inactive);

        let needs = &m
            .projects
            .iter()
            .find(|p| p.project == "app")
            .unwrap()
            .needs;
        let engines = needs
            .iter()
            .find(|n| n.source == "package.json engines.node")
            .unwrap();
        assert_eq!(
            (engines.status, engines.line.as_deref()),
            (NeedStatus::Choose, Some("22.21"))
        );
        let nvmrc = needs.iter().find(|n| n.source == ".nvmrc").unwrap();
        assert_eq!(
            (nvmrc.status, nvmrc.line.as_deref()),
            (NeedStatus::Download, Some("24.11"))
        );
        let go = needs.iter().find(|n| n.component == "go").unwrap();
        assert_eq!(go.status, NeedStatus::NotInstalled);

        // A project just cloned: what it declares is chosen, the first
        // declaration of a component wins, nothing to download here.
        let cloned = kit.projects().join("cloned");
        fs::create_dir_all(&cloned).unwrap();
        fs::write(
            cloned.join("package.json"),
            r#"{"engines": {"node": "^20.11"}}"#,
        )
        .unwrap();
        fs::write(cloned.join(".nvmrc"), "24\n").unwrap();
        assert!(settle(&kit, "it", "cloned").is_empty());
        assert_eq!(
            crate::versions::project_choice(&kit, "cloned")
                .get("node")
                .map(String::as_str),
            Some("20.19")
        );
        fs::write(cloned.join("go.mod"), "module x\n").unwrap();
        let cloned2 = kit.projects().join("cloned2");
        fs::create_dir_all(&cloned2).unwrap();
        fs::write(cloned2.join(".nvmrc"), "24\n").unwrap();
        assert_eq!(
            settle(&kit, "it", "cloned2"),
            [("node".to_owned(), "24.11".to_owned())]
        );

        // The project chooses 22.21: engines fit.
        crate::versions::set_project_choice(&kit, "app", "node", Some("22.21")).unwrap();
        let m2 = matrix(&kit, "it", false).unwrap();
        let engines = m2.projects[0]
            .needs
            .iter()
            .find(|n| n.source == "package.json engines.node")
            .unwrap();
        assert_eq!(engines.status, NeedStatus::Ok);
        // Saved, and the same while nothing changes.
        let again = matrix(&kit, "it", false).unwrap();
        assert_eq!(again.generated, m2.generated);
        assert_eq!(again.fingerprint, m2.fingerprint);
        fs::write(app.join(".nvmrc"), "22.21.0\n").unwrap();
        assert_ne!(
            fingerprint(&kit, "it"),
            m2.fingerprint,
            "a project file changed"
        );
    }

    #[test]
    fn declared_versions() {
        let tmp = TempDir::new("relations-declared");
        let dir = &tmp.0;
        let spec = |component: &str, file: &str| {
            known()
                .projects
                .iter()
                .find(|p| p.component == component && p.file == file)
                .unwrap()
        };
        fs::write(
            dir.join("pyproject.toml"),
            "[project]\nname='x'\nrequires-python = \">=3.10,<3.13\"\n",
        )
        .unwrap();
        assert_eq!(
            declared(dir, spec("python", "pyproject.toml")).as_deref(),
            Some(">=3.10,<3.13")
        );
        fs::write(dir.join("composer.json"), r#"{"require": {"php": "^8.2"}}"#).unwrap();
        assert_eq!(
            declared(dir, spec("php", "composer.json")).as_deref(),
            Some("^8.2")
        );
        fs::write(dir.join(".python-version"), "# pinned\n3.12.4\n").unwrap();
        assert_eq!(
            declared(dir, spec("python", ".python-version")).as_deref(),
            Some("3.12.4")
        );
        fs::write(
            dir.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.20...3.31)\n",
        )
        .unwrap();
        assert_eq!(
            declared(dir, spec("cmake", "CMakeLists.txt")).as_deref(),
            Some("3.20")
        );
        fs::write(
            dir.join("pubspec.yaml"),
            "name: x\nenvironment:\n  sdk: ^3.5.0\n",
        )
        .unwrap();
        assert_eq!(
            declared(dir, spec("dart", "pubspec.yaml")).as_deref(),
            Some("^3.5.0")
        );
        fs::write(
            dir.join("Project.toml"),
            "name = \"X\"\n\n[compat]\njulia = \"1.10\"\n",
        )
        .unwrap();
        assert_eq!(
            declared(dir, spec("julia", "Project.toml")).as_deref(),
            Some("1.10")
        );
        fs::write(
            dir.join("renv.lock"),
            r#"{"R": {"Version": "4.4.1"}, "Packages": {}}"#,
        )
        .unwrap();
        assert_eq!(
            declared(dir, spec("r", "renv.lock")).as_deref(),
            Some("4.4.1")
        );
        fs::write(
            dir.join("DESCRIPTION"),
            "Package: x\nDepends: R (>= 4.1.0), utils\n",
        )
        .unwrap();
        assert_eq!(
            declared(dir, spec("r", "DESCRIPTION")).as_deref(),
            Some("4.1.0")
        );
        fs::write(
            dir.join("typst.toml"),
            "[package]\nname = \"x\"\ncompiler = \"0.13.0\"\n",
        )
        .unwrap();
        assert_eq!(
            declared(dir, spec("typst", "typst.toml")).as_deref(),
            Some("0.13.0")
        );
        assert_eq!(as_line("3.12.4"), "3.12");
        assert_eq!(as_line("v22"), "22");
        assert_eq!(as_line("8.0.302"), "8.0");
    }
}
