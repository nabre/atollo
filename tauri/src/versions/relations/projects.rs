//! The versions the projects declare in their files, and what the kit has
//! or offers that fits them.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use super::known::{known, ProjectSpec};
use super::{admits, Have, Need, NeedStatus, ProjectNeeds};
use crate::kit::Kit;
use crate::setup::Catalog;
use crate::versions::ranges::{Bare, Range, Version};

/// The version text `spec` reads in the file `rel` of `dir` (`spec.file`
/// at any depth, D-186), if the file says one.
fn declared(dir: &Path, rel: &str, spec: &ProjectSpec) -> Option<String> {
    let path = dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    let text = crate::platform::safe_path::read_text(dir, &path)?;
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

/// What `dir` declares for every spec of the knowledge, at any depth
/// (D-186): for one spec the shallowest file that says a version; for one
/// component the shallowest declaration first, then the order of the
/// knowledge (package.json before .nvmrc). Different versions declared at
/// other depths go to the log.
pub(super) fn declarations<'a>(
    dir: &Path,
    project: &str,
) -> Vec<(&'a ProjectSpec, String, String)> {
    let walk = crate::detect::walk::walk(dir);
    let mut found: Vec<(usize, &ProjectSpec, String, String)> = Vec::new();
    for spec in &known().projects {
        let mut said = walk
            .named(&spec.file)
            .filter_map(|rel| Some((rel, declared(dir, rel, spec)?)));
        let Some((rel, text)) = said.next() else {
            continue;
        };
        for (other, other_text) in said {
            if other_text != text {
                crate::common::log::info(&format!(
                    "{project}: {other} declares {} {other_text}, {rel} {text} wins (the shallowest)",
                    spec.component
                ));
            }
        }
        found.push((crate::detect::walk::depth(rel), spec, rel.to_owned(), text));
    }
    // Stable: for one depth the knowledge's order stays.
    found.sort_by_key(|(depth, ..)| *depth);
    found
        .into_iter()
        .map(|(_, spec, rel, text)| (spec, rel, text))
        .collect()
}

fn need(
    kit: &Kit,
    project: &str,
    spec: &ProjectSpec,
    rel: &str,
    text: String,
    haves: &BTreeMap<String, Have>,
) -> Option<Need> {
    let catalog = Catalog::get();
    let c = catalog.find(&spec.component)?;
    let have = haves.get(&c.id)?;
    let source = match (&spec.json, spec.regex.is_some()) {
        (Some(path), _) => format!("{rel} {path}"),
        _ => rel.to_owned(),
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
        crate::db::services::project_line(kit, project, &c.id)
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

pub(super) fn projects(kit: &Kit, haves: &BTreeMap<String, Have>) -> Vec<ProjectNeeds> {
    let mut out: Vec<ProjectNeeds> = project_dirs(kit)
        .into_iter()
        .map(|(project, dir)| ProjectNeeds {
            needs: declarations(&dir, &project)
                .into_iter()
                .filter_map(|(spec, rel, text)| need(kit, &project, spec, &rel, text, haves))
                .collect(),
            project,
        })
        .filter(|p| !p.needs.is_empty())
        .collect();
    // Alphabetical, as every list of the page.
    out.sort_by_key(|p| p.project.to_lowercase());
    for p in &mut out {
        // By component; for one component the shallowest declaration, then
        // the order of the knowledge (package.json before .nvmrc: the first
        // declaration wins, D-186).
        p.needs.sort_by_key(|n| n.name.to_lowercase());
    }
    out
}

pub(super) fn project_dirs(kit: &Kit) -> Vec<(String, std::path::PathBuf)> {
    fs::read_dir(kit.projects())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect()
}

#[cfg(test)]
#[path = "../../../../test/rust/versions/relations/projects.rs"]
mod tests;
